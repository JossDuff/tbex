//! Block page UI tests

use super::*;
use tbex::app::{BlockResult, Screen};
use tbex::rpc::BlockStats;

#[test]
fn test_block_screen_shows_block_number() {
    let screen = Screen::BlockResult(BlockResult {
        info: mock_block_info(),
        transactions: mock_tx_summaries(),
        stats: BlockStats::default(),
        selected_index: 0,
        list_mode: true,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show block number
    assert!(buffer_contains(&buffer, "19000000") || buffer_contains(&buffer, "19,000,000"));
}

#[test]
fn test_block_screen_shows_miner_info() {
    let screen = Screen::BlockResult(BlockResult {
        info: mock_block_info(),
        transactions: mock_tx_summaries(),
        stats: BlockStats::default(),
        selected_index: 0,
        list_mode: false,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show miner address or ENS
    assert!(
        buffer_contains(&buffer, "rsync")
            || buffer_contains(&buffer, "95222290")
            || buffer_contains(&buffer, "Miner")
            || buffer_contains(&buffer, "Builder")
    );
}

#[test]
fn test_block_screen_shows_gas_info() {
    let screen = Screen::BlockResult(BlockResult {
        info: mock_block_info(),
        transactions: mock_tx_summaries(),
        stats: BlockStats::default(),
        selected_index: 0,
        list_mode: false,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show gas used/limit
    assert!(
        buffer_contains(&buffer, "Gas")
            || buffer_contains(&buffer, "15") // 15M gas used
            || buffer_contains(&buffer, "gwei")
    );
}

#[test]
fn test_block_screen_list_mode_shows_transactions() {
    let screen = Screen::BlockResult(BlockResult {
        info: mock_block_info(),
        transactions: mock_tx_summaries(),
        stats: BlockStats::default(),
        selected_index: 0,
        list_mode: true,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 120, 40);

    // Should show transaction hashes or method names
    assert!(
        buffer_contains(&buffer, "aaaa1111")
            || buffer_contains(&buffer, "transfer")
            || buffer_contains(&buffer, "swap")
            || buffer_contains(&buffer, "0x1111")
            || buffer_contains(&buffer, "ETH")
    );
}

#[test]
fn test_block_screen_shows_tx_count() {
    let screen = Screen::BlockResult(BlockResult {
        info: mock_block_info(),
        transactions: mock_tx_summaries(),
        stats: BlockStats::default(),
        selected_index: 0,
        list_mode: true,
    });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);

    // Should show transaction count (150 txs or 3 in our mock list)
    assert!(
        buffer_contains(&buffer, "150")
            || buffer_contains(&buffer, "3")
            || buffer_contains(&buffer, "Tx")
            || buffer_contains(&buffer, "transactions")
    );
}
