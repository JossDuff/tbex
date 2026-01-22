use alloy::{
    consensus::{Transaction as TxTrait, Typed2718},
    network::TransactionResponse,
    primitives::{keccak256, Address, Bytes, U256},
};
use std::collections::HashMap;

use super::helper::*;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub base_fee: Option<u64>,
    pub tx_count: usize,
    pub miner: String,
    pub miner_ens: Option<String>,
    pub state_root: String,
    pub receipts_root: String,
    pub transactions_root: String,
    pub extra_data: String,
    pub extra_data_decoded: Option<String>,
    pub size: Option<u64>,
    pub uncles_count: usize,
    pub withdrawals_count: Option<usize>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    // New computed fields
    pub blob_count: usize,
    pub total_value_transferred: U256,
    pub total_fees: U256,
    pub burnt_fees: U256,
    pub builder_tag: Option<String>,
}

impl BlockInfo {
    pub fn from_block(block: &alloy::rpc::types::Block) -> Self {
        let extra_data = format!("{}", block.header.extra_data);
        let extra_data_decoded = Self::try_decode_extra_data(&block.header.extra_data);
        let builder_tag = detect_builder_tag(&block.header.extra_data, block.header.beneficiary);

        Self {
            number: block.header.number,
            hash: format!("{:?}", block.header.hash),
            parent_hash: format!("{:?}", block.header.parent_hash),
            timestamp: block.header.timestamp,
            gas_used: block.header.gas_used,
            gas_limit: block.header.gas_limit,
            base_fee: block.header.base_fee_per_gas,
            tx_count: block.transactions.len(),
            miner: format!("{:?}", block.header.beneficiary),
            miner_ens: None,
            state_root: format!("{:?}", block.header.state_root),
            receipts_root: format!("{:?}", block.header.receipts_root),
            transactions_root: format!("{:?}", block.header.transactions_root),
            extra_data,
            extra_data_decoded,
            size: block.header.size.and_then(|s| s.try_into().ok()),
            uncles_count: block.uncles.len(),
            withdrawals_count: block.withdrawals.as_ref().map(|w| w.len()),
            blob_gas_used: block.header.blob_gas_used,
            excess_blob_gas: block.header.excess_blob_gas,
            // These will be computed from transactions
            blob_count: 0,
            total_value_transferred: U256::ZERO,
            total_fees: U256::ZERO,
            burnt_fees: U256::ZERO,
            builder_tag,
        }
    }

    fn try_decode_extra_data(data: &Bytes) -> Option<String> {
        if data.is_empty() {
            return None;
        }
        // Try to decode as UTF-8 string (often contains client version)
        String::from_utf8(data.to_vec())
            .ok()
            .filter(|s| s.chars().all(|c| c.is_ascii_graphic() || c == ' '))
    }
}

/// Decoded log/event
#[derive(Debug, Clone)]
pub struct DecodedLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub event_name: Option<String>, // Full signature like "Transfer(address,address,uint256)"
    pub decoded_params: Vec<DecodedParam>, // Individual decoded parameters
}

/// A decoded event parameter
#[derive(Debug, Clone)]
pub struct DecodedParam {
    pub name: String,     // Parameter name like "from", "to", "value"
    pub value: String,    // Decoded value
    pub is_address: bool, // Whether this is a navigable address
}

