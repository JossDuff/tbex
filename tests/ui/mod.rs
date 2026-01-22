//! UI rendering tests for tbex
//!
//! These tests ensure the UI renders correctly by comparing against expected buffer output.
//! Run with: cargo test --test ui_tests

pub mod address_tests;
pub mod block_tests;
pub mod common_tests;
pub mod home_tests;
pub mod tx_tests;

use tbex::app::{App, Screen};
use tbex::config::Config;
use tbex::rpc::{
    AddressInfo, BlockInfo, DecodedLog, DecodedParam, NetworkInfo, TokenBalance, TokenInfo,
    TokenTransfer, TxInfo, TxSummary, TxType,
};
use tbex::ui::draw;

use alloy::primitives::{Address, Bytes, U256};
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

// ==================== Test Data Builders ====================

pub fn mock_config() -> Config {
    Config {
        rpc_url: Some("http://localhost:8545".to_string()),
        recent_searches: vec![
            "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            "vitalik.eth".to_string(),
            "12345678".to_string(),
        ],
    }
}

pub fn mock_config_no_rpc() -> Config {
    Config {
        rpc_url: None,
        recent_searches: vec![],
    }
}

pub fn mock_network_info() -> NetworkInfo {
    NetworkInfo {
        latest_block: 19000000,
        gas_price: 30_000_000_000, // 30 gwei
        client_version: "Geth/v1.13.0".to_string(),
        base_fee_trend: Some(vec![25, 28, 30, 32, 30]),
        priority_fee_percentiles: Some(vec![1_000_000_000, 2_000_000_000, 5_000_000_000]),
    }
}

pub fn mock_block_info() -> BlockInfo {
    BlockInfo {
        number: 19000000,
        hash: "0xabc123def456789abc123def456789abc123def456789abc123def456789abcd".to_string(),
        parent_hash: "0xdef456789abc123def456789abc123def456789abc123def456789abc123def4"
            .to_string(),
        timestamp: 1700000000,
        miner: "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5".to_string(),
        miner_ens: Some("rsync-builder.eth".to_string()),
        gas_used: 15_000_000,
        gas_limit: 30_000_000,
        base_fee: Some(30_000_000_000),
        tx_count: 150,
        state_root: "0x1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        receipts_root: "0x2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        transactions_root: "0x3333333333333333333333333333333333333333333333333333333333333333"
            .to_string(),
        extra_data: "0x7273796e632d6275696c6465722e78797a".to_string(),
        extra_data_decoded: Some("rsync-builder.xyz".to_string()),
        size: Some(125000),
        uncles_count: 0,
        withdrawals_count: Some(16),
        blob_gas_used: Some(393216),
        excess_blob_gas: Some(0),
        blob_count: 3,
        total_value_transferred: U256::from(100_000_000_000_000_000_000u128), // 100 ETH
        total_fees: U256::from(500_000_000_000_000_000u128),                  // 0.5 ETH
        burnt_fees: U256::from(450_000_000_000_000_000u128),                  // 0.45 ETH
        builder_tag: Some("rsync".to_string()),
    }
}

