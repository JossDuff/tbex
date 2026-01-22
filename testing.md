# Testing Guide for tbex

## Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only UI integration tests
cargo test --test ui

# Run specific UI test file
cargo test --test ui home_tests
cargo test --test ui block_tests
cargo test --test ui tx_tests
cargo test --test ui address_tests
cargo test --test ui common_tests

# Run tests for a specific source module
cargo test rpc::tests
cargo test ui::tests
cargo test app::tests
cargo test search::tests

# Run a specific test
cargo test test_format_u256_zero
cargo test test_home_screen_with_rpc_shows_title

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

## Test Structure

```
tests/
├── ui.rs                   # Entry point for UI tests
└── ui/
    ├── mod.rs              # Shared imports, mock data, helper functions
    ├── home_tests.rs       # Home/RPC setup screen tests (5 tests)
    ├── block_tests.rs      # Block page tests (5 tests)
    ├── tx_tests.rs         # Transaction page tests (8 tests)
    ├── address_tests.rs    # Address page tests (7 tests)
    └── common_tests.rs     # Error, loading, layout, nav tests (5 tests)

src/
├── app.rs                  # Unit tests for app state (17 tests)
├── rpc.rs                  # Unit tests for RPC/formatting (26 tests)
├── search.rs               # Unit tests for query parsing (7 tests)
└── ui/
    ├── mod.rs
    ├── helper.rs           # Unit tests for UI helpers (19 tests)
    ├── block_page.rs
    ├── tx_page.rs
    └── address_page.rs
```

## Test Coverage Summary

### Unit Tests (in source files)

| Module | Tests | Coverage Focus |
|--------|-------|----------------|
| `rpc.rs` | 26 | Formatting, decoding, type conversions |
| `ui/helper.rs` | 19 | Display formatting, layout helpers |
| `app.rs` | 17 | State machine, navigation, link counting |
| `search.rs` | 7 | Query parsing |
| **Subtotal** | **69** | |

### Integration Tests (in tests/ directory)

| File | Tests | Coverage Focus |
|------|-------|----------------|
| `ui/home_tests.rs` | 5 | Home screen, RPC setup, search bar, network info |
| `ui/block_tests.rs` | 5 | Block info, miner, gas, transaction list |
| `ui/tx_tests.rs` | 8 | Tx hash, from/to, value, gas, status, method, transfers, logs |
| `ui/address_tests.rs` | 7 | EOA/contract info, balance, nonce, tokens, proxy |
| `ui/common_tests.rs` | 5 | Error, loading, navigation help, layout edge cases |
| **Subtotal** | **30** | |

**Total: 99 tests**

## Test Categories

### Unit Tests

#### rpc.rs Tests

**Formatting Functions:**
- `format_u256_decimals` - Token amount formatting with various decimal places
- `hex_encode` - Byte to hex string conversion

**Decoding Functions:**
- `decode_function_selector` - Function signature recognition (transfer, approve, etc.)
- `decode_event_signature` - Event signature recognition (Transfer, Approval, Swap, etc.)
- `detect_builder_tag` - Block builder identification

**ENS:**
- `namehash` - ENS name hashing algorithm

**Data Structures:**
- `DecodedParam` - Parameter parsing with address detection
- `TxType` - Transaction type classification

#### ui/helper.rs Tests

**Display Formatting:**
- `truncate_hash` - Hash/address truncation for display
- `format_gas` - Gas amount formatting (K, M suffixes)
- `format_gwei` - Wei to Gwei conversion
- `format_eth` - Wei to ETH conversion
- `format_token_amount` - Generic token amount formatting
- `format_address_with_ens` - Address with optional ENS name
- `format_addr_fixed_width` - Fixed-width address display

**Layout:**
- `padded_rect` - Rectangle padding calculations

#### app.rs Tests

**Initialization:**
- App creation with/without RPC
- RPC override handling

**Navigation:**
- Screen history (push/pop)
- Go back behavior
- Go home behavior

**Link Navigation (TxResult):**
- Basic link counting (from, to, block)
- Link counting with token transfers
- Link counting with logs (variable address params)
- Wrap-around behavior
- Scroll auto-adjustment

**Selected Link Resolution:**
- Getting correct NavLink for each position
- Transfer addresses
- Log addresses

**History:**
- History navigation
- History selection

**Block Result:**
- Mode toggling (info/list)

#### search.rs Tests

- Address parsing (40 hex chars)
- Transaction hash parsing (64 hex chars)
- Block number parsing (decimal and hex)
- ENS name parsing
- Case insensitivity

### Integration Tests (UI Rendering)

The `tests/ui/` directory contains UI snapshot-style tests that verify screen rendering.

