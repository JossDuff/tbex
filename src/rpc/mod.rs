mod helper;
mod types;

use helper::*;

pub use types::*;

use alloy::{
    consensus::Transaction as TxTrait,
    eips::{BlockId, BlockNumberOrTag},
    network::{Ethereum, TransactionResponse},
    primitives::{address, keccak256, Address, Bytes, TxHash, TxKind, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolCall,
};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

type HttpProvider = RootProvider<Ethereum>;

// ENS ReverseRecords contract on mainnet (for reverse resolution: address -> name)
const ENS_REVERSE_RECORDS: Address = address!("3671aE578E63FdF66ad4F3E12CC0c0d71Ac7510C");

// ENS Registry contract on mainnet (for forward resolution: name -> address)
const ENS_REGISTRY: Address = address!("00000000000C2E074eC69A0dFb2997BA6C7d2e1e");

sol! {
    #[sol(rpc)]
    interface ReverseRecords {
        function getNames(address[] calldata addresses) external view returns (string[] memory);
    }
}

sol! {
    #[sol(rpc)]
    interface ENSRegistry {
        function resolver(bytes32 node) external view returns (address);
    }
}

sol! {
    #[sol(rpc)]
    interface ENSResolver {
        function addr(bytes32 node) external view returns (address);
    }
}

/// RPC client with retry logic for rate-limited endpoints
pub struct RpcClient {
    provider: HttpProvider,
    max_retries: u32,
    base_delay: Duration,
}

impl RpcClient {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let url = rpc_url.parse().context("Invalid RPC URL")?;
        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .network::<Ethereum>()
            .connect_http(url);

        Ok(Self {
            provider,
            max_retries: 5,
            base_delay: Duration::from_millis(500),
        })
    }

    async fn with_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut all_errors: Vec<String> = Vec::new();

        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_full = format!("{e:#}");
                    let error_lower = error_full.to_lowercase();

                    // Log this attempt's error with full chain
                    all_errors.push(format!("Attempt {}: {}", attempt + 1, error_full));

                    // Retry on rate limits and transient network errors
                    let is_retryable = error_lower.contains("rate")
                        || error_lower.contains("limit")
                        || error_lower.contains("429")
                        || error_lower.contains("too many")
                        || error_lower.contains("timeout")
                        || error_lower.contains("timed out")
                        || error_lower.contains("connection")
                        || error_lower.contains("temporarily")
                        || error_lower.contains("unavailable")
                        || error_lower.contains("502")
                        || error_lower.contains("503")
                        || error_lower.contains("504");

                    if is_retryable && attempt < self.max_retries {
                        let delay = self.base_delay * 2_u32.pow(attempt);
                        sleep(delay).await;
                    } else {
                        // Return with full context of all attempts
                        if all_errors.len() > 1 {
                            return Err(anyhow!(
                                "{:#}\n\nAll attempts:\n{}",
                                e,
                                all_errors.join("\n")
                            ));
                        }
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!(
            "All {} retries failed:\n{}",
            self.max_retries + 1,
            all_errors.join("\n")
        ))
    }

    pub async fn get_block(&self, number: u64) -> Result<BlockInfo> {
        self.with_retry(|| async {
            let block = self
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(number))
                .await
                .with_context(|| format!("RPC call get_block_by_number({number}) failed"))?
                .ok_or_else(|| anyhow!("Block {number} not found (RPC returned null)"))?;

            let mut info = BlockInfo::from_block(&block);

            // Resolve ENS for miner
            info.miner_ens = self.resolve_ens_name(block.header.beneficiary).await;

            Ok(info)
        })
        .await
        .with_context(|| format!("Failed to fetch block #{number}"))
    }

    pub async fn get_block_tx_hashes(&self, number: u64) -> Result<Vec<String>> {
        self.with_retry(|| async {
            let block = self
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(number))
                .await
                .context("Failed to fetch block")?
                .ok_or_else(|| anyhow!("Block {number} not found"))?;

            let hashes: Vec<String> = block
                .transactions
                .hashes()
                .map(|h| format!("{h:?}"))
                .collect();
            Ok(hashes)
        })
        .await
    }

    /// Get block transactions with full details (for block screen)
    /// Also computes block statistics: total value, fees, blob count
    pub async fn get_block_transactions(
        &self,
        number: u64,
    ) -> Result<(Vec<TxSummary>, BlockStats)> {
        self.with_retry(|| async {
            // Fetch block with full transactions
            let block = self
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(number))
                .full()
                .await
                .with_context(|| format!("RPC call get_block_by_number({number}).full() failed"))?
                .ok_or_else(|| anyhow!("Block {number} not found (RPC returned null)"))?;

            // Collect all unique addresses for ENS resolution
            let mut addresses: Vec<Address> = Vec::new();
            for tx in block.transactions.txns() {
                addresses.push(tx.from());
                if let Some(to) = tx.to() {
                    addresses.push(to);
                }
            }
            // Deduplicate
            addresses.sort();
            addresses.dedup();

            // Batch resolve ENS names
            let ens_names = self.resolve_ens_names(&addresses).await;

            // Try to fetch block receipts for fee calculations
            let receipts = self
                .provider
                .get_block_receipts(BlockId::Number(BlockNumberOrTag::Number(number)))
                .await
                .ok()
                .flatten()
                .unwrap_or_default();

            // Build a map of tx_hash -> receipt
            let receipt_map: HashMap<_, _> =
                receipts.iter().map(|r| (r.transaction_hash, r)).collect();

            // Compute block statistics
            let mut total_value = U256::ZERO;
            let mut total_fees = U256::ZERO;
            let mut blob_count = 0usize;

            for tx in block.transactions.txns() {
                total_value += tx.value();

                // Count blobs
                if let Some(hashes) = TxTrait::blob_versioned_hashes(tx) {
                    blob_count += hashes.len();
                }

                // Sum fees from receipts
                if let Some(receipt) = receipt_map.get(&tx.tx_hash()) {
                    let fee =
                        U256::from(receipt.gas_used) * U256::from(receipt.effective_gas_price);
                    total_fees += fee;
                }
            }

            // Calculate burnt fees (base_fee * gas_used)
            let burnt_fees = if let Some(base_fee) = block.header.base_fee_per_gas {
                U256::from(base_fee) * U256::from(block.header.gas_used)
            } else {
                U256::ZERO
            };

            // Build summaries with ENS names and fee info
            let summaries: Vec<TxSummary> = block
                .transactions
                .txns()
                .map(|tx| {
                    let mut summary = TxSummary::from_tx(tx, &ens_names);
                    // Add fee paid from receipt
                    if let Some(receipt) = receipt_map.get(&tx.tx_hash()) {
                        summary.fee_paid = Some(
                            U256::from(receipt.gas_used) * U256::from(receipt.effective_gas_price),
                        );
                    }
                    summary
                })
                .collect();

            let stats = BlockStats {
                total_value_transferred: total_value,
                total_fees,
                burnt_fees,
                blob_count,
            };

            Ok((summaries, stats))
        })
        .await
        .with_context(|| format!("Failed to fetch transactions for block #{number}"))
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        self.with_retry(|| async {
            self.provider
                .get_block_number()
                .await
                .context("Failed to fetch latest block number")
        })
        .await
    }

    pub async fn get_transaction(&self, hash: TxHash) -> Result<TxInfo> {
        self.with_retry(|| async {
            let tx = self
                .provider
                .get_transaction_by_hash(hash)
                .await
                .with_context(|| format!("RPC call get_transaction_by_hash({hash:?}) failed"))?
                .ok_or_else(|| anyhow!("Transaction {hash:?} not found (RPC returned null)"))?;

            let receipt = self
                .provider
                .get_transaction_receipt(hash)
                .await
                .with_context(|| format!("RPC call get_transaction_receipt({hash:?}) failed"))?;

            let mut info = TxInfo::from_tx_and_receipt(&tx, receipt.as_ref());

            // Resolve ENS names for from and to addresses
            let mut addresses_to_resolve = vec![tx.from()];
            if let Some(to) = tx.to() {
                addresses_to_resolve.push(to);
            }

            let ens_names = self.resolve_ens_names(&addresses_to_resolve).await;
            info.from_ens = ens_names.get(&tx.from()).cloned();
            if let Some(to) = tx.to() {
                info.to_ens = ens_names.get(&to).cloned();
            }

            Ok(info)
        })
        .await
        .with_context(|| format!("Failed to fetch transaction {hash:?}"))
    }

    pub async fn get_address(&self, address: Address) -> Result<AddressInfo> {
        self.with_retry(|| async {
            let balance = self
                .provider
                .get_balance(address)
                .await
                .with_context(|| format!("RPC call get_balance({address:?}) failed"))?;

            let nonce = self
                .provider
                .get_transaction_count(address)
                .await
                .with_context(|| format!("RPC call get_transaction_count({address:?}) failed"))?;

            let code = self
                .provider
                .get_code_at(address)
                .await
                .with_context(|| format!("RPC call get_code_at({address:?}) failed"))?;

            let is_contract = !code.is_empty();
            let code_size = if is_contract { Some(code.len()) } else { None };

            // Check for EIP-1967 proxy implementation slot
            let proxy_impl = if is_contract {
                self.get_proxy_implementation(address).await.ok().flatten()
            } else {
                None
            };

            // Try to detect ERC-20 token info
            let token_info = if is_contract {
                self.detect_erc20(address).await.ok().flatten()
            } else {
                None
            };

            // Resolve ENS name
            let ens_name = self.resolve_ens_name(address).await;

            // Try to read owner() if contract
            let owner = if is_contract {
                self.read_owner(address).await.ok()
            } else {
                None
            };

            // Fetch token balances for popular tokens
            let token_balances = self.get_token_balances(address).await;

            Ok(AddressInfo {
                address,
                balance,
                nonce,
                is_contract,
                code_size,
                proxy_impl,
                token_info,
                ens_name,
                owner,
                token_balances,
            })
        })
        .await
        .with_context(|| format!("Failed to fetch address {address:?}"))
    }

    /// Get EIP-1967 proxy implementation address
    async fn get_proxy_implementation(&self, address: Address) -> Result<Option<Address>> {
        // EIP-1967 implementation slot: keccak256("eip1967.proxy.implementation") - 1
        let impl_slot: U256 = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc"
            .parse()
            .unwrap();

        let storage = self
            .provider
            .get_storage_at(address, impl_slot)
            .await
            .context("Failed to read storage")?;

        // Check if the slot has a non-zero value
        if storage != U256::ZERO {
            // Convert U256 to Address (take last 20 bytes)
            let bytes: [u8; 32] = storage.to_be_bytes();
            let addr_bytes: [u8; 20] = bytes[12..32].try_into().unwrap();
            let impl_addr = Address::from(addr_bytes);
            if impl_addr != Address::ZERO {
                return Ok(Some(impl_addr));
            }
        }

        Ok(None)
    }

    /// Try to detect if address is an ERC-20 token
    async fn detect_erc20(&self, address: Address) -> Result<Option<TokenInfo>> {
        // Try calling name(), symbol(), decimals()
        let name = self.call_string_getter(address, "name()").await.ok();
        let symbol = self.call_string_getter(address, "symbol()").await.ok();
        let decimals = self.call_uint8_getter(address, "decimals()").await.ok();
        let total_supply = self
            .call_uint256_getter(address, "totalSupply()")
            .await
            .ok();

        // If we got at least symbol and decimals, it's likely an ERC-20
        if symbol.is_some() && decimals.is_some() {
            return Ok(Some(TokenInfo {
                name,
                symbol,
                decimals,
                total_supply,
            }));
        }

        Ok(None)
    }

    async fn call_string_getter(&self, address: Address, signature: &str) -> Result<String> {
        use alloy::sol_types::SolValue;

        let selector = &alloy::primitives::keccak256(signature.as_bytes())[..4];
        let input = Bytes::copy_from_slice(selector);

        let result = self
            .provider
            .call(alloy::rpc::types::TransactionRequest {
                to: Some(TxKind::Call(address)),
                input: alloy::rpc::types::TransactionInput::new(input),
                ..Default::default()
            })
            .await
            .context("Call failed")?;

        // Try to decode as string (ABI encoded)
        if result.len() >= 64 {
            let decoded = String::abi_decode(&result).map_err(|e| anyhow!("Decode error: {e}"))?;
            Ok(decoded)
        } else {
            Err(anyhow!("Invalid response length"))
        }
    }

    async fn call_uint8_getter(&self, address: Address, signature: &str) -> Result<u8> {
        let selector = &alloy::primitives::keccak256(signature.as_bytes())[..4];
        let input = Bytes::copy_from_slice(selector);

        let result = self
            .provider
            .call(alloy::rpc::types::TransactionRequest {
                to: Some(TxKind::Call(address)),
                input: alloy::rpc::types::TransactionInput::new(input),
                ..Default::default()
            })
            .await
            .context("Call failed")?;

        if result.len() >= 32 {
            Ok(result[31])
        } else {
            Err(anyhow!("Invalid response length"))
        }
    }

    async fn call_uint256_getter(&self, address: Address, signature: &str) -> Result<U256> {
        let selector = &alloy::primitives::keccak256(signature.as_bytes())[..4];
        let input = Bytes::copy_from_slice(selector);

        let result = self
            .provider
            .call(alloy::rpc::types::TransactionRequest {
                to: Some(TxKind::Call(address)),
                input: alloy::rpc::types::TransactionInput::new(input),
                ..Default::default()
            })
            .await
            .context("Call failed")?;

        if result.len() >= 32 {
            Ok(U256::from_be_slice(&result[..32]))
        } else {
            Err(anyhow!("Invalid response length"))
        }
    }

    /// Try to read owner() from a contract (common in Ownable pattern)
    async fn read_owner(&self, address: Address) -> Result<String> {
        let selector = &keccak256("owner()".as_bytes())[..4];
        let input = Bytes::copy_from_slice(selector);

        let result = self
            .provider
            .call(alloy::rpc::types::TransactionRequest {
                to: Some(TxKind::Call(address)),
                input: alloy::rpc::types::TransactionInput::new(input),
                ..Default::default()
            })
            .await
            .context("owner() call failed")?;

        if result.len() >= 32 {
            let owner_addr = Address::from_slice(&result[12..32]);
            if owner_addr != Address::ZERO {
                Ok(format!("{owner_addr:?}"))
            } else {
                Err(anyhow!("No owner"))
            }
        } else {
            Err(anyhow!("Invalid response"))
        }
    }

    /// Get ERC-20 balances for popular tokens
    /// Returns empty vec on any error to avoid breaking address queries
    async fn get_token_balances(&self, address: Address) -> Vec<TokenBalance> {
        // Wrap in timeout to avoid hanging
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            self.fetch_token_balances_inner(address),
        )
        .await;

        result.unwrap_or_default()
    }

    async fn fetch_token_balances_inner(&self, address: Address) -> Vec<TokenBalance> {
        let mut balances = Vec::new();

        // balanceOf(address) selector
        let selector = &keccak256("balanceOf(address)".as_bytes())[..4];

        for (symbol, name, token_addr, decimals) in POPULAR_TOKENS {
            let Ok(token_address) = token_addr.parse::<Address>() else {
                continue;
            };

            // Build calldata: selector + padded address
            let mut calldata = Vec::with_capacity(36);
            calldata.extend_from_slice(selector);
            calldata.extend_from_slice(&[0u8; 12]); // padding
            calldata.extend_from_slice(address.as_slice());

            let result = self
                .provider
                .call(alloy::rpc::types::TransactionRequest {
                    to: Some(TxKind::Call(token_address)),
                    input: alloy::rpc::types::TransactionInput::new(Bytes::from(calldata)),
                    ..Default::default()
                })
                .await;

            if let Ok(data) = result {
                if data.len() >= 32 {
                    let balance = U256::from_be_slice(&data[..32]);
                    // Filter out tiny balances (< 0.0001 in token units)
                    // For 18 decimals: 0.0001 = 10^14
                    let min_balance = U256::from(10u64).pow(U256::from(decimals.saturating_sub(4)));
                    if balance >= min_balance {
                        balances.push(TokenBalance {
                            symbol: symbol.to_string(),
                            name: name.to_string(),
                            address: token_address,
                            balance,
                            decimals: *decimals,
                        });
                    }
                }
            }
        }

        balances
    }

    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let latest_block = self.get_latest_block_number().await?;

        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .context("Failed to get gas price")?;

        let client_version = self
            .provider
            .get_client_version()
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get fee history for base fee trend (last 5 blocks)
        let fee_history = self
            .provider
            .get_fee_history(5, BlockNumberOrTag::Latest, &[25.0, 50.0, 75.0])
            .await
            .ok();

        let base_fee_trend = fee_history.as_ref().map(|fh| {
            fh.base_fee_per_gas
                .iter()
                .map(|&f| f as u64)
                .collect::<Vec<_>>()
        });

        let priority_fee_percentiles = fee_history.and_then(|fh| {
            fh.reward
                .as_ref()
                .and_then(|rewards| rewards.last().map(|r| r.to_vec()))
        });

        Ok(NetworkInfo {
            latest_block,
            gas_price,
            client_version,
            base_fee_trend,
            priority_fee_percentiles,
        })
    }

    /// Resolve ENS names for a list of addresses
    /// Returns a HashMap of address -> ENS name (only for addresses that have names)
    pub async fn resolve_ens_names(&self, addresses: &[Address]) -> HashMap<Address, String> {
        let mut result = HashMap::new();

        if addresses.is_empty() {
            return result;
        }

        // Build the call data for ReverseRecords.getNames(addresses)
        let call = ReverseRecords::getNamesCall {
            addresses: addresses.to_vec(),
        };

        let tx = TransactionRequest {
            to: Some(TxKind::Call(ENS_REVERSE_RECORDS)),
            input: alloy::rpc::types::TransactionInput::new(call.abi_encode().into()),
            ..Default::default()
        };

        // Make the call
        if let Ok(response) = self.provider.call(tx).await {
            // Decode the response
            if let Ok(names) = ReverseRecords::getNamesCall::abi_decode_returns(&response) {
                for (addr, name) in addresses.iter().zip(names.iter()) {
                    if !name.is_empty() {
                        result.insert(*addr, name.clone());
                    }
                }
            }
        }

        result
    }

    /// Resolve a single ENS name for an address (reverse: address -> name)
    pub async fn resolve_ens_name(&self, address: Address) -> Option<String> {
        self.resolve_ens_names(&[address]).await.remove(&address)
    }

    /// Resolve an ENS name to an address (forward: name -> address)
    pub async fn resolve_ens_to_address(&self, name: &str) -> Result<Address> {
        let node = namehash(name);

        // Step 1: Get the resolver address from the ENS registry
        let registry_call = ENSRegistry::resolverCall { node };
        let registry_tx = TransactionRequest {
            to: Some(TxKind::Call(ENS_REGISTRY)),
            input: alloy::rpc::types::TransactionInput::new(registry_call.abi_encode().into()),
            ..Default::default()
        };

        let response = self
            .provider
            .call(registry_tx)
            .await
            .context("Failed to query ENS registry")?;

        let resolver_addr = ENSRegistry::resolverCall::abi_decode_returns(&response)
            .context("Failed to decode resolver address")?;

        if resolver_addr == Address::ZERO {
            return Err(anyhow!("No resolver found for ENS name: {name}"));
        }

        // Step 2: Query the resolver for the address
        let resolver_call = ENSResolver::addrCall { node };
        let resolver_tx = TransactionRequest {
            to: Some(TxKind::Call(resolver_addr)),
            input: alloy::rpc::types::TransactionInput::new(resolver_call.abi_encode().into()),
            ..Default::default()
        };

        let response = self
            .provider
            .call(resolver_tx)
            .await
            .context("Failed to query ENS resolver")?;

        let resolved_addr = ENSResolver::addrCall::abi_decode_returns(&response)
            .context("Failed to decode resolved address")?;

        if resolved_addr == Address::ZERO {
            return Err(anyhow!("ENS name {name} does not resolve to an address"));
        }

        Ok(resolved_addr)
    }
}
