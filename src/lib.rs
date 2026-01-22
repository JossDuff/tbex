//! tbex - Terminal Blockchain Explorer
//!
//! A terminal-based blockchain explorer for EVM chains.

pub mod app;
pub mod config;
pub mod rpc;
pub mod search;
pub mod ui;

// Re-export commonly used types
pub use app::{AddressResult, App, BlockResult, NavLink, Screen, TxResult};
pub use config::Config;