#### home_tests.rs
- `test_home_screen_with_rpc_shows_title` - ASCII art title displayed
- `test_home_screen_with_rpc_shows_search_bar` - Search input visible
- `test_home_screen_with_rpc_shows_network_info` - Block/gas info shown
- `test_home_screen_shows_recent_searches` - History list displayed
- `test_home_screen_no_rpc_shows_setup` - RPC setup prompt when unconfigured

#### block_tests.rs
- `test_block_screen_shows_block_number` - Block number displayed
- `test_block_screen_shows_miner_info` - Miner/builder info shown
- `test_block_screen_shows_gas_info` - Gas usage displayed
- `test_block_screen_list_mode_shows_transactions` - Transaction list rendered
- `test_block_screen_shows_tx_count` - Transaction count shown

#### tx_tests.rs
- `test_tx_screen_shows_hash` - Transaction hash displayed
- `test_tx_screen_shows_from_to` - From/to addresses with ENS
- `test_tx_screen_shows_value` - ETH value displayed
- `test_tx_screen_shows_gas_info` - Gas usage and fees
- `test_tx_screen_shows_status` - Success/failure status
- `test_tx_screen_shows_method` - Decoded method name
- `test_tx_screen_shows_token_transfers` - Token transfer section
- `test_tx_screen_shows_logs` - Event logs with parameters

#### address_tests.rs
- `test_address_screen_eoa_shows_address` - Address/ENS displayed
- `test_address_screen_eoa_shows_balance` - ETH balance shown
- `test_address_screen_eoa_shows_nonce` - Transaction count
- `test_address_screen_eoa_shows_token_balances` - Token holdings
- `test_address_screen_contract_shows_code_size` - Contract indicator
- `test_address_screen_contract_shows_token_info` - Token metadata
- `test_address_screen_proxy_shows_implementation` - Proxy implementation

#### common_tests.rs
- `test_error_screen_shows_message` - Error message displayed
- `test_loading_screen_shows_message` - Loading indicator shown
- `test_screens_show_navigation_help` - Nav help on all screens
- `test_small_terminal_renders_without_panic` - 40x10 terminal handling
- `test_large_terminal_renders_without_panic` - 200x100 terminal handling

## Adding New Tests

When adding features, add tests for:

1. **Pure functions first** - Easiest to test, highest value
2. **State transitions** - App screen changes, navigation
3. **Edge cases** - Empty data, max values, invalid input
4. **UI rendering** - Add to appropriate `tests/ui/*_tests.rs` file

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### UI Test Template

```rust
// In tests/ui/your_page_tests.rs
use super::*;
use tbex::app::Screen;

#[test]
fn test_screen_shows_element() {
    let screen = Screen::YourScreen(YourScreenData { ... });
    let app = create_test_app(screen, true);
    let buffer = render_to_buffer(&app, 100, 40);
    
    assert!(buffer_contains(&buffer, "expected text"));
}
```

## Mock Data Helpers

All mock data helpers are in `tests/ui/mod.rs`:

| Function | Description |
|----------|-------------|
| `mock_config()` | Config with RPC URL and recent searches |
| `mock_config_no_rpc()` | Config without RPC (triggers setup screen) |
| `mock_network_info()` | Network status (block, gas price) |
| `mock_block_info()` | Complete BlockInfo with all fields |
| `mock_tx_summaries()` | List of 3 TxSummary items |
| `mock_tx_info()` | Basic TxInfo |
| `mock_tx_info_with_transfers()` | TxInfo with token transfers and logs |
| `mock_address_info_eoa()` | EOA with balance and token holdings |
| `mock_address_info_contract()` | Contract with token info and proxy |
| `create_test_app(screen, with_rpc)` | Create App in specific state |
| `render_to_buffer(app, w, h)` | Render to test buffer |
| `buffer_contains(buffer, needle)` | Check if buffer contains text |
| `buffer_to_string(buffer)` | Convert buffer to searchable string |
| `buffer_line(buffer, y)` | Get specific line from buffer |
| `print_buffer(buffer)` | Debug helper to print buffer |

## What's NOT Tested

Due to complexity/external dependencies:

1. **Actual RPC calls** - Would need mock server or trait abstraction
2. **Keyboard input** - Event handling
3. **Config file I/O** - File system interaction
4. **Async behavior** - Main event loop

For these, manual testing or integration tests with a test node are recommended.

## Future Improvements

1. **RPC trait abstraction** - Allow mocking RPC client for integration tests
2. **Exact snapshot testing** - Compare buffer byte-by-byte against golden files
3. **Property-based testing** - For formatting functions with proptest
4. **Benchmarks** - For performance-critical formatting code
5. **Coverage reports** - Use tarpaulin for coverage metrics