pub fn mock_tx_summaries() -> Vec<TxSummary> {
    vec![
        TxSummary {
            hash: "0xaaaa111122223333444455556666777788889999aaaabbbbccccddddeeeefffff".to_string(),
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: Some("0x2222222222222222222222222222222222222222".to_string()),
            value: U256::from(1_000_000_000_000_000_000u128), // 1 ETH
            gas_limit: 21000,
            tx_type: TxType::EIP1559,
            is_contract_creation: false,
            from_ens: None,
            to_ens: None,
            input_size: 0,
            method_selector: Some("0xa9059cbb".to_string()),
            decoded_method: Some("transfer".to_string()),
            blob_count: 0,
            fee_paid: Some(U256::from(21000u64 * 50_000_000_000u64)),
        },
        TxSummary {
            hash: "0xbbbb111122223333444455556666777788889999aaaabbbbccccddddeeeefffff".to_string(),
            from: "0x3333333333333333333333333333333333333333".to_string(),
            to: Some("0x4444444444444444444444444444444444444444".to_string()),
            value: U256::ZERO,
            gas_limit: 200000,
            tx_type: TxType::EIP1559,
            is_contract_creation: false,
            from_ens: None,
            to_ens: None,
            input_size: 256,
            method_selector: Some("0x38ed1739".to_string()),
            decoded_method: Some("swap".to_string()),
            blob_count: 0,
            fee_paid: Some(U256::from(150000u64 * 50_000_000_000u64)),
        },
        TxSummary {
            hash: "0xcccc111122223333444455556666777788889999aaaabbbbccccddddeeeefffff".to_string(),
            from: "0x5555555555555555555555555555555555555555".to_string(),
            to: None,
            value: U256::ZERO,
            gas_limit: 1000000,
            tx_type: TxType::EIP1559,
            is_contract_creation: true,
            from_ens: None,
            to_ens: None,
            input_size: 5000,
            method_selector: None,
            decoded_method: None,
            blob_count: 0,
            fee_paid: Some(U256::from(500000u64 * 50_000_000_000u64)),
        },
    ]
}

pub fn mock_tx_info() -> TxInfo {
    TxInfo {
        hash: "0xaaaa111122223333444455556666777788889999aaaabbbbccccddddeeeefffff".to_string(),
        from: "0x1111111111111111111111111111111111111111".to_string(),
        to: Some("0x2222222222222222222222222222222222222222".to_string()),
        value: U256::from(1_500_000_000_000_000_000u128), // 1.5 ETH
        gas_price: Some(50_000_000_000),
        gas_limit: 100000,
        gas_used: Some(65000),
        nonce: 42,
        block_number: Some(19000000),
        status: Some(true),
        input_size: 136,
        tx_type: TxType::EIP1559,
        max_fee_per_gas: Some(100_000_000_000),
        max_priority_fee_per_gas: Some(2_000_000_000),
        tx_index: Some(5),
        contract_created: None,
        logs_count: Some(3),
        access_list_size: None,
        blob_gas_used: None,
        blob_gas_price: None,
        blob_hashes: vec![],
        input_data: Bytes::from_static(&[0xa9, 0x05, 0x9c, 0xbb]), // transfer selector
        from_ens: Some("alice.eth".to_string()),
        to_ens: Some("uniswap.eth".to_string()),
        actual_fee: Some(U256::from(3_250_000_000_000_000u128)), // 0.00325 ETH
        decoded_method: Some("transfer(address,uint256)".to_string()),
        logs: vec![],
        token_transfers: vec![],
    }
}

pub fn mock_tx_info_with_transfers() -> TxInfo {
    let mut info = mock_tx_info();
    info.token_transfers = vec![
        TokenTransfer {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: U256::from(1000_000_000u128), // 1000 USDC
            token_symbol: Some("USDC".to_string()),
            decimals: Some(6),
        },
        TokenTransfer {
            token_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            from: "0x3333333333333333333333333333333333333333".to_string(),
            to: "0x4444444444444444444444444444444444444444".to_string(),
            amount: U256::from(500_000_000u128), // 500 USDT
            token_symbol: Some("USDT".to_string()),
            decimals: Some(6),
        },
    ];
    info.logs = vec![
        DecodedLog {
            address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            topics: vec!["0xddf252...".to_string()],
            data: "0x...".to_string(),
            event_name: Some("Transfer(address,address,uint256)".to_string()),
            decoded_params: vec![
                DecodedParam {
                    name: "from".to_string(),
                    value: "0x1111111111111111111111111111111111111111".to_string(),
                    is_address: true,
                },
                DecodedParam {
                    name: "to".to_string(),
                    value: "0x2222222222222222222222222222222222222222".to_string(),
                    is_address: true,
                },
                DecodedParam {
                    name: "value".to_string(),
                    value: "1000".to_string(),
                    is_address: false,
                },
            ],
        },
        DecodedLog {
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            topics: vec!["0xe1ff...".to_string()],
            data: "0x...".to_string(),
            event_name: Some("Deposit(address,uint256)".to_string()),
            decoded_params: vec![
                DecodedParam {
                    name: "dst".to_string(),
                    value: "0x1111111111111111111111111111111111111111".to_string(),
                    is_address: true,
                },
                DecodedParam {
                    name: "wad".to_string(),
                    value: "1.5".to_string(),
                    is_address: false,
                },
            ],
        },
    ];
    info
}

