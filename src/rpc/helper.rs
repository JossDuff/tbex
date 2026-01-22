use alloy::primitives::{keccak256, Address, Bytes, B256, U256};

// ============================================================================
// Helper Functions
// ============================================================================

/// Compute the namehash for an ENS name
/// https://docs.ens.domains/contract-api-reference/name-processing#algorithm
pub fn namehash(name: &str) -> B256 {
    let mut node = B256::ZERO;

    if name.is_empty() {
        return node;
    }

    // Split by dots and process in reverse order
    for label in name.rsplit('.') {
        let label_hash = keccak256(label.as_bytes());
        // node = keccak256(node + label_hash)
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(node.as_slice());
        combined[32..].copy_from_slice(label_hash.as_slice());
        node = keccak256(combined);
    }

    node
}

/// Known MEV builder tags
pub fn detect_builder_tag(extra_data: &Bytes, miner: Address) -> Option<String> {
    // Check extra_data for known builder signatures
    if let Ok(s) = String::from_utf8(extra_data.to_vec()) {
        let lower = s.to_lowercase();
        if lower.contains("flashbots") {
            return Some("Flashbots".to_string());
        }
        if lower.contains("bloxroute") || lower.contains("blxr") {
            return Some("bloXroute".to_string());
        }
        if lower.contains("builder0x69") {
            return Some("builder0x69".to_string());
        }
        if lower.contains("titan") {
            return Some("Titan".to_string());
        }
        if lower.contains("rsync") {
            return Some("rsync".to_string());
        }
        if lower.contains("beaver") {
            return Some("Beaver".to_string());
        }
        if lower.contains("buildai") {
            return Some("BuildAI".to_string());
        }
        if lower.contains("penguinbuild") {
            return Some("Penguin".to_string());
        }
        if lower.contains("ethbuilder") {
            return Some("EthBuilder".to_string());
        }
        if lower.contains("blocknative") {
            return Some("Blocknative".to_string());
        }
        // Return decoded extra_data if it looks like a builder name
        if s.len() < 32
            && s.chars()
                .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
        {
            return Some(s);
        }
    }

    // Check known builder addresses
    let miner_str = format!("{miner:?}").to_lowercase();
    let known_builders = [
        ("0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5", "Flashbots"),
        ("0x690b9a9e9aa1c9db991c7721a92d351db4fac990", "builder0x69"),
        ("0x1f9090aae28b8a3dceadf281b0f12828e676c326", "rsync"),
        (
            "0xdafea492d9c6733ae3d56b7ed1adb60692c98bc5",
            "Beacon Depositor",
        ),
    ];

    for (addr, name) in known_builders {
        if miner_str.contains(addr) {
            return Some(name.to_string());
        }
    }

    None
}

/// Known event signatures (topic0)
pub fn decode_event_signature(topic0: &B256) -> Option<&'static str> {
    let bytes = topic0.as_slice();
    match bytes {
        // ERC-20 Transfer
        b if b == keccak256("Transfer(address,address,uint256)").as_slice() => {
            Some("Transfer(address,address,uint256)")
        }
        // ERC-20 Approval
        b if b == keccak256("Approval(address,address,uint256)").as_slice() => {
            Some("Approval(address,address,uint256)")
        }
        // ERC-721 Transfer (same sig as ERC-20 but indexed tokenId)
        // Uniswap V2 Swap
        b if b == keccak256("Swap(address,uint256,uint256,uint256,uint256,address)").as_slice() => {
            Some("Swap(address,uint256,uint256,uint256,uint256,address)")
        }
        // Uniswap V3 Swap
        b if b
            == keccak256("Swap(address,address,int256,int256,uint160,uint128,int24)")
                .as_slice() =>
        {
            Some("Swap(address,address,int256,int256,uint160,uint128,int24)")
        }
        // Deposit (WETH)
        b if b == keccak256("Deposit(address,uint256)").as_slice() => {
            Some("Deposit(address,uint256)")
        }
        // Withdrawal (WETH)
        b if b == keccak256("Withdrawal(address,uint256)").as_slice() => {
            Some("Withdrawal(address,uint256)")
        }
        _ => None,
    }
}