/// Token transfer extracted from logs
#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub token_address: String,
    pub from: String,
    pub to: String,
    pub amount: U256,
    pub token_symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct TxInfo {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: U256,
    pub gas_price: Option<u128>,
    pub gas_limit: u64,
    pub gas_used: Option<u64>,
    pub nonce: u64,
    pub block_number: Option<u64>,
    pub status: Option<bool>,
    pub input_size: usize,
    pub tx_type: TxType,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub tx_index: Option<u64>,
    pub contract_created: Option<String>,
    pub logs_count: Option<usize>,
    pub access_list_size: Option<usize>,
    pub blob_gas_used: Option<u64>,
    pub blob_gas_price: Option<u128>,
    pub blob_hashes: Vec<String>,
    pub input_data: Bytes,
    // ENS names
    pub from_ens: Option<String>,
    pub to_ens: Option<String>,
    // New computed fields
    pub actual_fee: Option<U256>,
    pub decoded_method: Option<String>,
    pub logs: Vec<DecodedLog>,
    pub token_transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxType {
    Legacy,     // Type 0
    AccessList, // Type 1 (EIP-2930)
    EIP1559,    // Type 2 (EIP-1559)
    Blob,       // Type 3 (EIP-4844)
    Unknown(u8),
}

impl TxType {
    pub fn from_type_byte(ty: u8) -> Self {
        match ty {
            0 => TxType::Legacy,
            1 => TxType::AccessList,
            2 => TxType::EIP1559,
            3 => TxType::Blob,
            n => TxType::Unknown(n),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TxType::Legacy => "Legacy (Type 0)",
            TxType::AccessList => "Access List (Type 1)",
            TxType::EIP1559 => "EIP-1559 (Type 2)",
            TxType::Blob => "Blob (Type 3)",
            TxType::Unknown(_) => "Unknown",
        }
    }
}

impl TxInfo {
    pub fn from_tx_and_receipt(
        tx: &alloy::rpc::types::Transaction,
        receipt: Option<&alloy::rpc::types::TransactionReceipt>,
    ) -> Self {
        let tx_type = TxType::from_type_byte(tx.ty());

        let access_list_size = TxTrait::access_list(tx).map(|al| al.len());

        let blob_hashes: Vec<String> = TxTrait::blob_versioned_hashes(tx)
            .map(|hashes| hashes.iter().map(|h| format!("{h:?}")).collect())
            .unwrap_or_default();

        // Compute actual fee paid
        let actual_fee = receipt.map(|r| {
            let gas_used = r.gas_used;
            let effective_price = r.effective_gas_price;
            U256::from(gas_used) * U256::from(effective_price)
        });

        // Decode method selector
        let decoded_method = if tx.input().len() >= 4 {
            decode_function_selector(tx.input()).map(String::from)
        } else {
            None
        };

        // Process logs
        let (logs, token_transfers) = if let Some(r) = receipt {
            let mut decoded_logs = Vec::new();
            let mut transfers = Vec::new();

            // Known event signatures
            let transfer_sig = keccak256("Transfer(address,address,uint256)");
            let approval_sig = keccak256("Approval(address,address,uint256)");
            let swap_v2_sig = keccak256("Swap(address,uint256,uint256,uint256,uint256,address)");
            let deposit_sig = keccak256("Deposit(address,uint256)");
            let withdrawal_sig = keccak256("Withdrawal(address,uint256)");

            for log in r.inner.logs() {
                let topics: Vec<String> = log.topics().iter().map(|t| format!("{t:?}")).collect();

                let event_name = log
                    .topics()
                    .first()
                    .and_then(|t| decode_event_signature(t))
                    .map(String::from);

                let mut decoded_params: Vec<DecodedParam> = Vec::new();

                // Decode data based on event type
                if let Some(topic0) = log.topics().first() {
                    if topic0 == &transfer_sig && log.topics().len() >= 3 {
                        // ERC-20 Transfer: from and to in topics, amount in data
                        let from = format!("0x{}", hex_encode(&log.topics()[1].as_slice()[12..]));
                        let to = format!("0x{}", hex_encode(&log.topics()[2].as_slice()[12..]));
                        let amount = if log.data().data.len() >= 32 {
                            U256::from_be_slice(&log.data().data[..32])
                        } else {
                            U256::ZERO
                        };

                        transfers.push(TokenTransfer {
                            token_address: format!("{:?}", log.address()),
                            from: from.clone(),
                            to: to.clone(),
                            amount,
                            token_symbol: None,
                            decimals: None,
                        });

                        decoded_params.push(DecodedParam {
                            name: "from".to_string(),
                            value: from,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "to".to_string(),
                            value: to,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "value".to_string(),
                            value: format_u256_decimals(amount, 18),
                            is_address: false,
                        });
                    } else if topic0 == &approval_sig && log.topics().len() >= 3 {
                        // Approval: owner and spender in topics, amount in data
                        let owner = format!("0x{}", hex_encode(&log.topics()[1].as_slice()[12..]));
                        let spender =
                            format!("0x{}", hex_encode(&log.topics()[2].as_slice()[12..]));
                        let amount = if log.data().data.len() >= 32 {
                            U256::from_be_slice(&log.data().data[..32])
                        } else {
                            U256::ZERO
                        };

                        let amount_display = if amount == U256::MAX {
                            "unlimited".to_string()
                        } else {
                            format_u256_decimals(amount, 18)
                        };

                        decoded_params.push(DecodedParam {
                            name: "owner".to_string(),
                            value: owner,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "spender".to_string(),
                            value: spender,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "value".to_string(),
                            value: amount_display,
                            is_address: false,
                        });
                    } else if topic0 == &swap_v2_sig && log.topics().len() >= 2 {
                        // Uniswap V2 Swap: sender in topic, amounts in data, to in data
                        let sender = format!("0x{}", hex_encode(&log.topics()[1].as_slice()[12..]));
                        decoded_params.push(DecodedParam {
                            name: "sender".to_string(),
                            value: sender,
                            is_address: true,
                        });

                        if log.data().data.len() >= 128 {
                            let amount0_in = U256::from_be_slice(&log.data().data[0..32]);
                            let amount1_in = U256::from_be_slice(&log.data().data[32..64]);
                            let amount0_out = U256::from_be_slice(&log.data().data[64..96]);
                            let amount1_out = U256::from_be_slice(&log.data().data[96..128]);

                            decoded_params.push(DecodedParam {
                                name: "amount0In".to_string(),
                                value: format_u256_decimals(amount0_in, 18),
                                is_address: false,
                            });
                            decoded_params.push(DecodedParam {
                                name: "amount1In".to_string(),
                                value: format_u256_decimals(amount1_in, 18),
                                is_address: false,
                            });
                            decoded_params.push(DecodedParam {
                                name: "amount0Out".to_string(),
                                value: format_u256_decimals(amount0_out, 18),
                                is_address: false,
                            });
                            decoded_params.push(DecodedParam {
                                name: "amount1Out".to_string(),
                                value: format_u256_decimals(amount1_out, 18),
                                is_address: false,
                            });
                        }
                        if log.data().data.len() >= 160 {
                            let to = format!("0x{}", hex_encode(&log.data().data[140..160]));
                            decoded_params.push(DecodedParam {
                                name: "to".to_string(),
                                value: to,
                                is_address: true,
                            });
                        }
                    } else if topic0 == &deposit_sig && log.topics().len() >= 2 {
                        // WETH Deposit
                        let dst = format!("0x{}", hex_encode(&log.topics()[1].as_slice()[12..]));
                        let amount = if log.data().data.len() >= 32 {
                            U256::from_be_slice(&log.data().data[..32])
                        } else {
                            U256::ZERO
                        };
                        decoded_params.push(DecodedParam {
                            name: "dst".to_string(),
                            value: dst,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "wad".to_string(),
                            value: format_u256_decimals(amount, 18),
                            is_address: false,
                        });
                    } else if topic0 == &withdrawal_sig && log.topics().len() >= 2 {
                        // WETH Withdrawal
                        let src = format!("0x{}", hex_encode(&log.topics()[1].as_slice()[12..]));
                        let amount = if log.data().data.len() >= 32 {
                            U256::from_be_slice(&log.data().data[..32])
                        } else {
                            U256::ZERO
                        };
                        decoded_params.push(DecodedParam {
                            name: "src".to_string(),
                            value: src,
                            is_address: true,
                        });
                        decoded_params.push(DecodedParam {
                            name: "wad".to_string(),
                            value: format_u256_decimals(amount, 18),
                            is_address: false,
                        });
                    } else {
                        // Generic: decode topics and data as best we can
                        // Topics after topic0 are indexed params (addresses are 32 bytes, last 20 are the address)
                        for (i, topic) in log.topics().iter().enumerate().skip(1) {
                            // Try to interpret as address (last 20 bytes)
                            let bytes = topic.as_slice();
                            if bytes[..12].iter().all(|&b| b == 0) {
                                // Looks like an address
                                decoded_params.push(DecodedParam {
                                    name: format!("topic{i}"),
                                    value: format!("0x{}", hex_encode(&bytes[12..])),
                                    is_address: true,
                                });
                            } else {
                                // Treat as uint256
                                let val = U256::from_be_slice(bytes);
                                decoded_params.push(DecodedParam {
                                    name: format!("topic{i}"),
                                    value: format!("{val}"),
                                    is_address: false,
                                });
                            }
                        }
                        // Decode data as uint256 chunks
                        let data = &log.data().data;
                        let num_chunks = data.len() / 32;
                        for i in 0..num_chunks.min(4) {
                            // Max 4 data params
                            let start = i * 32;
                            let end = start + 32;
                            if end <= data.len() {
                                let val = U256::from_be_slice(&data[start..end]);
                                decoded_params.push(DecodedParam {
                                    name: format!("data{i}"),
                                    value: format_u256_decimals(val, 18),
                                    is_address: false,
                                });
                            }
                        }
                    }
                }

                decoded_logs.push(DecodedLog {
                    address: format!("{:?}", log.address()),
                    topics,
                    data: format!("0x{}", hex_encode(log.data().data.as_ref())),
                    event_name,
                    decoded_params,
                });
            }

            (decoded_logs, transfers)
        } else {
            (Vec::new(), Vec::new())
        };

        Self {
            hash: format!("{:?}", tx.tx_hash()),
            from: format!("{:?}", tx.from()),
            to: tx.to().map(|a| format!("{a:?}")),
            value: tx.value(),
            gas_price: <_ as TransactionResponse>::gas_price(tx),
            gas_limit: tx.gas_limit(),
            gas_used: receipt.map(|r| r.gas_used),
            nonce: tx.nonce(),
            block_number: tx.block_number(),
            status: receipt.map(|r| r.status()),
            input_size: tx.input().len(),
            tx_type,
            max_fee_per_gas: <_ as TransactionResponse>::max_fee_per_gas(tx),
            max_priority_fee_per_gas: TxTrait::max_priority_fee_per_gas(tx),
            tx_index: tx.transaction_index(),
            contract_created: receipt
                .and_then(|r| r.contract_address)
                .map(|a| format!("{a:?}")),
            logs_count: receipt.map(|r| r.inner.logs().len()),
            access_list_size,
            blob_gas_used: receipt.and_then(|r| r.blob_gas_used),
            blob_gas_price: receipt.and_then(|r| r.blob_gas_price),
            blob_hashes,
            input_data: tx.input().clone(),
            from_ens: None,
            to_ens: None,
            actual_fee,
            decoded_method,
            logs,
            token_transfers,
        }
    }
}

/// Lightweight transaction summary for block list view
#[derive(Debug, Clone)]
pub struct TxSummary {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: U256,
    pub gas_limit: u64,
    pub tx_type: TxType,
    pub is_contract_creation: bool,
    pub from_ens: Option<String>,
    pub to_ens: Option<String>,
    pub input_size: usize,
    pub method_selector: Option<String>,
    pub decoded_method: Option<String>,
    pub blob_count: usize,
    pub fee_paid: Option<U256>,
}

impl TxSummary {
    pub fn from_tx(
        tx: &alloy::rpc::types::Transaction,
        ens_names: &HashMap<Address, String>,
    ) -> Self {
        let from_addr = tx.from();
        let to_addr = tx.to();
        let input = tx.input();

        // Extract method selector (first 4 bytes) if this is a contract call
        let method_selector = if input.len() >= 4 && to_addr.is_some() {
            Some(format!(
                "0x{:02x}{:02x}{:02x}{:02x}",
                input[0], input[1], input[2], input[3]
            ))
        } else {
            None
        };

        // Decode method name
        let decoded_method = if input.len() >= 4 {
            decode_function_selector(input).map(|s| {
                // Extract just the function name
                s.split('(').next().unwrap_or(s).to_string()
            })
        } else {
            None
        };

        let blob_count = TxTrait::blob_versioned_hashes(tx)
            .map(|h| h.len())
            .unwrap_or(0);

        Self {
            hash: format!("{:?}", tx.tx_hash()),
            from: format!("{from_addr:?}"),
            to: to_addr.map(|a| format!("{a:?}")),
            value: tx.value(),
            gas_limit: tx.gas_limit(),
            tx_type: TxType::from_type_byte(tx.ty()),
            is_contract_creation: to_addr.is_none(),
            from_ens: ens_names.get(&from_addr).cloned(),
            to_ens: to_addr.and_then(|a| ens_names.get(&a).cloned()),
            input_size: input.len(),
            method_selector,
            decoded_method,
            blob_count,
            fee_paid: None, // Will be set from receipt
        }
    }
}

/// Block-level statistics computed from transactions
#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub total_value_transferred: U256,
    pub total_fees: U256,
    pub burnt_fees: U256,
    pub blob_count: usize,
}

