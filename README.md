# tbex
Locally hosted terminal block explorer

# Testing 

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
```

## Test Structure

```
tests/
├── ui.rs                   # Entry point for UI tests
└── ui/
    ├── mod.rs              # Shared imports, mock data, helper functions
    ├── home_tests.rs       # Home/RPC setup screen tests 
    ├── block_tests.rs      # Block page tests 
    ├── tx_tests.rs         # Transaction page tests 
    ├── address_tests.rs    # Address page tests 
    └── common_tests.rs     # Error, loading, layout, nav tests 

src/
├── app.rs                  # Unit tests for app state 
├── rpc.rs                  # Unit tests for RPC/formatting 
├── search.rs               # Unit tests for query parsing 
└── ui/
    ├── mod.rs
    ├── helper.rs           # Unit tests for UI helpers 
    ├── block_page.rs
    ├── tx_page.rs
    └── address_page.rs
```

All mock data helpers are in `tests/ui/mod.rs`:


