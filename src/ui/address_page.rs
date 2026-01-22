use super::helper::*;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::{NAV_HELP_NO_LIST, NAV_HELP_SIMPLE};

use crate::app::AddressResult;

pub fn draw_address_result(frame: &mut Frame, result: &AddressResult) {
    let area = frame.area();
    let info = &result.info;

    let addr_type = if info.is_contract {
        if info.proxy_impl.is_some() {
            "Proxy Contract"
        } else if info.token_info.is_some() {
            "ERC-20 Token"
        } else {
            "Contract"
        }
    } else {
        "EOA"
    };

    let chunks = Layout::vertical([
        Constraint::Min(10),   // Address info
        Constraint::Length(1), // Nav help
    ])
    .split(padded_rect(area, 1));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" 👤 {addr_type} "));

    let mut lines = vec![];

    // Show ENS name prominently if available
    if let Some(ens) = &info.ens_name {
        lines.push(Line::from(vec![
            Span::styled("ENS: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                ens,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    lines.push(format_kv("Address", &format!("{:?}", info.address)));
    lines.push(Line::from(""));
    lines.push(format_kv("ETH Balance", &format_eth(info.balance)));
    lines.push(format_kv("Nonce", &info.nonce.to_string()));

    if let Some(size) = info.code_size {
        lines.push(format_kv("Code Size", &format!("{size} bytes")));
    }

    // Owner info for contracts
    if let Some(ref owner) = info.owner {
        lines.push(format_kv("Owner", owner));
    }

    // Proxy info
    if let Some(impl_addr) = &info.proxy_impl {
        lines.push(Line::from(""));
        lines.push(format_kv_link(
            "Implementation",
            &format!("{impl_addr:?}"),
            result.selected_link == 0,
        ));
    }

    // Token info (for ERC-20 contracts being viewed)
    if let Some(token) = &info.token_info {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "── Token Contract ──",
            Style::default().fg(Color::DarkGray),
        )]));

        if let Some(name) = &token.name {
            lines.push(format_kv("Name", name));
        }
        if let Some(symbol) = &token.symbol {
            lines.push(format_kv("Symbol", symbol));
        }
        if let Some(decimals) = token.decimals {
            lines.push(format_kv("Decimals", &decimals.to_string()));
        }
        if let Some(supply) = token.total_supply {
            let decimals = token.decimals.unwrap_or(18);
            lines.push(format_kv(
                "Total Supply",
                &format_token_amount(supply, decimals),
            ));
        }
    }

    // Token balances (for any address with token holdings)
    if !info.token_balances.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "── Token Balances ──",
            Style::default().fg(Color::Yellow),
        )]));

        for balance in &info.token_balances {
            let amount = format_token_amount(balance.balance, balance.decimals);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {amount:>12} "),
                    Style::default().fg(Color::White),
                ),
                Span::styled(&balance.symbol, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!(" ({})", balance.name),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, chunks[0]);

    let nav_help = if info.proxy_impl.is_some() {
        NAV_HELP_SIMPLE
    } else {
        NAV_HELP_NO_LIST
    };

    let help = Paragraph::new(nav_help)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}
