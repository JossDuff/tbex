use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::rpc::{TxSummary, TxType};

// ============================================================================
// Helper Functions
// ============================================================================

pub fn truncate_hash(hash: &str) -> String {
    if hash.len() > 20 {
        format!("{}...{}", &hash[..10], &hash[hash.len() - 6..])
    } else {
        hash.to_string()
    }
}

/// Format an address or ENS name to a fixed width (19 chars to match truncated hashes)
pub fn format_addr_fixed_width(addr: &str, ens: Option<&str>) -> String {
    const WIDTH: usize = 19;

    match ens {
        Some(name) => {
            if name.len() > WIDTH {
                // Truncate long ENS names
                format!("{}...", &name[..WIDTH - 3])
            } else {
                // Pad short ENS names
                format!("{name:WIDTH$}")
            }
        }
        None => truncate_hash(addr),
    }
}

pub fn format_tx_list_item<'a>(index: usize, tx: &TxSummary, selected: bool) -> ListItem<'a> {
    // Format addresses to fixed width
    let from_display = format_addr_fixed_width(&tx.from, tx.from_ens.as_deref());
    let to_display = if tx.is_contract_creation {
        format!("{:>19}", "[Contract Create]")
    } else {
        let to_addr = tx.to.as_deref().unwrap_or("?");
        format_addr_fixed_width(to_addr, tx.to_ens.as_deref())
    };

    // Tx type indicator
    let type_indicator = match tx.tx_type {
        TxType::Legacy => "L",
        TxType::AccessList => "A",
        TxType::EIP1559 => "2",
        TxType::Blob => "B",
        TxType::Unknown(_) => "?",
    };

    // Method: prefer decoded name, then selector, then transfer/deploy
    let action = if tx.is_contract_creation {
        "deploy".to_string()
    } else if let Some(ref method) = tx.decoded_method {
        // Truncate long method names
        if method.len() > 10 {
            format!("{}…", &method[..9])
        } else {
            method.clone()
        }
    } else if let Some(ref selector) = tx.method_selector {
        selector.clone()
    } else if tx.input_size == 0 {
        "transfer".to_string()
    } else {
        format!("{}B", tx.input_size)
    };

    let value_str = format_eth(tx.value);
    let fee_str = tx
        .fee_paid
        .map(format_eth)
        .unwrap_or_else(|| "—".to_string());

    // Enhanced format with tx hash, type, addresses, method, value, and fee
    let line = Line::from(vec![
        Span::styled(
            format!("{index:>3} "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(type_indicator, Style::default().fg(Color::DarkGray)),
        Span::styled(" ", Style::default()),
        Span::styled(
            from_display,
            if tx.from_ens.is_some() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Cyan)
            },
        ),
        Span::styled(" → ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            to_display,
            if tx.is_contract_creation {
                Style::default().fg(Color::Magenta)
            } else if tx.to_ens.is_some() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Cyan)
            },
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{action:>10}"), Style::default().fg(Color::Gray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{value_str:>12}"),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{fee_str:>12}"),
            Style::default().fg(Color::Magenta),
        ),
    ]);

    let style = if selected {
        Style::default().bg(Color::Cyan).fg(Color::Black)
    } else {
        Style::default()
    };

    ListItem::new(line).style(style)
}

pub fn format_tx_list_header<'a>() -> ListItem<'a> {
    let line = Line::from(vec![
        Span::styled("    ", Style::default()), // index space
        Span::styled("T", Style::default().fg(Color::DarkGray)), // type
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("{:^19}", "From"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("   ", Style::default()), // arrow space
        Span::styled(
            format!("{:^19}", "To"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:>10}", "Method"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:>12}", "Value"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:>12}", "Fee"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    ListItem::new(line).style(Style::default())
}

pub fn format_kv(key: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key}: "), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

pub fn format_kv_link(key: &str, value: &str, selected: bool) -> Line<'static> {
    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::UNDERLINED)
    };

    Line::from(vec![
        Span::styled(format!("{key}: "), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), style),
    ])
}

