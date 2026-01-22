/// Represents the type of search query
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchQuery {
    /// Ethereum address (0x + 40 hex chars)
    Address(String),
    /// Transaction hash (0x + 64 hex chars)
    TxHash(String),
    /// Block number (decimal or hex with 0x prefix)
    BlockNumber(u64),
    /// ENS name (contains . and valid characters)
    EnsName(String),
    /// Invalid or unrecognized query
    Invalid(String),
}

impl SearchQuery {
    /// Parse a search string into a typed query
    pub fn parse(input: &str) -> Self {
        let trimmed = input.trim();

        // Check if it looks like an ENS name (contains a dot, ends with known TLD)
        if Self::looks_like_ens(trimmed) {
            return Self::EnsName(trimmed.to_lowercase());
        }

        // Check if it looks like hex
        if let Some(hex_part) = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
        {
            // Validate hex characters
            if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Self::Invalid(format!("Invalid hex characters in: {trimmed}"));
            }

            match hex_part.len() {
                // Address: 40 hex chars
                40 => Self::Address(trimmed.to_lowercase()),
                // Tx hash: 64 hex chars
                64 => Self::TxHash(trimmed.to_lowercase()),
                // Could be a hex block number
                1..=16 => match u64::from_str_radix(hex_part, 16) {
                    Ok(num) => Self::BlockNumber(num),
                    Err(_) => Self::Invalid(format!("Invalid hex number: {trimmed}")),
                },
                _ => Self::Invalid(format!(
                    "Unrecognized format: {} ({} hex chars)",
                    trimmed,
                    hex_part.len()
                )),
            }
        } else if trimmed.chars().all(|c| c.is_ascii_digit()) {
            // Pure decimal - treat as block number
            match trimmed.parse::<u64>() {
                Ok(num) => Self::BlockNumber(num),
                Err(_) => Self::Invalid(format!("Block number too large: {trimmed}")),
            }
        } else {
            Self::Invalid(format!("Unrecognized query format: {trimmed}"))
        }
    }

    /// Check if a string looks like an ENS name
    fn looks_like_ens(s: &str) -> bool {
        // Must contain at least one dot
        if !s.contains('.') {
            return false;
        }

        // Common ENS TLDs
        let ens_tlds = [".eth", ".xyz", ".luxe", ".kred", ".art", ".club"];
        let lower = s.to_lowercase();

        // Check if it ends with a known ENS TLD
        if ens_tlds.iter().any(|tld| lower.ends_with(tld)) {
            // Validate characters (alphanumeric, hyphens, dots)
            return s
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '.');
        }

        false
    }

    /// Returns a human-readable description of the query type
    pub fn description(&self) -> String {
        match self {
            Self::Address(addr) => format!("Address: {addr}"),
            Self::TxHash(hash) => format!("Transaction: {hash}"),
            Self::BlockNumber(num) => format!("Block: {num}"),
            Self::EnsName(name) => format!("ENS: {name}"),
            Self::Invalid(reason) => format!("Invalid: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31";
        assert!(matches!(SearchQuery::parse(addr), SearchQuery::Address(_)));
    }

    #[test]
    fn test_parse_tx_hash() {
        let hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
        assert!(matches!(SearchQuery::parse(hash), SearchQuery::TxHash(_)));
    }

    #[test]
    fn test_parse_block_decimal() {
        assert!(matches!(
            SearchQuery::parse("12345678"),
            SearchQuery::BlockNumber(12345678)
        ));
    }

    #[test]
    fn test_parse_block_hex() {
        assert!(matches!(
            SearchQuery::parse("0xBC614E"),
            SearchQuery::BlockNumber(12345678)
        ));
    }

    #[test]
    fn test_parse_ens_eth() {
        assert!(matches!(
            SearchQuery::parse("vitalik.eth"),
            SearchQuery::EnsName(_)
        ));
        assert!(matches!(
            SearchQuery::parse("nick.eth"),
            SearchQuery::EnsName(_)
        ));
        assert!(matches!(
            SearchQuery::parse("sub.domain.eth"),
            SearchQuery::EnsName(_)
        ));
    }

    #[test]
    fn test_parse_ens_other_tlds() {
        assert!(matches!(
            SearchQuery::parse("test.xyz"),
            SearchQuery::EnsName(_)
        ));
        assert!(matches!(
            SearchQuery::parse("example.art"),
            SearchQuery::EnsName(_)
        ));
    }

    #[test]
    fn test_parse_ens_case_insensitive() {
        if let SearchQuery::EnsName(name) = SearchQuery::parse("VITALIK.ETH") {
            assert_eq!(name, "vitalik.eth");
        } else {
            panic!("Expected EnsName variant");
        }
    }
}
