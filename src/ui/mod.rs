mod address_page;
mod block_page;
mod helper;
mod tx_page;

use address_page::draw_address_result;
use block_page::draw_block_result;
use helper::*;
use tx_page::draw_tx_result;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Screen};

const TITLE_ART: &str = r#"
████████╗██████╗ ███████╗██╗  ██╗
╚══██╔══╝██╔══██╗██╔════╝╚██╗██╔╝
   ██║   ██████╔╝█████╗   ╚███╔╝ 
   ██║   ██╔══██╗██╔══╝   ██╔██╗ 
   ██║   ██████╔╝███████╗██╔╝ ██╗
   ╚═╝   ╚═════╝ ╚══════╝╚═╝  ╚═╝
"#;

const NAV_HELP: &str = "↑↓ navigate • Enter select • Tab toggle • b back • h home • Esc quit";
const NAV_HELP_SIMPLE: &str = "↑↓ navigate • Enter select • b back • h home • Esc quit";
const NAV_HELP_NO_LIST: &str = "b back • h home • Esc quit";

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::Home => draw_home(frame, app),
        Screen::Loading(msg) => draw_loading(frame, msg),
        Screen::BlockResult(result) => draw_block_result(frame, result),
        Screen::TxResult(result) => draw_tx_result(frame, result),
        Screen::AddressResult(result) => draw_address_result(frame, result),
        Screen::Error(msg) => draw_error(frame, msg),
    }
}

fn draw_home(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if app.needs_rpc_setup() {
        draw_rpc_setup(frame, app, area);
    } else {
        draw_search_home(frame, app, area);
    }
}

