//! Common UI tests - error, loading, layout, and navigation

use super::*;
use tbex::app::{AddressResult, BlockResult, Screen, TxResult};
use tbex::rpc::BlockStats;

// ==================== Error Screen Tests ====================

#[test]
fn test_error_screen_shows_message() {
    let screen = Screen::Error("Failed to fetch block: connection timeout".to_string());
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show error message
    assert!(
        buffer_contains(&buffer, "Error")
            || buffer_contains(&buffer, "Failed")
            || buffer_contains(&buffer, "timeout")
    );
}

// ==================== Loading Screen Tests ====================

#[test]
fn test_loading_screen_shows_message() {
    let screen = Screen::Loading("Fetching block 19000000...".to_string());
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show loading message
    assert!(
        buffer_contains(&buffer, "Loading")
            || buffer_contains(&buffer, "Fetching")
            || buffer_contains(&buffer, "19000000")
            || buffer_contains(&buffer, "...")
    );
}

// ==================== Navigation Help Tests ====================

#[test]
fn test_screens_show_navigation_help() {
    // Check that screens show navigation help
    let screens = vec![
        Screen::Home,
        Screen::BlockResult(BlockResult {
            info: mock_block_info(),
            transactions: vec![],
            stats: BlockStats::default(),
            selected_index: 0,
            list_mode: true,
        }),
        Screen::TxResult(TxResult {
            info: mock_tx_info(),
            selected_link: 0,
            transfer_scroll: 0,
            log_scroll: 0,
        }),
    ];

    for screen in screens {
        let app = create_test_app(screen, true);
        let buffer = render_to_buffer(&app, 100, 40);

        // Should show some navigation help
        assert!(
            buffer_contains(&buffer, "back")
                || buffer_contains(&buffer, "quit")
                || buffer_contains(&buffer, "home")
                || buffer_contains(&buffer, "Esc")
                || buffer_contains(&buffer, "Enter")
                || buffer_contains(&buffer, "navigate")
                || buffer_contains(&buffer, "↑")
                || buffer_contains(&buffer, "↓")
        );
    }
}

// ==================== Layout Tests ====================

#[test]
fn test_small_terminal_renders_without_panic() {
    // Ensure UI handles small terminals gracefully
    let screens = vec![
        Screen::Home,
        Screen::BlockResult(BlockResult {
            info: mock_block_info(),
            transactions: vec![],
            stats: BlockStats::default(),
            selected_index: 0,
            list_mode: true,
        }),
        Screen::TxResult(TxResult {
            info: mock_tx_info(),
            selected_link: 0,
            transfer_scroll: 0,
            log_scroll: 0,
        }),
        Screen::AddressResult(AddressResult {
            info: mock_address_info_eoa(),
            selected_link: 0,
        }),
        Screen::Error("Error".to_string()),
        Screen::Loading("Loading...".to_string()),
    ];

    for screen in screens {
        let app = create_test_app(screen, true);
        // This should not panic even with very small dimensions
        let _ = render_to_buffer(&app, 40, 10);
    }
}

#[test]
fn test_large_terminal_renders_without_panic() {
    // Ensure UI handles large terminals gracefully
    let app = create_test_app(Screen::Home, true);
    let _ = render_to_buffer(&app, 200, 100);
}
