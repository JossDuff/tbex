use crate::config::Config;
use crate::rpc::{AddressInfo, BlockInfo, BlockStats, NetworkInfo, RpcClient, TxInfo, TxSummary};
use tui_input::Input;

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Loading(String),
    BlockResult(BlockResult),
    TxResult(TxResult),
    AddressResult(AddressResult),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct BlockResult {
    pub info: BlockInfo,
    pub transactions: Vec<TxSummary>,
    pub stats: BlockStats,
    pub selected_index: usize,
    pub list_mode: bool, // true = tx list, false = info links
}

#[derive(Debug, Clone)]
pub struct TxResult {
    pub info: TxInfo,
    pub selected_link: usize, // 0 = from, 1 = to, 2 = block, 3 = contract created, then transfers, then logs
    pub transfer_scroll: usize, // Scroll offset for token transfers
    pub log_scroll: usize,    // Scroll offset for logs
}

// Max visible items in scrollable sections
pub const MAX_VISIBLE_TRANSFERS: usize = 4;
pub const MAX_VISIBLE_LOGS: usize = 3;

#[derive(Debug, Clone)]
pub struct AddressResult {
    pub info: AddressInfo,
    pub selected_link: usize, // 0 = proxy impl
}

/// Navigable links from a screen
#[derive(Debug, Clone)]
pub enum NavLink {
    Address(String),
    Block(u64),
    Transaction(String),
}

pub struct App {
    pub config: Config,
    pub screen: Screen,
    pub history: Vec<Screen>,
    pub search_input: Input,
    pub rpc_input: Input,
    pub selected_history_index: Option<usize>,
    pub should_quit: bool,
    pub rpc_url: Option<String>,
    pub rpc_client: Option<RpcClient>,
    pub network_info: Option<NetworkInfo>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let rpc_url = config.rpc_url.clone();
        let rpc_client = rpc_url.as_ref().and_then(|url| RpcClient::new(url).ok());

