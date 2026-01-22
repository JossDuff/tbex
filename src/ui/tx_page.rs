use super::helper::*;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{TxResult, MAX_VISIBLE_LOGS, MAX_VISIBLE_TRANSFERS};
use crate::rpc::TxType;
use crate::ui::NAV_HELP_SIMPLE;

pub fn draw_tx_result(frame: &mut Frame, result: &TxResult) {
    let area = frame.area();
    let info = &result.info;

    let chunks = Layout::vertical([
        Constraint::Min(20),   // Tx info
        Constraint::Length(1), // Nav help
    ])
    .split(padded_rect(area, 1));

    let status_str = match info.status {
        Some(true) => "✓ Success",
        Some(false) => "✗ Failed",
        None => "Pending",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" 📄 Transaction ({status_str}) "));

    let mut link_idx = 0;

    let mut lines = vec![
        format_kv("Hash", &info.hash),
        format_kv("Type", info.tx_type.as_str()),
    ];

    // Show decoded method if available
    if let Some(ref method) = info.decoded_method {
        lines.push(format_kv("Method", method));
    }

    // From (link 0) - show ENS name if available
    let from_display = format_address_with_ens(&info.from, info.from_ens.as_deref());
    lines.push(format_kv_link(
        "From",
        &from_display,
        result.selected_link == link_idx,
    ));
    link_idx += 1;

    // To or Contract Creation (link 1 if to exists)
    if let Some(to) = &info.to {
        let to_display = format_address_with_ens(to, info.to_ens.as_deref());
        lines.push(format_kv_link(
            "To",
            &to_display,
            result.selected_link == link_idx,
        ));
        link_idx += 1;
    } else {
        lines.push(format_kv("To", "Contract Creation"));
    }

    lines.push(Line::from(""));
    lines.push(format_kv("Value", &format_eth(info.value)));

    // Actual fee paid
    if let Some(fee) = info.actual_fee {
        lines.push(format_kv("Fee Paid", &format_eth(fee)));
    }

    // Gas info
    match info.tx_type {
        TxType::EIP1559 | TxType::Blob => {
            if let Some(max_fee) = info.max_fee_per_gas {
                lines.push(format_kv("Max Fee", &format_gwei(max_fee)));
            }
            if let Some(priority) = info.max_priority_fee_per_gas {
                lines.push(format_kv("Priority Fee", &format_gwei(priority)));
            }
        }
        _ => {}
    }

    if let Some(gp) = info.gas_price {
        lines.push(format_kv("Gas Price", &format_gwei(gp)));
    }

    lines.push(format_kv("Gas Limit", &format_gas(info.gas_limit)));

    if let Some(used) = info.gas_used {
        lines.push(format_kv(
            "Gas Used",
            &format!(
                "{} ({:.2}%)",
                format_gas(used),
                (used as f64 / info.gas_limit as f64) * 100.0
            ),
        ));
    }

    lines.push(Line::from(""));
    lines.push(format_kv("Nonce", &info.nonce.to_string()));

    // Block (navigable link)
    if let Some(block_num) = info.block_number {
        lines.push(format_kv_link(
            "Block",
            &format!("#{block_num}"),
            result.selected_link == link_idx,
        ));
        link_idx += 1;
    }

    if let Some(idx) = info.tx_index {
        lines.push(format_kv("Tx Index", &idx.to_string()));
    }

    // Contract created (navigable link)
    if let Some(contract) = &info.contract_created {
        lines.push(format_kv_link(
            "Contract Created",
            contract,
            result.selected_link == link_idx,
        ));
        link_idx += 1;
    }

    // Access list
    if let Some(al_size) = info.access_list_size {
        if al_size > 0 {
            lines.push(format_kv("Access List", &format!("{al_size} entries")));
        }
    }

    // Blob info
    if !info.blob_hashes.is_empty() {
        lines.push(Line::from(""));
        lines.push(format_kv("Blob Count", &info.blob_hashes.len().to_string()));
        if let Some(bg) = info.blob_gas_used {
            lines.push(format_kv("Blob Gas Used", &bg.to_string()));
        }
        if let Some(bp) = info.blob_gas_price {
            lines.push(format_kv("Blob Gas Price", &format_gwei(bp)));
        }
    }

    // Input data (truncated)
    lines.push(Line::from(""));
    if info.input_size > 0 {
        let input_hex = format!("{}", info.input_data);
        let display_data = if input_hex.len() > 66 {
            format!(
                "{}...{}",
                &input_hex[..34],
                &input_hex[input_hex.len() - 32..]
            )
        } else {
            input_hex
        };
        lines.push(format_kv("Input", &format!("{} bytes", info.input_size)));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(display_data, Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        lines.push(format_kv("Input", "None (ETH transfer)"));
    }

    // Token transfers - scrollable list with navigable addresses
    if !info.token_transfers.is_empty() {
        lines.push(Line::from(""));

        // Show scroll indicator if scrolled down
        let header_text = if result.transfer_scroll > 0 {
            format!(
                "── Token Transfers ({}) ── ↑{} more",
                info.token_transfers.len(),
                result.transfer_scroll
            )
        } else {
            format!("── Token Transfers ({}) ──", info.token_transfers.len())
        };
        lines.push(Line::from(vec![Span::styled(
            header_text,
            Style::default().fg(Color::Yellow),
        )]));

        // Calculate visible range
        let visible_end =
            (result.transfer_scroll + MAX_VISIBLE_TRANSFERS).min(info.token_transfers.len());
        let visible_transfers = &info.token_transfers[result.transfer_scroll..visible_end];

        // Advance link_idx to skip scrolled-out transfers (3 links per transfer: from, to, token)
        link_idx += result.transfer_scroll * 3;

        for (i, transfer) in visible_transfers.iter().enumerate() {
            let transfer_num = result.transfer_scroll + i + 1; // 1-indexed
            let amount_str = format_token_amount(transfer.amount, transfer.decimals.unwrap_or(18));
            let token_symbol = transfer.token_symbol.as_deref().unwrap_or("Unknown");

            // From address (navigable)
            let from_selected = result.selected_link == link_idx;
            let from_style = if from_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED)
            };
            link_idx += 1;

            // To address (navigable)
            let to_selected = result.selected_link == link_idx;
            let to_style = if to_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED)
            };
            link_idx += 1;

            // Token contract address (navigable)
            let token_selected = result.selected_link == link_idx;
            let token_style = if token_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED)
            };
            link_idx += 1;

            // Line 1: [#] from → to
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {transfer_num:>3}. "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&transfer.from, from_style),
                Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                Span::styled(&transfer.to, to_style),
            ]));
            // Line 2: amount + token address
            lines.push(Line::from(vec![
                Span::styled(
                    format!("       {amount_str} {token_symbol} "),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(&transfer.token_address, token_style),
            ]));
        }

        // Skip remaining link indices for non-visible transfers (3 links per transfer)
        let remaining_transfers = info.token_transfers.len() - visible_end;
        link_idx += remaining_transfers * 3;

        // Show scroll indicator if more below
        if visible_end < info.token_transfers.len() {
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "  ↓ {} more transfers",
                    info.token_transfers.len() - visible_end
                ),
                Style::default().fg(Color::DarkGray),
            )]));
        }
    }

    // Logs - scrollable list with navigable addresses and decoded data
    if !info.logs.is_empty() {
        lines.push(Line::from(""));

        // Show scroll indicator if scrolled down
        let header_text = if result.log_scroll > 0 {
            format!(
                "── Logs ({}) ── ↑{} more",
                info.logs.len(),
                result.log_scroll
            )
        } else {
            format!("── Logs ({}) ──", info.logs.len())
        };
        lines.push(Line::from(vec![Span::styled(
            header_text,
            Style::default().fg(Color::Magenta),
        )]));

        // Calculate visible range
        let visible_end = (result.log_scroll + MAX_VISIBLE_LOGS).min(info.logs.len());
        let visible_logs = &info.logs[result.log_scroll..visible_end];

        // Advance link_idx to skip scrolled-out logs (1 for contract + N for address params)
        for log in &info.logs[..result.log_scroll] {
            link_idx += 1; // contract address
            link_idx += log.decoded_params.iter().filter(|p| p.is_address).count();
        }

        for (i, log) in visible_logs.iter().enumerate() {
            let log_num = result.log_scroll + i + 1; // 1-indexed
            let event_sig = log.event_name.as_deref().unwrap_or("Unknown Event");

            // Log contract address (navigable) on its own line
            let addr_selected = result.selected_link == link_idx;
            let addr_style = if addr_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED)
            };
            link_idx += 1;

            // Line 1: [#] Contract address
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {log_num:>3}. "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&log.address, addr_style),
            ]));

            // Line 2: Event signature
            lines.push(Line::from(vec![Span::styled(
                format!("       {event_sig}"),
                Style::default().fg(Color::White),
            )]));

            // Lines 3+: Decoded parameters (addresses are navigable)
            for param in &log.decoded_params {
                if param.is_address {
                    let param_selected = result.selected_link == link_idx;
                    let param_style = if param_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::UNDERLINED)
                    };
                    link_idx += 1;

                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("         {}: ", param.name),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(&param.value, param_style),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("         {}: ", param.name),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(&param.value, Style::default().fg(Color::Yellow)),
                    ]));
                }
            }
        }

        // Skip remaining link indices for non-visible logs
        for log in &info.logs[visible_end..] {
            link_idx += 1; // contract address
            link_idx += log.decoded_params.iter().filter(|p| p.is_address).count();
        }

        // Show scroll indicator if more below
        if visible_end < info.logs.len() {
            lines.push(Line::from(vec![Span::styled(
                format!("  ↓ {} more logs", info.logs.len() - visible_end),
                Style::default().fg(Color::DarkGray),
            )]));
        }
    }

    // Suppress unused variable warning
    let _ = link_idx;

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, chunks[0]);

    let help = Paragraph::new(NAV_HELP_SIMPLE)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}