/// Token balance for a specific token
#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub symbol: String,
    pub name: String,
    pub address: Address,
    pub balance: U256,
    pub decimals: u8,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: Address,
    pub balance: U256,
    pub nonce: u64,
    pub is_contract: bool,
    pub code_size: Option<usize>,
    pub proxy_impl: Option<Address>,
    pub token_info: Option<TokenInfo>,
    pub ens_name: Option<String>,
    pub owner: Option<String>,
    pub token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub total_supply: Option<U256>,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub latest_block: u64,
    pub gas_price: u128,
    pub client_version: String,
    pub base_fee_trend: Option<Vec<u64>>,
    pub priority_fee_percentiles: Option<Vec<u128>>, // 25th, 50th, 75th percentile
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== DecodedParam tests ====================

    #[test]
    fn test_decoded_param_address() {
        let param = DecodedParam {
            name: "from".to_string(),
            value: "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31".to_string(),
            is_address: true,
        };
        assert!(param.is_address);
        assert_eq!(param.name, "from");
    }

    #[test]
    fn test_decoded_param_value() {
        let param = DecodedParam {
            name: "amount".to_string(),
            value: "1000.5".to_string(),
            is_address: false,
        };
        assert!(!param.is_address);
    }

    // ==================== TxType tests ====================

    #[test]
    fn test_tx_type_from_type_byte() {
        assert!(matches!(TxType::from_type_byte(0), TxType::Legacy));
        assert!(matches!(TxType::from_type_byte(1), TxType::AccessList));
        assert!(matches!(TxType::from_type_byte(2), TxType::EIP1559));
        assert!(matches!(TxType::from_type_byte(3), TxType::Blob));
        assert!(matches!(TxType::from_type_byte(99), TxType::Unknown(99)));
    }

    #[test]
    fn test_tx_type_as_str() {
        assert_eq!(TxType::Legacy.as_str(), "Legacy (Type 0)");
        assert_eq!(TxType::AccessList.as_str(), "Access List (Type 1)");
        assert_eq!(TxType::EIP1559.as_str(), "EIP-1559 (Type 2)");
        assert_eq!(TxType::Blob.as_str(), "Blob (Type 3)");
    }
}
