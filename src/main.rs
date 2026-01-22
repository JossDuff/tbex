use tbex::app::{App, NavLink};
use tbex::config::Config;
use tbex::rpc::{AddressInfo, BlockInfo, BlockStats, NetworkInfo, RpcClient, TxInfo, TxSummary};
use tbex::search::SearchQuery;
use tbex::ui;

use alloy::primitives::{Address, TxHash};
use anyhow::Result;
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
};
use std::io::stdout;
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    run_tui(config).await?;

    Ok(())
}

/// Messages from async tasks back to the main loop
enum AsyncMessage {
    BlockResult(Result<(BlockInfo, Vec<TxSummary>, BlockStats)>),
    TxResult(Result<TxInfo>),
    AddressResult(Result<AddressInfo>),
    NetworkInfo(Result<NetworkInfo>),
}

async fn run_tui(config: Config) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new(config);

    let (tx, mut rx) = mpsc::channel::<AsyncMessage>(10);

    // Fetch network info on startup (only if RPC is configured)
    if app.has_rpc() {
        if let Some(ref url) = app.rpc_url {
            let tx_clone = tx.clone();
            let url_clone = url.clone();
            tokio::spawn(async move {
                if let Ok(client) = RpcClient::new(&url_clone) {
                    let result = client.get_network_info().await;
                    let _ = tx_clone.send(AsyncMessage::NetworkInfo(result)).await;
                }
            });
        }
    }

    let result = run_event_loop(&mut terminal, &mut app, tx, &mut rx).await;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    tx: mpsc::Sender<AsyncMessage>,
    rx: &mut mpsc::Receiver<AsyncMessage>,
) -> Result<()> {
    let mut last_network_refresh = std::time::Instant::now();

    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Periodically refresh network info (every 12 seconds ~ 1 block)
        if app.is_on_home()
            && app.has_rpc()
            && last_network_refresh.elapsed() > std::time::Duration::from_secs(12)
        {
            last_network_refresh = std::time::Instant::now();
            if let Some(ref url) = app.rpc_url {
                let tx_clone = tx.clone();
                let url_clone = url.clone();
                tokio::spawn(async move {
                    if let Ok(client) = RpcClient::new(&url_clone) {
                        let result = client.get_network_info().await;
                        let _ = tx_clone.send(AsyncMessage::NetworkInfo(result)).await;
                    }
                });
            }
        }

        // Check for async results
        while let Ok(msg) = rx.try_recv() {
            match msg {
                AsyncMessage::BlockResult(Ok((info, transactions, stats))) => {
                    app.set_block_result(info, transactions, stats);
                }
                AsyncMessage::TxResult(Ok(info)) => app.set_tx_result(info),
                AsyncMessage::AddressResult(Ok(info)) => app.set_address_result(info),
                AsyncMessage::NetworkInfo(Ok(info)) => app.set_network_info(info),
                AsyncMessage::BlockResult(Err(e))
                | AsyncMessage::TxResult(Err(e))
                | AsyncMessage::AddressResult(Err(e)) => {
                    // Use {:#} to get full error chain from anyhow
                    app.set_error(format!("{e:#}"));
                }
                AsyncMessage::NetworkInfo(Err(_)) => {
                    // Silently ignore network info errors
                }
            }
        }

        // Poll for input events
        if event::poll(std::time::Duration::from_millis(50))? {
            let ev = event::read()?;

            if let Event::Key(key) = &ev {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Global keys
                match key.code {
                    KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    _ => {}
                }

                // Screen-specific keys
                if app.is_on_home() {
                    if app.needs_rpc_setup() {
                        // RPC setup mode
                        match key.code {
                            KeyCode::Enter => {
                                match app.submit_rpc() {
                                    Ok(()) => {
                                        // RPC configured, fetch network info
                                        if let Some(ref url) = app.rpc_url {
                                            let tx_clone = tx.clone();
                                            let url_clone = url.clone();
                                            tokio::spawn(async move {
                                                if let Ok(client) = RpcClient::new(&url_clone) {
                                                    let result = client.get_network_info().await;
                                                    let _ = tx_clone
                                                        .send(AsyncMessage::NetworkInfo(result))
                                                        .await;
                                                }
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        app.set_error(e);
                                    }
                                }
                            }
                            KeyCode::Esc => {}
                            _ => {
                                app.rpc_input.handle_event(&ev);
                            }
                        }
                    } else {
                        // Normal search mode with history
                        match key.code {
                            KeyCode::Enter => {
                                // Check if a history item is selected
                                if let Some(query) = app.get_selected_history_query() {
                                    app.clear_history_selection();
                                    // Add to history again to move it to top
                                    let _ = app.config.add_recent_search(query.clone());
                                    execute_search(app, &query, tx.clone());
                                } else if let Some(query) = app.submit_search() {
                                    execute_search(app, &query, tx.clone());
                                }
                            }
                            KeyCode::Up => {
                                app.select_history_prev();
                            }
                            KeyCode::Down => {
                                app.select_history_next();
                            }
                            KeyCode::Delete | KeyCode::Backspace
                                if app.selected_history_index.is_some() =>
                            {
                                app.delete_selected_history();
                            }
                            KeyCode::Esc => {}
                            _ => {
                                // Only handle text input when history not selected
                                if app.selected_history_index.is_none() {
                                    app.search_input.handle_event(&ev);
                                } else {
                                    // Any other key clears history selection and goes to search
                                    app.clear_history_selection();
                                    app.search_input.handle_event(&ev);
                                }
                            }
                        }
                    }
                } else if !app.is_loading() {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.select_prev();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.select_next();
                        }
                        KeyCode::Tab => {
                            app.toggle_mode();
                        }
                        KeyCode::Enter => {
                            if let Some(link) = app.get_selected_link() {
                                navigate_to_link(app, link, tx.clone());
                            }
                        }
                        KeyCode::Backspace | KeyCode::Char('b') => {
                            app.go_back();
                        }
                        KeyCode::Char('h') => {
                            app.go_home();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn navigate_to_link(app: &mut App, link: NavLink, tx: mpsc::Sender<AsyncMessage>) {
    match link {
        NavLink::Address(addr) => {
            execute_search(app, &addr, tx);
        }
        NavLink::Block(num) => {
            execute_search(app, &num.to_string(), tx);
        }
        NavLink::Transaction(hash) => {
            execute_search(app, &hash, tx);
        }
    }
}

fn execute_search(app: &mut App, query: &str, tx: mpsc::Sender<AsyncMessage>) {
    let parsed = SearchQuery::parse(query);

    if let SearchQuery::Invalid(reason) = parsed {
        app.set_error(reason);
        return;
    }

    let Some(_) = &app.rpc_client else {
        app.set_error("No RPC configured. Use 'tbex set-rpc <url>' first.".into());
        return;
    };

    let rpc_url = app.rpc_url.clone().unwrap();

    match parsed {
        SearchQuery::BlockNumber(num) => {
            app.set_loading(&format!("Fetching block {num}..."));
            let tx = tx.clone();
            let rpc_url_for_error = rpc_url.clone();
            tokio::spawn(async move {
                let client = RpcClient::new(&rpc_url).unwrap();
                let result = async {
                    let info = client.get_block(num).await?;
                    let (transactions, stats) = client.get_block_transactions(num).await?;
                    Ok((info, transactions, stats))
                }
                .await
                .map_err(|e: anyhow::Error| {
                    anyhow::anyhow!("{e:#}\n\nRPC: {rpc_url_for_error}")
                });
                let _ = tx.send(AsyncMessage::BlockResult(result)).await;
            });
        }
        SearchQuery::TxHash(hash) => {
            app.set_loading("Fetching transaction...");
            let tx = tx.clone();
            let rpc_url_for_error = rpc_url.clone();
            tokio::spawn(async move {
                let client = RpcClient::new(&rpc_url).unwrap();
                let result = async {
                    let hash: TxHash = hash.parse()?;
                    client.get_transaction(hash).await
                }
                .await
                .map_err(|e: anyhow::Error| {
                    anyhow::anyhow!("{e:#}\n\nRPC: {rpc_url_for_error}")
                });
                let _ = tx.send(AsyncMessage::TxResult(result)).await;
            });
        }
        SearchQuery::Address(addr) => {
            app.set_loading("Fetching address...");
            let tx = tx.clone();
            let rpc_url_for_error = rpc_url.clone();
            tokio::spawn(async move {
                let client = RpcClient::new(&rpc_url).unwrap();
                let result = async {
                    let addr: Address = addr.parse()?;
                    client.get_address(addr).await
                }
                .await
                .map_err(|e: anyhow::Error| {
                    anyhow::anyhow!("{e:#}\n\nRPC: {rpc_url_for_error}")
                });
                let _ = tx.send(AsyncMessage::AddressResult(result)).await;
            });
        }
        SearchQuery::EnsName(name) => {
            app.set_loading(&format!("Resolving {name}..."));
            let tx = tx.clone();
            tokio::spawn(async move {
                let client = RpcClient::new(&rpc_url).unwrap();
                let result = async {
                    // First resolve ENS name to address
                    let addr = client.resolve_ens_to_address(&name).await?;
                    // Then fetch address info
                    client.get_address(addr).await
                }
                .await;
                let _ = tx.send(AsyncMessage::AddressResult(result)).await;
            });
        }
        SearchQuery::Invalid(_) => unreachable!(),
    }
}