/// Format an address with optional ENS name
pub fn format_address_with_ens(address: &str, ens_name: Option<&str>) -> String {
    match ens_name {
        Some(name) => format!("{name} ({address})"),
        None => address.to_string(),
    }
}

pub fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let datetime = UNIX_EPOCH + Duration::from_secs(ts);
    let secs_ago = std::time::SystemTime::now()
        .duration_since(datetime)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if secs_ago < 60 {
        format!("{secs_ago} secs ago")
    } else if secs_ago < 3600 {
        format!("{} mins ago", secs_ago / 60)
    } else if secs_ago < 86400 {
        format!("{} hours ago", secs_ago / 3600)
    } else {
        format!("{} days ago", secs_ago / 86400)
    }
}

pub fn format_gas(gas: u64) -> String {
    if gas >= 1_000_000 {
        format!("{:.2}M", gas as f64 / 1_000_000.0)
    } else if gas >= 1_000 {
        format!("{:.2}K", gas as f64 / 1_000.0)
    } else {
        gas.to_string()
    }
}

pub fn format_gwei(wei: u128) -> String {
    let gwei = wei as f64 / 1_000_000_000.0;
    if gwei >= 1.0 {
        format!("{gwei:.2} gwei")
    } else {
        format!("{gwei:.4} gwei")
    }
}

pub fn format_eth(wei: alloy::primitives::U256) -> String {
    let wei_str = wei.to_string();
    if wei_str.len() <= 18 {
        let eth = wei.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
        format!("{eth:.6} ETH")
    } else {
        let len = wei_str.len();
        let decimal_pos = len - 18;
        let (whole, frac) = wei_str.split_at(decimal_pos);
        format!("{}.{:.6} ETH", whole, &frac[..6.min(frac.len())])
    }
}

pub fn format_token_amount(amount: alloy::primitives::U256, decimals: u8) -> String {
    let amount_str = amount.to_string();
    let dec = decimals as usize;

    if dec == 0 {
        return amount_str;
    }

    if amount_str.len() <= dec {
        let padded = format!("{:0>width$}", amount_str, width = dec + 1);
        let (whole, frac) = padded.split_at(padded.len() - dec);
        let frac_trimmed = frac.trim_end_matches('0');
        if frac_trimmed.is_empty() {
            whole.to_string()
        } else {
            format!("{}.{}", whole, &frac[..4.min(frac.len())])
        }
    } else {
        let split_pos = amount_str.len() - dec;
        let (whole, frac) = amount_str.split_at(split_pos);
        let frac_trimmed = frac.trim_end_matches('0');
        if frac_trimmed.is_empty() {
            whole.to_string()
        } else {
            format!("{}.{}", whole, &frac[..4.min(frac.len())])
        }
    }
}

pub fn centered_rect(percent_x: u16, area: Rect) -> Rect {
    let popup_layout = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(area);

    popup_layout[1]
}

pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Length(height),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .split(area);

    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(vertical[1]);

    horizontal[1]
}

