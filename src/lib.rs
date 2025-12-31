//! # The Slvr Programming Language
//!
//! A Turing-incomplete smart contract language designed for the SilverBitcoin blockchain.
//! Slvr combines the best practices from Pact with SilverBitcoin's architecture,
//! providing a safe, efficient, and deterministic execution environment for smart contracts.
//!
//! ## Overview
//!
//! Slvr is a transactional, database-focused language that emphasizes:
//! - **Safety**: Turing-incomplete design prevents infinite loops and unbounded recursion
//! - **Determinism**: Consistent execution across all nodes in the network
//! - **Efficiency**: Optimized for blockchain execution with fuel metering
//! - **Clarity**: Readable syntax that makes contract logic transparent
//!
//! ## Architecture
//!
//! The language is built on a multi-stage compilation pipeline:
//! 1. **Lexer**: Tokenizes source code into a stream of tokens
//! 2. **Parser**: Builds an Abstract Syntax Tree (AST) from tokens
//! 3. **Type Checker**: Validates types and catches errors early
//! 4. **Compiler**: Generates optimized bytecode
//! 5. **Runtime**: Executes bytecode with fuel metering and state management
//!
//! ## Example Contract
//!
//! ```slvr
//! (module coin
//!   "A simple coin contract"
//!
//!   (defschema coin-schema
//!     "Schema for coin objects"
//!     balance:integer
//!     owner:string)
//!
//!   (deftable coins:{coin-schema}
//!     "Table of coin objects")
//!
//!   (defun mint (owner:string amount:integer)
//!     "Mint new coins"
//!     (insert coins owner
//!       { "balance": amount, "owner": owner }))
//!
//!   (defun transfer (from:string to:string amount:integer)
//!     "Transfer coins between accounts"
//!     (with-read coins from { "balance": balance }
//!       (enforce (>= balance amount) "Insufficient balance")
//!       (update coins from { "balance": (- balance amount) })
//!       (update coins to { "balance": (+ (at "balance" (read coins to)) amount) }))))
//! ```

pub mod account_api;
pub mod api;
pub mod api_handler;
pub mod ast;
pub mod blockchain_api;
pub mod bytecode;
pub mod chainweb;
pub mod compiler;
pub mod debugger;
pub mod defcap;
pub mod defpact;
pub mod error;
pub mod evaluator;
pub mod keyset;
pub mod lexer;
pub mod lsp;
pub mod modules;
pub mod parser;
pub mod profiler;
pub mod query;
pub mod runtime;
pub mod smartcontract_api;
pub mod stdlib;
pub mod testing;
pub mod transaction;
pub mod types;
pub mod upgrades;
pub mod value;
pub mod verification;
pub mod vm;

pub use chainweb::ChainwebNetwork;
pub use compiler::Compiler;
pub use debugger::Debugger;
pub use error::{SlvrError, SlvrResult};
pub use evaluator::Evaluator;
pub use lexer::Lexer;
pub use lsp::LspServer;
pub use parser::Parser;
pub use profiler::Profiler;
pub use runtime::Runtime;
pub use types::{Type, TypeEnv};
pub use value::Value;
pub use vm::VirtualMachine;

/// The version of the Slvr language
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Language name
pub const LANGUAGE_NAME: &str = "The Slvr Programming Language";

/// Language description
pub const LANGUAGE_DESCRIPTION: &str =
    "A Turing-incomplete smart contract language for the SilverBitcoin blockchain";

/// Maximum recursion depth to prevent stack overflow
pub const MAX_RECURSION_DEPTH: usize = 1024;

/// Maximum execution steps per transaction
pub const MAX_EXECUTION_STEPS: u64 = 10_000_000;

/// Minimum fuel per operation
pub const MIN_FUEL_PER_OP: u64 = 1;

/// Maximum fuel per transaction
pub const MAX_FUEL_PER_TX: u64 = 1_000_000_000;

/// Configuration for Slvr language execution
#[derive(Debug, Clone)]
pub struct SlvrConfig {
    /// Maximum recursion depth
    pub max_recursion_depth: usize,
    /// Maximum execution steps
    pub max_execution_steps: u64,
    /// Enable gas/fuel metering
    pub enable_fuel_metering: bool,
    /// Maximum fuel per transaction
    pub max_fuel_per_tx: u64,
    /// Enable type checking
    pub enable_type_checking: bool,
    /// Enable optimization passes
    pub enable_optimization: bool,
}

impl Default for SlvrConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: MAX_RECURSION_DEPTH,
            max_execution_steps: MAX_EXECUTION_STEPS,
            enable_fuel_metering: true,
            max_fuel_per_tx: MAX_FUEL_PER_TX,
            enable_type_checking: true,
            enable_optimization: true,
        }
    }
}

impl SlvrConfig {
    /// Create a new configuration with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Set maximum execution steps
    pub fn with_max_execution_steps(mut self, steps: u64) -> Self {
        self.max_execution_steps = steps;
        self
    }

    /// Enable or disable fuel metering
    pub fn with_fuel_metering(mut self, enabled: bool) -> Self {
        self.enable_fuel_metering = enabled;
        self
    }

    /// Set maximum fuel per transaction
    pub fn with_max_fuel_per_tx(mut self, fuel: u64) -> Self {
        self.max_fuel_per_tx = fuel;
        self
    }

    /// Enable or disable type checking
    pub fn with_type_checking(mut self, enabled: bool) -> Self {
        self.enable_type_checking = enabled;
        self
    }

    /// Enable or disable optimization
    pub fn with_optimization(mut self, enabled: bool) -> Self {
        self.enable_optimization = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_language_name() {
        assert!(LANGUAGE_NAME.contains("Slvr"));
    }

    #[test]
    fn test_default_config() {
        let config = SlvrConfig::default();
        assert_eq!(config.max_recursion_depth, MAX_RECURSION_DEPTH);
        assert_eq!(config.max_execution_steps, MAX_EXECUTION_STEPS);
        assert!(config.enable_fuel_metering);
        assert!(config.enable_type_checking);
    }

    #[test]
    fn test_config_builder() {
        let config = SlvrConfig::new()
            .with_max_recursion_depth(512)
            .with_fuel_metering(false);

        assert_eq!(config.max_recursion_depth, 512);
        assert!(!config.enable_fuel_metering);
    }
}