fn draw_rpc_setup(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(9), // Title
        Constraint::Length(1), // Subtitle
        Constraint::Length(3), // Spacing
        Constraint::Length(5), // RPC input box
        Constraint::Length(2), // Spacing
        Constraint::Length(1), // Help
        Constraint::Min(0),    // Padding
    ])
    .split(area);

    // Title
    let title = Paragraph::new(TITLE_ART)
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let subtitle = Paragraph::new("Terminal Blockchain Explorer")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[1]);

    // RPC input box
    let rpc_area = centered_rect(70, chunks[3]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" ⚡ RPC Configuration Required ")
        .title_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(rpc_area);
    frame.render_widget(block, rpc_area);

    // Input field inside the box
    let input_chunks = Layout::vertical([
        Constraint::Length(1), // Label
        Constraint::Length(1), // Input
    ])
    .split(inner_area);

    let label = Paragraph::new("Enter your Ethereum RPC URL (e.g., https://eth.llamarpc.com):")
        .style(Style::default().fg(Color::White));
    frame.render_widget(label, input_chunks[0]);

    let inner_width = input_chunks[1].width as usize;
    let scroll = app.rpc_input.visual_scroll(inner_width);

    let display_text = if app.rpc_input.value().is_empty() {
        Span::styled("https://...", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(app.rpc_input.value(), Style::default().fg(Color::White))
    };

    let input = Paragraph::new(display_text).scroll((0, scroll as u16));
    frame.render_widget(input, input_chunks[1]);

    // Cursor
    let cursor_x =
        input_chunks[1].x + (app.rpc_input.visual_cursor().saturating_sub(scroll)) as u16;
    let cursor_y = input_chunks[1].y;
    if cursor_x < input_chunks[1].x + input_chunks[1].width {
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    let help = Paragraph::new("Press Enter to connect • Esc to quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

fn draw_search_home(frame: &mut Frame, app: &App, area: Rect) {
    let recent_searches = app.get_recent_searches();
    let has_history = !recent_searches.is_empty();

    // Calculate history section height (max 5 items + 2 for border)
    let history_height = if has_history {
        (recent_searches.len().min(5) + 2) as u16
    } else {
        0
    };

    let chunks = Layout::vertical([
        Constraint::Length(9),              // Title
        Constraint::Length(1),              // Subtitle
        Constraint::Length(2),              // Spacing
        Constraint::Length(3),              // Search bar
        Constraint::Length(1),              // Spacing
        Constraint::Length(history_height), // History
        Constraint::Length(1),              // Spacing
        Constraint::Length(1),              // RPC status
        Constraint::Length(1),              // Help
        Constraint::Min(0),                 // Network info
    ])
    .split(area);

    // Title
    let title = Paragraph::new(TITLE_ART)
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let subtitle = Paragraph::new("Terminal Blockchain Explorer")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[1]);

    // Search bar
    let search_area = centered_rect(60, chunks[3]);
    let search_selected = app.selected_history_index.is_none();
    draw_search_bar_with_selection(frame, app, search_area, search_selected);

    // History section
    if has_history {
        let history_area = centered_rect(60, chunks[5]);
        draw_history_list(frame, app, history_area);
    }

    // RPC status
    let rpc_status = if let Some(ref url) = app.rpc_url {
        let truncated = if url.len() > 50 {
            format!("{}...", &url[..47])
        } else {
            url.clone()
        };
        Line::from(vec![
            Span::styled("RPC: ", Style::default().fg(Color::DarkGray)),
            Span::styled(truncated, Style::default().fg(Color::Green)),
        ])
    } else {
        Line::from(vec![
            Span::styled("RPC: ", Style::default().fg(Color::DarkGray)),
            Span::styled("Not configured", Style::default().fg(Color::Yellow)),
        ])
    };
    let rpc_widget = Paragraph::new(rpc_status).alignment(Alignment::Center);
    frame.render_widget(rpc_widget, chunks[7]);

    let help_text = if has_history {
        "Enter search • ↑↓ history • Del remove • Esc quit"
    } else {
        "Enter to search • Esc to quit"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[8]);

    // Network info (if available)
    if let Some(info) = &app.network_info {
        let net_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Network Status ");

        let mut lines = vec![Line::from(vec![
            Span::styled("Block: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("#{}", info.latest_block),
                Style::default().fg(Color::White),
            ),
            Span::raw("  "),
            Span::styled("Gas: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_gwei(info.gas_price),
                Style::default().fg(Color::White),
            ),
        ])];

        if let Some(trend) = &info.base_fee_trend {
            if !trend.is_empty() {
                let trend_str = if trend.len() >= 2 {
                    let last = *trend.last().unwrap() as f64;
                    let first = *trend.first().unwrap() as f64;
                    if last > first * 1.1 {
                        "↑"
                    } else if last < first * 0.9 {
                        "↓"
                    } else {
                        "→"
                    }
                } else {
                    ""
                };
                lines[0].spans.push(Span::raw("  "));
                lines[0].spans.push(Span::styled(
                    "Base Fee: ",
                    Style::default().fg(Color::DarkGray),
                ));
                lines[0].spans.push(Span::styled(
                    format!(
                        "{} {}",
                        format_gwei(*trend.last().unwrap() as u128),
                        trend_str
                    ),
                    Style::default().fg(Color::White),
                ));
            }
        }

        lines.push(Line::from(vec![
            Span::styled("Client: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&info.client_version, Style::default().fg(Color::Gray)),
        ]));

        let net_para = Paragraph::new(lines)
            .block(net_block)
            .alignment(Alignment::Center);
        frame.render_widget(net_para, chunks[9]);
    }
}

fn draw_search_bar_with_selection(frame: &mut Frame, app: &App, area: Rect, selected: bool) {
    let border_color = if selected {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" 🔍 Search ")
        .title_style(Style::default().fg(border_color));

    let inner_width = area.width.saturating_sub(2) as usize;
    let scroll = app.search_input.visual_scroll(inner_width);

    let display_text = if app.search_input.value().is_empty() {
        Span::styled(
            "Search by Address / Txn Hash / Block",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::styled(app.search_input.value(), Style::default().fg(Color::White))
    };

    let input = Paragraph::new(display_text)
        .block(block)
        .scroll((0, scroll as u16));

    frame.render_widget(input, area);

    // Only show cursor if search bar is selected
    if selected {
        let cursor_x =
            area.x + 1 + (app.search_input.visual_cursor().saturating_sub(scroll)) as u16;
        let cursor_y = area.y + 1;

        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn draw_history_list(frame: &mut Frame, app: &App, area: Rect) {
    let recent_searches = app.get_recent_searches();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Recent Searches ");

    let items: Vec<ListItem> = recent_searches
        .iter()
        .enumerate()
        .take(5)
        .map(|(i, query)| {
            let is_selected = app.selected_history_index == Some(i);
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Truncate long queries
            let display = if query.len() > 60 {
                format!("{}...", &query[..57])
            } else {
                query.clone()
            };

            ListItem::new(format!(" {display}")).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_loading(frame: &mut Frame, msg: &str) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Loading ");

    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 100) as usize
        % spinner_frames.len();

    let text = format!("{} {}", spinner_frames[idx], msg);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    let centered = centered_rect_fixed(50, 5, area);
    frame.render_widget(paragraph, centered);
}

fn draw_error(frame: &mut Frame, msg: &str) {
    let area = frame.area();
    let padded = padded_rect(area, 1);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" ❌ Error ");

    // Split message into lines and format them
    let mut lines: Vec<Line> = msg
        .lines()
        .map(|line| Line::from(line.to_string()).fg(Color::Red))
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(NAV_HELP_NO_LIST).fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, padded);
}