pub fn mock_address_info_eoa() -> AddressInfo {
    AddressInfo {
        address: Address::parse_checksummed("0x1111111111111111111111111111111111111111", None)
            .unwrap(),
        balance: U256::from(5_500_000_000_000_000_000u128), // 5.5 ETH
        nonce: 150,
        is_contract: false,
        code_size: None,
        proxy_impl: None,
        token_info: None,
        ens_name: Some("alice.eth".to_string()),
        owner: None,
        token_balances: vec![
            TokenBalance {
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                address: Address::parse_checksummed(
                    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    None,
                )
                .unwrap(),
                balance: U256::from(10000_000_000u128), // 10000 USDC
                decimals: 6,
            },
            TokenBalance {
                symbol: "WETH".to_string(),
                name: "Wrapped Ether".to_string(),
                address: Address::parse_checksummed(
                    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    None,
                )
                .unwrap(),
                balance: U256::from(2_500_000_000_000_000_000u128), // 2.5 WETH
                decimals: 18,
            },
        ],
    }
}

pub fn mock_address_info_contract() -> AddressInfo {
    AddressInfo {
        address: Address::parse_checksummed("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", None)
            .unwrap(),
        balance: U256::ZERO,
        nonce: 1,
        is_contract: true,
        code_size: Some(15000),
        proxy_impl: Some(
            Address::parse_checksummed("0x43506849D7C04F9138D1A2050bbF3A0c054402dd", None).unwrap(),
        ),
        token_info: Some(TokenInfo {
            name: Some("USD Coin".to_string()),
            symbol: Some("USDC".to_string()),
            decimals: Some(6),
            total_supply: Some(U256::from(25_000_000_000_000_000u128)),
        }),
        ens_name: None,
        owner: Some("0x807a96288A1A408dBC13DE2b1d087d10356395d2".to_string()),
        token_balances: vec![],
    }
}

pub fn create_test_app(screen: Screen, with_rpc: bool) -> App {
    let config = if with_rpc {
        mock_config()
    } else {
        mock_config_no_rpc()
    };
    let mut app = App::new(config);
    app.screen = screen;
    if with_rpc {
        app.network_info = Some(mock_network_info());
    }
    app
}

// ==================== Helper Functions ====================

/// Render the app to a buffer and return it
pub fn render_to_buffer(app: &App, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            draw(frame, app);
        })
        .unwrap();

    terminal.backend().buffer().clone()
}

/// Check if buffer contains a specific string anywhere
pub fn buffer_contains(buffer: &Buffer, needle: &str) -> bool {
    let content = buffer_to_string(buffer);
    content.contains(needle)
}

/// Convert buffer to a single string for searching
pub fn buffer_to_string(buffer: &Buffer) -> String {
    let mut content = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            content.push(
                buffer
                    .cell((x, y))
                    .map(|c| c.symbol().chars().next().unwrap_or(' '))
                    .unwrap_or(' '),
            );
        }
        content.push('\n');
    }
    content
}

/// Get a specific line from the buffer
#[allow(dead_code)]
pub fn buffer_line(buffer: &Buffer, y: u16) -> String {
    let mut line = String::new();
    for x in 0..buffer.area.width {
        if let Some(cell) = buffer.cell((x, y)) {
            line.push_str(cell.symbol());
        }
    }
    line.trim_end().to_string()
}

/// Print buffer for debugging
#[allow(dead_code)]
pub fn print_buffer(buffer: &Buffer) {
    for y in 0..buffer.area.height {
        println!("{}", buffer_line(buffer, y));
    }
}
