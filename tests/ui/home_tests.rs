//! Home screen UI tests

use super::*;
use tbex::app::Screen;

#[test]
fn test_home_screen_with_rpc_shows_title() {
    let app = create_test_app(Screen::Home, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show the ASCII art title
    assert!(buffer_contains(&buffer, "TBEX") || buffer_contains(&buffer, "████"));
}

#[test]
fn test_home_screen_with_rpc_shows_search_bar() {
    let app = create_test_app(Screen::Home, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show search prompt or search box
    assert!(
        buffer_contains(&buffer, "Search")
            || buffer_contains(&buffer, "Enter")
            || buffer_contains(&buffer, "block")
            || buffer_contains(&buffer, "address")
    );
}

#[test]
fn test_home_screen_with_rpc_shows_network_info() {
    let app = create_test_app(Screen::Home, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show network info like block number or gas price
    assert!(
        buffer_contains(&buffer, "19000000") || // block number
        buffer_contains(&buffer, "gwei") ||
        buffer_contains(&buffer, "Block")
    );
}

#[test]
fn test_home_screen_shows_recent_searches() {
    let app = create_test_app(Screen::Home, true);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show recent searches from config
    assert!(
        buffer_contains(&buffer, "Recent")
            || buffer_contains(&buffer, "History")
            || buffer_contains(&buffer, "vitalik.eth")
            || buffer_contains(&buffer, "0x1234")
    );
}

#[test]
fn test_home_screen_no_rpc_shows_setup() {
    let app = create_test_app(Screen::Home, false);
    let buffer = render_to_buffer(&app, 80, 30);

    // Should show RPC setup prompt
    assert!(
        buffer_contains(&buffer, "RPC")
            || buffer_contains(&buffer, "Enter")
            || buffer_contains(&buffer, "URL")
            || buffer_contains(&buffer, "endpoint")
    );
}