pub fn padded_rect(area: Rect, padding: u16) -> Rect {
    Rect {
        x: area.x + padding,
        y: area.y + padding,
        width: area.width.saturating_sub(padding * 2),
        height: area.height.saturating_sub(padding * 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    // ==================== truncate_hash tests ====================

    #[test]
    fn test_truncate_hash_long() {
        let hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
        let truncated = truncate_hash(hash);
        assert!(truncated.contains("..."));
        assert!(truncated.starts_with("0x5c504ed4"));
        assert!(truncated.ends_with("b22060"));
    }

    #[test]
    fn test_truncate_hash_short() {
        let short = "0x1234";
        assert_eq!(truncate_hash(short), short);
    }

    // ==================== format_gas tests ====================

    #[test]
    fn test_format_gas_small() {
        assert_eq!(format_gas(500), "500");
        assert_eq!(format_gas(21000), "21.00K");
    }

    #[test]
    fn test_format_gas_large() {
        assert_eq!(format_gas(1_000_000), "1.00M");
        assert_eq!(format_gas(30_000_000), "30.00M");
    }

    // ==================== format_gwei tests ====================

    #[test]
    fn test_format_gwei_large() {
        // 50 gwei in wei
        let wei = 50_000_000_000u128;
        assert_eq!(format_gwei(wei), "50.00 gwei");
    }

    #[test]
    fn test_format_gwei_small() {
        // 0.5 gwei in wei
        let wei = 500_000_000u128;
        assert_eq!(format_gwei(wei), "0.5000 gwei");
    }

    // ==================== format_eth tests ====================

    #[test]
    fn test_format_eth_zero() {
        assert_eq!(format_eth(U256::ZERO), "0.000000 ETH");
    }

    #[test]
    fn test_format_eth_one() {
        let one_eth = U256::from(10u64).pow(U256::from(18));
        let formatted = format_eth(one_eth);
        assert!(formatted.starts_with("1."));
        assert!(formatted.ends_with(" ETH"));
    }

    #[test]
    fn test_format_eth_small_fraction() {
        // 0.001 ETH
        let small = U256::from(10u64).pow(U256::from(15));
        let formatted = format_eth(small);
        assert!(formatted.starts_with("0.00"));
        assert!(formatted.ends_with(" ETH"));
    }

    // ==================== format_token_amount tests ====================

    #[test]
    fn test_format_token_amount_whole() {
        // 100 tokens with 18 decimals
        let amount = U256::from(100u64) * U256::from(10u64).pow(U256::from(18));
        let formatted = format_token_amount(amount, 18);
        assert_eq!(formatted, "100");
    }

    #[test]
    fn test_format_token_amount_fractional() {
        // 1.5 tokens with 18 decimals
        let amount = U256::from(15u64) * U256::from(10u64).pow(U256::from(17));
        let formatted = format_token_amount(amount, 18);
        assert!(formatted.starts_with("1.5"));
    }

    #[test]
    fn test_format_token_amount_usdc() {
        // 100 USDC (6 decimals)
        let amount = U256::from(100_000_000u64);
        let formatted = format_token_amount(amount, 6);
        assert_eq!(formatted, "100");
    }

    #[test]
    fn test_format_token_amount_zero_decimals() {
        let amount = U256::from(1000u64);
        let formatted = format_token_amount(amount, 0);
        assert_eq!(formatted, "1000");
    }

    // ==================== format_address_with_ens tests ====================

    #[test]
    fn test_format_address_no_ens() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31";
        assert_eq!(format_address_with_ens(addr, None), addr);
    }

    #[test]
    fn test_format_address_with_ens() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31";
        let result = format_address_with_ens(addr, Some("vitalik.eth"));
        assert!(result.contains("vitalik.eth"));
        assert!(result.contains(addr));
    }

    // ==================== format_addr_fixed_width tests ====================

    #[test]
    fn test_format_addr_fixed_width_no_ens() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31";
        let formatted = format_addr_fixed_width(addr, None);
        // Should truncate to fit width
        assert!(formatted.len() <= 24);
    }

    #[test]
    fn test_format_addr_fixed_width_with_ens() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE31";
        let formatted = format_addr_fixed_width(addr, Some("vitalik.eth"));
        assert!(formatted.contains("vitalik.eth"));
    }

    // ==================== padded_rect tests ====================

    #[test]
    fn test_padded_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let padded = padded_rect(area, 5);
        assert_eq!(padded.x, 5);
        assert_eq!(padded.y, 5);
        assert_eq!(padded.width, 90);
        assert_eq!(padded.height, 40);
    }

    #[test]
    fn test_padded_rect_small_area() {
        let area = Rect::new(0, 0, 10, 10);
        let padded = padded_rect(area, 20); // Padding larger than area
        assert_eq!(padded.width, 0);
        assert_eq!(padded.height, 0);
    }
}