/// Popular ERC-20 tokens on Ethereum mainnet
pub const POPULAR_TOKENS: &[(&str, &str, &str, u8)] = &[
    (
        "USDT",
        "Tether USD",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        6,
    ),
    (
        "USDC",
        "USD Coin",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        6,
    ),
    (
        "WETH",
        "Wrapped Ether",
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        18,
    ),
    (
        "DAI",
        "Dai Stablecoin",
        "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        18,
    ),
    (
        "WBTC",
        "Wrapped Bitcoin",
        "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
        8,
    ),
    (
        "LINK",
        "Chainlink",
        "0x514910771AF9Ca656af840dff83E8264EcF986CA",
        18,
    ),
    (
        "UNI",
        "Uniswap",
        "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984",
        18,
    ),
    (
        "MATIC",
        "Polygon",
        "0x7D1AfA7B718fb893dB30A3aBc0Cfc608AaCfeBB0",
        18,
    ),
    (
        "SHIB",
        "Shiba Inu",
        "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
        18,
    ),
    (
        "stETH",
        "Lido Staked ETH",
        "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
        18,
    ),
];

/// Known function selectors
pub fn decode_function_selector(selector: &[u8]) -> Option<&'static str> {
    if selector.len() < 4 {
        return None;
    }
    let sig = &selector[..4];
    match sig {
        // ERC-20
        [0xa9, 0x05, 0x9c, 0xbb] => Some("transfer(address,uint256)"),
        [0x23, 0xb8, 0x72, 0xdd] => Some("transferFrom(address,address,uint256)"),
        [0x09, 0x5e, 0xa7, 0xb3] => Some("approve(address,uint256)"),
        [0x70, 0xa0, 0x82, 0x31] => Some("balanceOf(address)"),
        [0xdd, 0x62, 0xed, 0x3e] => Some("allowance(address,address)"),
        // ERC-721
        [0x42, 0x84, 0x2e, 0x0e] => Some("safeTransferFrom(address,address,uint256)"),
        [0xb8, 0x8d, 0x4f, 0xde] => Some("safeTransferFrom(address,address,uint256,bytes)"),
        [0x63, 0x52, 0x21, 0x1e] => Some("getApproved(uint256)"),
        [0xa2, 0x2c, 0xb4, 0x65] => Some("setApprovalForAll(address,bool)"),
        // Uniswap V2
        [0x38, 0xed, 0x17, 0x39] => Some("swapExactTokensForTokens"),
        [0x7f, 0xf3, 0x6a, 0xb5] => Some("swapExactETHForTokens"),
        [0x18, 0xcb, 0xaf, 0xe5] => Some("swapExactTokensForETH"),
        [0xfb, 0x3b, 0xdb, 0x41] => Some("swapETHForExactTokens"),
        // Uniswap V3
        [0xc0, 0x4b, 0x8d, 0x59] => Some("exactInput"),
        [0xdb, 0x3e, 0x21, 0x98] => Some("exactInputSingle"),
        [0x09, 0xb8, 0x13, 0x46] => Some("exactOutput"),
        [0x5a, 0xe4, 0x01, 0xdc] => Some("exactOutputSingle"),
        [0xac, 0x96, 0x50, 0xd8] => Some("multicall(uint256,bytes[])"),
        [0x1f, 0x0e, 0x74, 0x08] => Some("multicall(bytes[])"),
        // Common
        [0x39, 0x50, 0x93, 0x51] => Some("deposit"),
        [0x2e, 0x1a, 0x7d, 0x4d] => Some("withdraw(uint256)"),
        [0x3c, 0xcf, 0xd6, 0x0b] => Some("withdraw"),
        [0xd0, 0xe3, 0x0d, 0xb0] => Some("mint"),
        [0xa0, 0x71, 0x2d, 0x68] => Some("burn"),
        [0x01, 0xff, 0xc9, 0xa7] => Some("supportsInterface(bytes4)"),
        // Proxy
        [0x3e, 0x58, 0xc5, 0x8c] => Some("proxy()"),
        [0x5c, 0x60, 0xda, 0x1b] => Some("implementation()"),
        [0xf8, 0x51, 0xa4, 0x40] => Some("admin()"),
        [0x4f, 0x1e, 0xf2, 0x86] => Some("upgradeTo(address)"),
        // Aave
        [0xe8, 0xed, 0xa9, 0xdf] => Some("flashLoan"),
        [0x69, 0x32, 0x8d, 0xec] => Some("supply"),
        [0xa4, 0x15, 0xbc, 0xad] => Some("borrow"),
        [0x57, 0x3e, 0xab, 0x5f] => Some("repay"),
        // ENS
        [0x3b, 0x3b, 0x57, 0xde] => Some("setAddr(bytes32,address)"),
        [0x01, 0xfb, 0xc9, 0x8e] => Some("setName(string)"),
        _ => None,
    }
}

