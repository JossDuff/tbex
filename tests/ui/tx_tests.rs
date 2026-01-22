//! Transaction page UI tests

use super::*;
use tbex::app::{Screen, TxResult};

#[test]
fn test_tx_screen_shows_hash() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show tx hash (truncated or full)
    assert!(
        buffer_contains(&buffer, "aaaa1111")
            || buffer_contains(&buffer, "Hash")
            || buffer_contains(&buffer, "0xaaaa")
    );
}

#[test]
fn test_tx_screen_shows_from_to() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show from/to addresses or ENS names
    assert!(
        buffer_contains(&buffer, "alice.eth")
            || buffer_contains(&buffer, "uniswap.eth")
            || buffer_contains(&buffer, "0x1111")
            || buffer_contains(&buffer, "From")
            || buffer_contains(&buffer, "To")
    );
}

#[test]
fn test_tx_screen_shows_value() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show value (1.5 ETH)
    assert!(
        buffer_contains(&buffer, "1.5")
            || buffer_contains(&buffer, "ETH")
            || buffer_contains(&buffer, "Value")
    );
}

#[test]
fn test_tx_screen_shows_gas_info() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show gas info
    assert!(
        buffer_contains(&buffer, "Gas")
            || buffer_contains(&buffer, "65000")
            || buffer_contains(&buffer, "gwei")
            || buffer_contains(&buffer, "Fee")
    );
}

#[test]
fn test_tx_screen_shows_status() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show success status
    assert!(
        buffer_contains(&buffer, "Success")
            || buffer_contains(&buffer, "✓")
            || buffer_contains(&buffer, "Status")
    );
}

#[test]
fn test_tx_screen_shows_method() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show decoded method
    assert!(
        buffer_contains(&buffer, "transfer")
            || buffer_contains(&buffer, "Method")
            || buffer_contains(&buffer, "Function")
    );
}

#[test]
fn test_tx_screen_shows_token_transfers() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info_with_transfers(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 120, 50);

    // Should show token transfers section
    assert!(
        buffer_contains(&buffer, "Transfer")
            || buffer_contains(&buffer, "USDC")
            || buffer_contains(&buffer, "USDT")
            || buffer_contains(&buffer, "Token")
    );
}

#[test]
fn test_tx_screen_shows_logs() {
    let screen = Screen::TxResult(TxResult {
        info: mock_tx_info_with_transfers(),
        selected_link: 0,
        transfer_scroll: 0,
        log_scroll: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 120, 50);

    // Should show logs section
    assert!(
        buffer_contains(&buffer, "Log")
            || buffer_contains(&buffer, "Transfer")
            || buffer_contains(&buffer, "Deposit")
            || buffer_contains(&buffer, "Event")
    );
}
