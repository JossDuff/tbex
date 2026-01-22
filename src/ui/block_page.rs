use super::helper::*;

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::NAV_HELP;

use crate::app::BlockResult;

pub fn draw_block_result(frame: &mut Frame, result: &BlockResult) {
    let area = frame.area();
    let info = &result.info;
    let padded = padded_rect(area, 1);

    // Calculate block info height (fixed content)
    let block_info_height: u16 = 14; // Base height for block info

    // Calculate transaction list constraints:
    // - Each tx takes 1 line
    // - Plus 2 for borders
    // - Minimum: 3 txs = 5 lines
    // - Maximum: half terminal height
    let min_tx_height: u16 = 5;
    let max_tx_height = padded.height / 2;
    let remaining = padded.height.saturating_sub(block_info_height + 1); // +1 for nav help
    let tx_list_height = remaining.max(min_tx_height).min(max_tx_height);

    let chunks = Layout::vertical([
        Constraint::Length(padded.height.saturating_sub(tx_list_height + 1)), // Block info takes what's left
        Constraint::Length(tx_list_height),                                   // Transaction list
        Constraint::Length(1),                                                // Nav help
    ])
    .split(padded);

    // Block info section
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" 📦 Block #{} ", info.number));

    // Builder tag display
    let miner_display = if let Some(ref tag) = info.builder_tag {
        format!(
            "{} ({})",
            format_address_with_ens(&info.miner, info.miner_ens.as_deref()),
            tag
        )
    } else {
        format_address_with_ens(&info.miner, info.miner_ens.as_deref())
    };

    // Gas usage percentage and bar
    let gas_pct = (info.gas_used as f64 / info.gas_limit as f64) * 100.0;
    let bar_width = 20;
    let filled = ((gas_pct / 100.0) * bar_width as f64) as usize;
    let gas_bar = format!(
        "[{}{}] {:.2}%",
        "█".repeat(filled),
        "░".repeat(bar_width - filled),
        gas_pct
    );

    let mut lines = vec![
        format_kv("Hash", &info.hash),
        format_kv_link(
            "Parent Block",
            &format!("#{}", info.number.saturating_sub(1)),
            !result.list_mode,
        ),
        format_kv("Timestamp", &format_timestamp(info.timestamp)),
        format_kv("Miner/Builder", &miner_display),
        Line::from(""),
    ];

    // Gas section with visual bar
    lines.push(format_kv("Transactions", &info.tx_count.to_string()));
    lines.push(Line::from(vec![
        Span::styled("Gas Used: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format_gas(info.gas_used), Style::default().fg(Color::White)),
        Span::styled(" / ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format_gas(info.gas_limit),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("  {gas_bar}"),
            Style::default().fg(if gas_pct > 90.0 {
                Color::Red
            } else if gas_pct > 70.0 {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
    ]));
    lines.push(format_kv(
        "Base Fee",
        &info
            .base_fee
            .map(|f| format_gwei(f as u128))
            .unwrap_or_else(|| "N/A".to_string()),
    ));

    // Block stats
    lines.push(Line::from(""));
    lines.push(format_kv(
        "Value Transferred",
        &format_eth(result.stats.total_value_transferred),
    ));
    lines.push(format_kv(
        "Total Fees",
        &format_eth(result.stats.total_fees),
    ));
    lines.push(format_kv(
        "Burnt Fees",
        &format_eth(result.stats.burnt_fees),
    ));

    // Blob info
    if result.stats.blob_count > 0 || info.blob_gas_used.is_some() {
        lines.push(Line::from(""));
        lines.push(format_kv("Blobs", &result.stats.blob_count.to_string()));
        if let Some(blob_gas) = info.blob_gas_used {
            lines.push(format_kv("Blob Gas Used", &format_gas(blob_gas)));
        }
    }

    if let Some(size) = info.size {
        lines.push(format_kv("Size", &format!("{size} bytes")));
    }

    if info.uncles_count > 0 {
        lines.push(format_kv("Uncles", &info.uncles_count.to_string()));
    }

    if let Some(wc) = info.withdrawals_count {
        lines.push(format_kv("Withdrawals", &wc.to_string()));
    }

    if let Some(decoded) = &info.extra_data_decoded {
        if info.builder_tag.is_none() {
            lines.push(format_kv("Extra Data", decoded));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, chunks[0]);

    // Transaction list section
    let tx_title = if result.list_mode {
        format!(" Transactions ({}) [selected] ", result.transactions.len())
    } else {
        format!(
            " Transactions ({}) [Tab to select] ",
            result.transactions.len()
        )
    };

    let tx_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if result.list_mode {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(tx_title);

    if result.transactions.is_empty() {
        let empty_msg = Paragraph::new("No transactions in this block")
            .block(tx_block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        // Account for header row and borders
        let visible_count = (chunks[1].height.saturating_sub(3)) as usize; // -2 borders, -1 header
        let start = result.selected_index.saturating_sub(visible_count / 2);

        // Build items: header first, then transactions
        let mut items: Vec<ListItem> = vec![format_tx_list_header()];

        items.extend(
            result
                .transactions
                .iter()
                .enumerate()
                .skip(start)
                .take(visible_count)
                .map(|(i, tx)| {
                    let is_selected = result.list_mode && i == result.selected_index;
                    format_tx_list_item(i, tx, is_selected)
                }),
        );

        let list = List::new(items).block(tx_block);
        frame.render_widget(list, chunks[1]);
    }

    // Navigation help
    let help = Paragraph::new(NAV_HELP)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
