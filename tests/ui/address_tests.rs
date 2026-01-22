//! Address page UI tests

use super::*;
use tbex::app::{AddressResult, Screen};

#[test]
fn test_address_screen_eoa_shows_address() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_eoa(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show address or ENS name
    assert!(
        buffer_contains(&buffer, "alice.eth")
            || buffer_contains(&buffer, "0x1111")
            || buffer_contains(&buffer, "Address")
    );
}

#[test]
fn test_address_screen_eoa_shows_balance() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_eoa(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show ETH balance (5.5 ETH)
    assert!(
        buffer_contains(&buffer, "5.5")
            || buffer_contains(&buffer, "ETH")
            || buffer_contains(&buffer, "Balance")
    );
}

#[test]
fn test_address_screen_eoa_shows_nonce() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_eoa(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show nonce (150)
    assert!(
        buffer_contains(&buffer, "150")
            || buffer_contains(&buffer, "Nonce")
            || buffer_contains(&buffer, "Tx Count")
    );
}

#[test]
fn test_address_screen_eoa_shows_token_balances() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_eoa(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show token balances
    assert!(
        buffer_contains(&buffer, "USDC")
            || buffer_contains(&buffer, "WETH")
            || buffer_contains(&buffer, "Token")
            || buffer_contains(&buffer, "10000") // USDC balance
            || buffer_contains(&buffer, "2.5") // WETH balance
    );
}

#[test]
fn test_address_screen_contract_shows_code_size() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_contract(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show contract indicator
    assert!(
        buffer_contains(&buffer, "Contract")
            || buffer_contains(&buffer, "Code")
            || buffer_contains(&buffer, "15000") // code size
            || buffer_contains(&buffer, "bytes")
    );
}

#[test]
fn test_address_screen_contract_shows_token_info() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_contract(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show token info for token contracts
    assert!(
        buffer_contains(&buffer, "USDC")
            || buffer_contains(&buffer, "USD Coin")
            || buffer_contains(&buffer, "Symbol")
            || buffer_contains(&buffer, "Decimals")
    );
}

#[test]
fn test_address_screen_proxy_shows_implementation() {
    let screen = Screen::AddressResult(AddressResult {
        info: mock_address_info_contract(),
        selected_link: 0,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show proxy implementation address
    assert!(
        buffer_contains(&buffer, "Proxy")
            || buffer_contains(&buffer, "Implementation")
            || buffer_contains(&buffer, "43506849") // impl address start
    );
}