/// Simple hex encoding helper
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Format U256 with decimals for display
pub fn format_u256_decimals(value: U256, decimals: u8) -> String {
    if value == U256::ZERO {
        return "0".to_string();
    }

    let divisor = U256::from(10u64).pow(U256::from(decimals));
    let whole = value / divisor;
    let remainder = value % divisor;

    if remainder == U256::ZERO {
        format!("{whole}")
    } else {
        // Get fractional part as string, pad with zeros
        let frac_str = format!("{remainder}");
        let padded = format!("{:0>width$}", frac_str, width = decimals as usize);
        // Trim trailing zeros
        let trimmed = padded.trim_end_matches('0');
        if trimmed.is_empty() {
            format!("{whole}")
        } else {
            format!("{whole}.{trimmed}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== format_u256_decimals tests ====================

    #[test]
    fn test_format_u256_zero() {
        assert_eq!(format_u256_decimals(U256::ZERO, 18), "0");
        assert_eq!(format_u256_decimals(U256::ZERO, 6), "0");
    }

    #[test]
    fn test_format_u256_whole_number() {
        // 1 ETH = 10^18 wei
        let one_eth = U256::from(10u64).pow(U256::from(18));
        assert_eq!(format_u256_decimals(one_eth, 18), "1");

        // 100 ETH
        let hundred_eth = U256::from(100u64) * U256::from(10u64).pow(U256::from(18));
        assert_eq!(format_u256_decimals(hundred_eth, 18), "100");
    }

    #[test]
    fn test_format_u256_fractional() {
        // 1.5 ETH
        let one_point_five = U256::from(15u64) * U256::from(10u64).pow(U256::from(17));
        assert_eq!(format_u256_decimals(one_point_five, 18), "1.5");

        // 0.123 ETH
        let point_123 = U256::from(123u64) * U256::from(10u64).pow(U256::from(15));
        assert_eq!(format_u256_decimals(point_123, 18), "0.123");
    }

    #[test]
    fn test_format_u256_trailing_zeros_trimmed() {
        // 1.50000 should be "1.5"
        let val = U256::from(150000u64) * U256::from(10u64).pow(U256::from(13));
        assert_eq!(format_u256_decimals(val, 18), "1.5");
    }

    #[test]
    fn test_format_u256_usdc_6_decimals() {
        // 100 USDC (6 decimals)
        let hundred_usdc = U256::from(100_000_000u64);
        assert_eq!(format_u256_decimals(hundred_usdc, 6), "100");

        // 1.50 USDC
        let one_fifty = U256::from(1_500_000u64);
        assert_eq!(format_u256_decimals(one_fifty, 6), "1.5");
    }

    // ==================== hex_encode tests ====================

    #[test]
    fn test_hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_encode_bytes() {
        assert_eq!(hex_encode(&[0x00]), "00");
        assert_eq!(hex_encode(&[0xff]), "ff");
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    // ==================== decode_function_selector tests ====================

    #[test]
    fn test_decode_transfer_selector() {
        // transfer(address,uint256) = 0xa9059cbb
        let selector = [0xa9, 0x05, 0x9c, 0xbb];
        assert_eq!(
            decode_function_selector(&selector),
            Some("transfer(address,uint256)")
        );
    }

    #[test]
    fn test_decode_approve_selector() {
        // approve(address,uint256) = 0x095ea7b3
        let selector = [0x09, 0x5e, 0xa7, 0xb3];
        assert_eq!(
            decode_function_selector(&selector),
            Some("approve(address,uint256)")
        );
    }

    #[test]
    fn test_decode_unknown_selector() {
        let selector = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(decode_function_selector(&selector), None);
    }

    #[test]
    fn test_decode_short_selector() {
        let selector = [0xa9, 0x05, 0x9c]; // Only 3 bytes
        assert_eq!(decode_function_selector(&selector), None);
    }

    // ==================== decode_event_signature tests ====================

    #[test]
    fn test_decode_transfer_event() {
        let sig = keccak256("Transfer(address,address,uint256)");
        assert_eq!(
            decode_event_signature(&sig),
            Some("Transfer(address,address,uint256)")
        );
    }

    #[test]
    fn test_decode_approval_event() {
        let sig = keccak256("Approval(address,address,uint256)");
        assert_eq!(
            decode_event_signature(&sig),
            Some("Approval(address,address,uint256)")
        );
    }

    #[test]
    fn test_decode_swap_event() {
        let sig = keccak256("Swap(address,uint256,uint256,uint256,uint256,address)");
        assert_eq!(
            decode_event_signature(&sig),
            Some("Swap(address,uint256,uint256,uint256,uint256,address)")
        );
    }

    #[test]
    fn test_decode_unknown_event() {
        let sig = keccak256("UnknownEvent(uint256)");
        assert_eq!(decode_event_signature(&sig), None);
    }

    // ==================== detect_builder_tag tests ====================

    #[test]
    fn test_detect_builder_beaverbuild() {
        let extra_data = Bytes::from_static(b"beaverbuild.org");
        let miner = Address::ZERO;
        assert_eq!(
            detect_builder_tag(&extra_data, miner),
            Some("Beaver".to_string())
        );
    }

    #[test]
    fn test_detect_builder_rsync() {
        let extra_data = Bytes::from_static(b"rsync-builder.xyz");
        let miner = Address::ZERO;
        assert_eq!(
            detect_builder_tag(&extra_data, miner),
            Some("rsync".to_string())
        );
    }

    #[test]
    fn test_detect_builder_flashbots() {
        let extra_data = Bytes::from_static(b"Flashbots Builder");
        let miner = Address::ZERO;
        assert_eq!(
            detect_builder_tag(&extra_data, miner),
            Some("Flashbots".to_string())
        );
    }

    #[test]
    fn test_detect_builder_unknown_binary() {
        // Binary data that can't be decoded as UTF-8 string
        let extra_data = Bytes::from_static(&[0xff, 0xfe, 0x00, 0x01]);
        let miner = Address::ZERO;
        assert_eq!(detect_builder_tag(&extra_data, miner), None);
    }

    // ==================== namehash tests ====================

    #[test]
    fn test_namehash_empty() {
        // namehash("") = 0x0000...0000
        let hash = namehash("");
        assert_eq!(hash, B256::ZERO);
    }

    #[test]
    fn test_namehash_eth() {
        // namehash("eth") = known value
        let hash = namehash("eth");
        // This is the known namehash for "eth"
        let expected = "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae";
        assert_eq!(format!("{:?}", hash).to_lowercase(), expected);
    }

    #[test]
    fn test_namehash_vitalik_eth() {
        let hash = namehash("vitalik.eth");
        // Known namehash for vitalik.eth
        let expected = "0xee6c4522aab0003e8d14cd40a6af439055fd2577951148c14b6cea9a53475835";
        assert_eq!(format!("{:?}", hash).to_lowercase(), expected);
    }
}