        Self {
            config,
            screen: Screen::Home,
            history: Vec::new(),
            search_input: Input::default(),
            rpc_input: Input::default(),
            selected_history_index: None,
            should_quit: false,
            rpc_url,
            rpc_client,
            network_info: None,
        }
    }

    pub fn submit_rpc(&mut self) -> Result<(), String> {
        let url = self.rpc_input.value().trim().to_string();
        if url.is_empty() {
            return Err("RPC URL cannot be empty".to_string());
        }

        // Try to create a client to validate the URL
        match RpcClient::new(&url) {
            Ok(client) => {
                self.rpc_client = Some(client);
                self.rpc_url = Some(url.clone());
                let _ = self.config.set_rpc(url);
                self.rpc_input.reset();
                Ok(())
            }
            Err(e) => Err(format!("Invalid RPC URL: {e}")),
        }
    }

    pub fn needs_rpc_setup(&self) -> bool {
        self.rpc_client.is_none()
    }

    pub fn get_recent_searches(&self) -> &[String] {
        &self.config.recent_searches
    }

    pub fn select_history_prev(&mut self) {
        let len = self.config.recent_searches.len();
        if len == 0 {
            return;
        }

        self.selected_history_index = match self.selected_history_index {
            None => Some(0),
            Some(0) => None, // Wrap to search input
            Some(i) => Some(i - 1),
        };
    }

    pub fn select_history_next(&mut self) {
        let len = self.config.recent_searches.len();
        if len == 0 {
            return;
        }

        self.selected_history_index = match self.selected_history_index {
            None => Some(0),
            Some(i) if i >= len - 1 => None, // Wrap to search input
            Some(i) => Some(i + 1),
        };
    }

    pub fn get_selected_history_query(&self) -> Option<String> {
        self.selected_history_index
            .and_then(|i| self.config.recent_searches.get(i).cloned())
    }

    pub fn clear_history_selection(&mut self) {
        self.selected_history_index = None;
    }

    pub fn delete_selected_history(&mut self) {
        if let Some(idx) = self.selected_history_index {
            if idx < self.config.recent_searches.len() {
                self.config.recent_searches.remove(idx);
                let _ = self.config.save();

                // Adjust selection
                if self.config.recent_searches.is_empty() {
                    self.selected_history_index = None;
                } else if idx >= self.config.recent_searches.len() {
                    self.selected_history_index = Some(self.config.recent_searches.len() - 1);
                }
            }
        }
    }

    pub fn submit_search(&mut self) -> Option<String> {
        let value = self.search_input.value();
        if value.is_empty() {
            return None;
        }

        let query = value.to_string();
        self.search_input.reset();
        let _ = self.config.add_recent_search(query.clone());
        Some(query)
    }

    pub fn navigate_to(&mut self, screen: Screen) {
        if !matches!(self.screen, Screen::Home | Screen::Loading(_)) {
            self.history.push(self.screen.clone());
        }
        self.screen = screen;
    }

    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.screen = prev;
            true
        } else {
            self.go_home();
            true
        }
    }

    pub fn go_home(&mut self) {
        self.history.clear();
        self.screen = Screen::Home;
    }

    pub fn set_loading(&mut self, msg: &str) {
        // Save current screen to history before showing loading (if it's a navigable screen)
        if !matches!(
            self.screen,
            Screen::Home | Screen::Loading(_) | Screen::Error(_)
        ) {
            self.history.push(self.screen.clone());
        }
        self.screen = Screen::Loading(msg.to_string());
    }

    pub fn set_error(&mut self, msg: String) {
        // Save current screen to history before showing error (if it's a navigable screen)
        if !matches!(
            self.screen,
            Screen::Home | Screen::Loading(_) | Screen::Error(_)
        ) {
            self.history.push(self.screen.clone());
        }
        self.screen = Screen::Error(msg);
    }

    pub fn set_block_result(
        &mut self,
        info: BlockInfo,
        transactions: Vec<TxSummary>,
        stats: crate::rpc::BlockStats,
    ) {
        self.navigate_to(Screen::BlockResult(BlockResult {
            info,
            transactions,
            stats,
            selected_index: 0,
            list_mode: true,
        }));
    }

    pub fn set_tx_result(&mut self, info: TxInfo) {
        self.navigate_to(Screen::TxResult(TxResult {
            info,
            selected_link: 0,
            transfer_scroll: 0,
            log_scroll: 0,
        }));
    }

    pub fn set_address_result(&mut self, info: AddressInfo) {
        self.navigate_to(Screen::AddressResult(AddressResult {
            info,
            selected_link: 0,
        }));
    }

    pub fn set_network_info(&mut self, info: NetworkInfo) {
        self.network_info = Some(info);
    }

    pub fn has_rpc(&self) -> bool {
        self.rpc_client.is_some()
    }

    pub fn is_on_home(&self) -> bool {
        matches!(self.screen, Screen::Home)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.screen, Screen::Loading(_))
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        match &mut self.screen {
            Screen::BlockResult(result) => {
                if result.list_mode
                    && result.selected_index > 0 {
                        result.selected_index -= 1;
                    }
            }
            Screen::TxResult(result) => {
                // Calculate total navigable links
                let mut max = 1; // from
                if result.info.to.is_some() {
                    max += 1; // to
                }
                if result.info.block_number.is_some() {
                    max += 1;
                }
                if result.info.contract_created.is_some() {
                    max += 1;
                }
                let transfer_start = max;
                // Each token transfer has 3 addresses (from, to, token contract)
                max += result.info.token_transfers.len() * 3;
                let log_start = max;
                // Each log has 1 contract address + N address params
                for log in &result.info.logs {
                    max += 1; // contract address
                    max += log.decoded_params.iter().filter(|p| p.is_address).count();
                }

                if result.selected_link > 0 {
                    result.selected_link -= 1;
                } else {
                    result.selected_link = max - 1;
                }

                // Auto-scroll transfers
                if result.selected_link >= transfer_start && result.selected_link < log_start {
                    let transfer_idx = (result.selected_link - transfer_start) / 3;
                    if transfer_idx < result.transfer_scroll {
                        result.transfer_scroll = transfer_idx;
                    } else if transfer_idx >= result.transfer_scroll + MAX_VISIBLE_TRANSFERS {
                        result.transfer_scroll = transfer_idx - MAX_VISIBLE_TRANSFERS + 1;
                    }
                }

                // Auto-scroll logs - find which log the selected link is in
                if result.selected_link >= log_start {
                    let mut link_offset = log_start;
                    for (log_idx, log) in result.info.logs.iter().enumerate() {
                        let links_in_log =
                            1 + log.decoded_params.iter().filter(|p| p.is_address).count();
                        if result.selected_link < link_offset + links_in_log {
                            // Found the log
                            if log_idx < result.log_scroll {
                                result.log_scroll = log_idx;
                            } else if log_idx >= result.log_scroll + MAX_VISIBLE_LOGS {
                                result.log_scroll = log_idx - MAX_VISIBLE_LOGS + 1;
                            }
                            break;
                        }
                        link_offset += links_in_log;
                    }
                }
            }
            Screen::AddressResult(result) => {
                let max = if result.info.proxy_impl.is_some() {
                    1
                } else {
                    0
                };
                if max > 0 {
                    if result.selected_link > 0 {
                        result.selected_link -= 1;
                    } else {
                        result.selected_link = max - 1;
                    }
                }
            }
            _ => {}
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        match &mut self.screen {
            Screen::BlockResult(result) => {
                if result.list_mode && !result.transactions.is_empty()
                    && result.selected_index < result.transactions.len() - 1 {
                        result.selected_index += 1;
                    }
            }
            Screen::TxResult(result) => {
                // Calculate total navigable links
                let mut max = 1; // from
                if result.info.to.is_some() {
                    max += 1; // to
                }
                if result.info.block_number.is_some() {
                    max += 1;
                }
                if result.info.contract_created.is_some() {
                    max += 1;
                }
                let transfer_start = max;
                // Each token transfer has 3 addresses (from, to, token contract)
                max += result.info.token_transfers.len() * 3;
                let log_start = max;
                // Each log has 1 contract address + N address params
                for log in &result.info.logs {
                    max += 1; // contract address
                    max += log.decoded_params.iter().filter(|p| p.is_address).count();
                }

                result.selected_link = (result.selected_link + 1) % max;

                // Auto-scroll transfers
                if result.selected_link >= transfer_start && result.selected_link < log_start {
                    let transfer_idx = (result.selected_link - transfer_start) / 3;
                    if transfer_idx < result.transfer_scroll {
                        result.transfer_scroll = transfer_idx;
                    } else if transfer_idx >= result.transfer_scroll + MAX_VISIBLE_TRANSFERS {
                        result.transfer_scroll = transfer_idx - MAX_VISIBLE_TRANSFERS + 1;
                    }
                }

                // Auto-scroll logs - find which log the selected link is in
                if result.selected_link >= log_start {
                    let mut link_offset = log_start;
                    for (log_idx, log) in result.info.logs.iter().enumerate() {
                        let links_in_log =
                            1 + log.decoded_params.iter().filter(|p| p.is_address).count();
                        if result.selected_link < link_offset + links_in_log {
                            // Found the log
                            if log_idx < result.log_scroll {
                                result.log_scroll = log_idx;
                            } else if log_idx >= result.log_scroll + MAX_VISIBLE_LOGS {
                                result.log_scroll = log_idx - MAX_VISIBLE_LOGS + 1;
                            }
                            break;
                        }
                        link_offset += links_in_log;
                    }
                }
            }
            Screen::AddressResult(result) => {
                let max = if result.info.proxy_impl.is_some() {
                    1
                } else {
                    0
                };
                if max > 0 {
                    result.selected_link = (result.selected_link + 1) % max;
                }
            }
            _ => {}
        }
    }

    /// Toggle between list mode and link mode (for blocks)
    pub fn toggle_mode(&mut self) {
        if let Screen::BlockResult(result) = &mut self.screen {
            result.list_mode = !result.list_mode;
            result.selected_index = 0;
        }
    }

    /// Get the currently selected navigation link
    pub fn get_selected_link(&self) -> Option<NavLink> {
        match &self.screen {
            Screen::BlockResult(result) => {
                if result.list_mode {
                    result
                        .transactions
                        .get(result.selected_index)
                        .map(|tx| NavLink::Transaction(tx.hash.clone()))
                } else {
                    // Link to parent block
                    if result.info.number > 0 {
                        Some(NavLink::Block(result.info.number - 1))
                    } else {
                        None
                    }
                }
            }
            Screen::TxResult(result) => {
                let mut links: Vec<NavLink> = vec![NavLink::Address(result.info.from.clone())];

                if let Some(to) = &result.info.to {
                    links.push(NavLink::Address(to.clone()));
                }

                if let Some(block) = result.info.block_number {
                    links.push(NavLink::Block(block));
                }

                if let Some(contract) = &result.info.contract_created {
                    links.push(NavLink::Address(contract.clone()));
                }

                // Add token transfer addresses (from, to, token contract)
                for transfer in &result.info.token_transfers {
                    links.push(NavLink::Address(transfer.from.clone()));
                    links.push(NavLink::Address(transfer.to.clone()));
                    links.push(NavLink::Address(transfer.token_address.clone()));
                }

                // Add log addresses (contract + address params)
                for log in &result.info.logs {
                    links.push(NavLink::Address(log.address.clone()));
                    for param in &log.decoded_params {
                        if param.is_address {
                            links.push(NavLink::Address(param.value.clone()));
                        }
                    }
                }

                links.get(result.selected_link).cloned()
            }
            Screen::AddressResult(result) => {
                if result.info.proxy_impl.is_some() && result.selected_link == 0 {
                    result
                        .info
                        .proxy_impl
                        .map(|a| NavLink::Address(format!("{a:?}")))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::{BlockInfo, DecodedLog, DecodedParam, TokenTransfer, TxInfo, TxType};
    use alloy::primitives::{Bytes, U256};

    // ==================== Helper functions for creating test data ====================

    fn mock_config() -> Config {
        Config {
            rpc_url: Some("http://localhost:8545".to_string()),
            recent_searches: vec![],
        }
    }

    fn mock_tx_info() -> TxInfo {
        TxInfo {
            hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            from: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            to: Some("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string()),
            value: U256::ZERO,
            gas_price: Some(50_000_000_000),
            gas_limit: 21000,
            gas_used: Some(21000),
            nonce: 1,
            block_number: Some(12345678),
            status: Some(true),
            input_size: 0,
            tx_type: TxType::EIP1559,
            max_fee_per_gas: Some(100_000_000_000),
            max_priority_fee_per_gas: Some(2_000_000_000),
            tx_index: Some(0),
            contract_created: None,
            logs_count: Some(0),
            access_list_size: None,
            blob_gas_used: None,
            blob_gas_price: None,
            blob_hashes: vec![],
            input_data: Bytes::new(),
            from_ens: None,
            to_ens: None,
            actual_fee: None,
            decoded_method: None,
            logs: vec![],
            token_transfers: vec![],
        }
    }

    fn mock_tx_info_with_transfers(num_transfers: usize, num_logs: usize) -> TxInfo {
        let mut info = mock_tx_info();

        for i in 0..num_transfers {
            info.token_transfers.push(TokenTransfer {
                token_address: format!("0x{:040x}", i),
                from: format!("0x{:040x}", i * 2),
                to: format!("0x{:040x}", i * 2 + 1),
                amount: U256::from(1000u64),
                token_symbol: Some(format!("TKN{}", i)),
                decimals: Some(18),
            });
        }

        for i in 0..num_logs {
            info.logs.push(DecodedLog {
                address: format!("0x{:040x}", i + 100),
                topics: vec![],
                data: "0x".to_string(),
                event_name: Some("Transfer(address,address,uint256)".to_string()),
                decoded_params: vec![
                    DecodedParam {
                        name: "from".to_string(),
                        value: format!("0x{:040x}", i * 3),
                        is_address: true,
                    },
                    DecodedParam {
                        name: "to".to_string(),
                        value: format!("0x{:040x}", i * 3 + 1),
                        is_address: true,
                    },
                    DecodedParam {
                        name: "value".to_string(),
                        value: "1000".to_string(),
                        is_address: false,
                    },
                ],
            });
        }

        info
    }

    fn mock_block_info() -> BlockInfo {
        BlockInfo {
            number: 12345678,
            hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            parent_hash: "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
                .to_string(),
            timestamp: 1700000000,
            miner: "0x0000000000000000000000000000000000000000".to_string(),
            miner_ens: None,
            gas_used: 15_000_000,
            gas_limit: 30_000_000,
            base_fee: Some(50_000_000_000),
            tx_count: 100,
            state_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            receipts_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            transactions_root: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            extra_data: "0x".to_string(),
            extra_data_decoded: None,
            size: Some(50000),
            uncles_count: 0,
            withdrawals_count: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            blob_count: 0,
            total_value_transferred: U256::ZERO,
            total_fees: U256::ZERO,
            burnt_fees: U256::ZERO,
            builder_tag: None,
        }
    }

    // ==================== App initialization tests ====================

    #[test]
    fn test_app_new() {
        let config = mock_config();
        let app = App::new(config);
        assert!(app.has_rpc());
        assert!(app.is_on_home());
    }

    #[test]
    fn test_app_new_no_rpc() {
        let mut config = mock_config();
        config.rpc_url = None;
        let app = App::new(config);
        assert!(!app.has_rpc());
    }

    // ==================== Navigation tests ====================

    #[test]
    fn test_navigate_to_and_back() {
        let config = mock_config();
        let mut app = App::new(config);

        app.navigate_to(Screen::Loading("Loading...".to_string()));
        assert!(matches!(app.screen, Screen::Loading(_)));

        let went_back = app.go_back();
        assert!(went_back);
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_go_back_at_home() {
        let config = mock_config();
        let mut app = App::new(config);

        // go_back always returns true (goes home if no history)
        let went_back = app.go_back();
        assert!(went_back);
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_go_home() {
        let config = mock_config();
        let mut app = App::new(config);

        app.navigate_to(Screen::Loading("Loading...".to_string()));
        app.navigate_to(Screen::Error("Error".to_string()));

        app.go_home();
        assert!(matches!(app.screen, Screen::Home));
        assert!(app.history.is_empty());
    }

    // ==================== TxResult link counting tests ====================

    #[test]
    fn test_tx_result_basic_link_count() {
        // Basic tx: from + to + block = 3 links
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info());

        if let Screen::TxResult(result) = &app.screen {
            // from(1) + to(1) + block(1) = 3
            // Navigate through all links
            assert_eq!(result.selected_link, 0); // starts at from

            app.select_next();
            if let Screen::TxResult(result) = &app.screen {
                assert_eq!(result.selected_link, 1); // to
            }

            app.select_next();
            if let Screen::TxResult(result) = &app.screen {
                assert_eq!(result.selected_link, 2); // block
            }

            app.select_next();
            if let Screen::TxResult(result) = &app.screen {
                assert_eq!(result.selected_link, 0); // wraps to from
            }
        } else {
            panic!("Expected TxResult screen");
        }
    }

    #[test]
    fn test_tx_result_with_transfers_link_count() {
        // Tx with 2 transfers: from + to + block + (2 transfers * 3 addresses each) = 3 + 6 = 9
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info_with_transfers(2, 0));

        // Navigate through all 9 links and verify wrap
        for _ in 0..9 {
            app.select_next();
        }
        if let Screen::TxResult(result) = &app.screen {
            assert_eq!(result.selected_link, 0); // should wrap back to start
        }
    }

    #[test]
    fn test_tx_result_with_logs_link_count() {
        // Tx with 2 logs: from + to + block + (2 logs * (1 contract + 2 address params)) = 3 + 6 = 9
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info_with_transfers(0, 2));

        // from + to + block + 2*(contract + 2 addr params) = 3 + 6 = 9
        for _ in 0..9 {
            app.select_next();
        }
        if let Screen::TxResult(result) = &app.screen {
            assert_eq!(result.selected_link, 0); // should wrap
        }
    }

    #[test]
    fn test_tx_result_select_prev_wraps() {
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info());

        // At position 0, select_prev should wrap to last
        app.select_prev();

        if let Screen::TxResult(result) = &app.screen {
            assert_eq!(result.selected_link, 2); // should be at block (last link)
        }
    }

    // ==================== TxResult scroll tests ====================

    #[test]
    fn test_tx_result_transfer_scroll() {
        let config = mock_config();
        let mut app = App::new(config);

        // Create tx with 10 transfers
        app.set_tx_result(mock_tx_info_with_transfers(10, 0));

        if let Screen::TxResult(result) = &app.screen {
            assert_eq!(result.transfer_scroll, 0);
        }

        // Navigate past the basic links (from, to, block) into transfers
        app.select_next(); // to
        app.select_next(); // block
        app.select_next(); // transfer 0 from

        // Navigate through many transfers to trigger scroll
        for _ in 0..20 {
            app.select_next();
        }

        if let Screen::TxResult(result) = &app.screen {
            // Scroll should have moved
            assert!(result.transfer_scroll > 0 || result.selected_link == 0);
        }
    }

    // ==================== get_selected_link tests ====================

    #[test]
    fn test_get_selected_link_tx_from() {
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info());

        let link = app.get_selected_link();
        assert!(matches!(link, Some(NavLink::Address(_))));

        if let Some(NavLink::Address(addr)) = link {
            assert!(addr.starts_with("0xaaaa")); // from address
        }
    }

    #[test]
    fn test_get_selected_link_tx_block() {
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info());

        app.select_next(); // to
        app.select_next(); // block

        let link = app.get_selected_link();
        assert!(matches!(link, Some(NavLink::Block(12345678))));
    }

    #[test]
    fn test_get_selected_link_tx_transfer() {
        let config = mock_config();
        let mut app = App::new(config);

        app.set_tx_result(mock_tx_info_with_transfers(1, 0));

        app.select_next(); // to
        app.select_next(); // block
        app.select_next(); // transfer from

        let link = app.get_selected_link();
        assert!(matches!(link, Some(NavLink::Address(_))));
    }

    // ==================== History tests ====================

    #[test]
    fn test_history_navigation() {
        let mut config = mock_config();
        config.recent_searches = vec![
            "0x123".to_string(),
            "0x456".to_string(),
            "0x789".to_string(),
        ];
        let mut app = App::new(config);

        assert_eq!(app.selected_history_index, None);

        app.select_history_next();
        assert_eq!(app.selected_history_index, Some(0));

        app.select_history_next();
        assert_eq!(app.selected_history_index, Some(1));

        app.select_history_prev();
        assert_eq!(app.selected_history_index, Some(0));

        app.select_history_prev();
        assert_eq!(app.selected_history_index, None);
    }

    #[test]
    fn test_get_selected_history_query() {
        let mut config = mock_config();
        config.recent_searches = vec!["query1".to_string(), "query2".to_string()];
        let mut app = App::new(config);

        assert_eq!(app.get_selected_history_query(), None);

        app.select_history_next();
        assert_eq!(app.get_selected_history_query(), Some("query1".to_string()));
    }

    // ==================== BlockResult tests ====================

    #[test]
    fn test_block_result_toggle_mode() {
        let config = mock_config();
        let mut app = App::new(config);

        app.set_block_result(mock_block_info(), vec![], crate::rpc::BlockStats::default());

        if let Screen::BlockResult(result) = &app.screen {
            assert!(result.list_mode); // starts in list mode
        }

        app.toggle_mode();

        if let Screen::BlockResult(result) = &app.screen {
            assert!(!result.list_mode); // now in info mode
        }
    }
}
