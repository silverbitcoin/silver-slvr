//! Comprehensive API Handler for all three API categories
//!
//! This module provides a unified interface for handling requests across
//! Blockchain APIs, Smart Contract APIs, and Account APIs.

use crate::error::{SlvrError, SlvrResult};
use crate::blockchain_api::BlockchainState;
use crate::smartcontract_api::{ContractManager, DeploymentRequest, CallRequest};
use crate::account_api::AccountManager;
use crate::runtime::Runtime;
use chrono::Utc;

/// Unified API handler
pub struct ApiHandler {
    pub blockchain: BlockchainState,
    pub contracts: ContractManager,
    pub accounts: AccountManager,
    pub runtime: Runtime,
}

impl ApiHandler {
    /// Create new API handler
    pub fn new() -> Self {
        Self {
            blockchain: BlockchainState::new(),
            contracts: ContractManager::new(),
            accounts: AccountManager::new(),
            runtime: Runtime::new(1_000_000_000),
        }
    }

    // ============ BLOCKCHAIN API METHODS ============

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> SlvrResult<serde_json::Value> {
        let block = self.blockchain.get_block_by_height(height)?;
        serde_json::to_value(block).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> SlvrResult<serde_json::Value> {
        let block = self.blockchain.get_block_by_hash(hash)?;
        serde_json::to_value(block).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get transaction details
    pub fn get_transaction_details(&self, tx_hash: &str) -> SlvrResult<serde_json::Value> {
        let tx = self.blockchain.get_transaction(tx_hash)?;
        serde_json::to_value(tx).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get network status
    pub fn get_network_status(&self) -> SlvrResult<serde_json::Value> {
        let status = self.blockchain.get_network_status();
        serde_json::to_value(status).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get chain statistics
    pub fn get_chain_stats(&self) -> SlvrResult<serde_json::Value> {
        let stats = self.blockchain.get_chain_stats();
        serde_json::to_value(stats).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get blocks in range
    pub fn get_blocks_range(&self, start: u64, end: u64) -> SlvrResult<serde_json::Value> {
        let blocks = self.blockchain.get_blocks_range(start, end)?;
        serde_json::to_value(blocks).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get account transaction history
    pub fn get_account_transactions(&self, address: &str) -> SlvrResult<serde_json::Value> {
        let txs = self.blockchain.get_account_transactions(address)?;
        serde_json::to_value(txs).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    // ============ SMART CONTRACT API METHODS ============

    /// Deploy contract
    pub fn deploy_contract(
        &self,
        name: String,
        code: String,
        author: String,
        version: String,
    ) -> SlvrResult<serde_json::Value> {
        let request = DeploymentRequest {
            name,
            source_code: code,
            author,
            version,
            deployer: "system".to_string(),
        };

        let contract = self.contracts.deploy(request)?;
        serde_json::to_value(contract.metadata).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Call contract function
    pub fn call_contract_function(
        &self,
        contract_id: String,
        function: String,
        args: Vec<serde_json::Value>,
        caller: String,
    ) -> SlvrResult<serde_json::Value> {
        let request = CallRequest {
            contract_id,
            function,
            args,
            caller,
        };

        let result = self.contracts.call_function(&request, &self.runtime)?;
        serde_json::to_value(result).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Query contract state
    pub fn query_contract_state(
        &self,
        contract_id: String,
        table: String,
        key: String,
    ) -> SlvrResult<serde_json::Value> {
        let value = self.contracts.query_state(&contract_id, &table, &key)?;
        Ok(serde_json::json!({
            "contract_id": contract_id,
            "table": table,
            "key": key,
            "value": value
        }))
    }

    /// Get contract metadata
    pub fn get_contract_metadata(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let metadata = self.contracts.get_metadata(&contract_id)?;
        serde_json::to_value(metadata).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// List all contracts
    pub fn list_contracts(&self) -> SlvrResult<serde_json::Value> {
        let contracts = self.contracts.list_contracts();
        serde_json::to_value(contracts).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get contract functions
    pub fn get_contract_functions(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let functions = self.contracts.get_functions(&contract_id)?;
        serde_json::to_value(functions).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get contract schemas
    pub fn get_contract_schemas(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let schemas = self.contracts.get_schemas(&contract_id)?;
        serde_json::to_value(schemas).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get contract tables
    pub fn get_contract_tables(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let tables = self.contracts.get_tables(&contract_id)?;
        serde_json::to_value(tables).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get contract constants
    pub fn get_contract_constants(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let constants = self.contracts.get_constants(&contract_id)?;
        serde_json::to_value(constants).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get contract source code
    pub fn get_contract_source(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let source = self.contracts.get_source_code(&contract_id)?;
        Ok(serde_json::json!({
            "contract_id": contract_id,
            "source_code": source
        }))
    }

    /// Get contract module info
    pub fn get_contract_module_info(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let info = self.contracts.get_module_info(&contract_id)?;
        Ok(info)
    }

    /// Query table data
    pub fn query_table(
        &self,
        contract_id: String,
        table_name: String,
        key: String,
    ) -> SlvrResult<serde_json::Value> {
        let value = self.contracts.query_table(&contract_id, &table_name, &key)?;
        Ok(serde_json::json!({
            "contract_id": contract_id,
            "table": table_name,
            "key": key,
            "value": value
        }))
    }

    /// Write to table
    pub fn write_to_table(
        &self,
        contract_id: String,
        table_name: String,
        key: String,
        value: serde_json::Value,
    ) -> SlvrResult<serde_json::Value> {
        self.contracts.write_table(&contract_id, &table_name, key.clone(), value)?;
        Ok(serde_json::json!({
            "contract_id": contract_id,
            "table": table_name,
            "key": key,
            "status": "written"
        }))
    }

    /// Verify Slvr code
    pub fn verify_slvr_code(&self, code: String) -> SlvrResult<serde_json::Value> {
        self.contracts.verify_code(&code)?;
        Ok(serde_json::json!({
            "valid": true,
            "message": "Code is valid"
        }))
    }

    /// Get contract execution history
    pub fn get_contract_execution_history(&self, contract_id: String) -> SlvrResult<serde_json::Value> {
        let history = self.contracts.get_execution_history(&contract_id);
        serde_json::to_value(history).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    // ============ ACCOUNT API METHODS ============

    /// Get account balance
    pub fn get_account_balance(&self, address: String) -> SlvrResult<serde_json::Value> {
        let balance = self.accounts.get_balance(&address)?;
        Ok(serde_json::json!({
            "address": address,
            "balance": balance
        }))
    }

    /// Get transaction history
    pub fn get_transaction_history(&self, address: String) -> SlvrResult<serde_json::Value> {
        let history = self.accounts.get_transaction_history(&address)?;
        serde_json::to_value(history).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Estimate gas cost
    pub fn estimate_gas_cost(
        &self,
        data_size: usize,
        execution_complexity: u64,
    ) -> SlvrResult<serde_json::Value> {
        let estimate = self.accounts.estimate_gas(data_size, execution_complexity);
        serde_json::to_value(estimate).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Validate address
    pub fn validate_address(&self, address: String) -> SlvrResult<serde_json::Value> {
        self.accounts.validate_address(&address)?;
        Ok(serde_json::json!({
            "address": address,
            "valid": true
        }))
    }

    /// Get account statistics
    pub fn get_account_stats(&self, address: String) -> SlvrResult<serde_json::Value> {
        let stats = self.accounts.get_account_stats(&address)?;
        serde_json::to_value(stats).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Create account
    pub fn create_account(&self, public_key: String) -> SlvrResult<serde_json::Value> {
        let account = self.accounts.create_account(public_key)?;
        serde_json::to_value(account).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    /// Get account info
    pub fn get_account_info(&self, address: String) -> SlvrResult<serde_json::Value> {
        let account = self.accounts.get_account(&address)?;
        serde_json::to_value(account).map_err(|_| SlvrError::RuntimeError {
            message: "Serialization error".to_string(),
        })
    }

    // ============ UTILITY METHODS ============

    /// Get API statistics
    pub fn get_api_stats(&self) -> SlvrResult<serde_json::Value> {
        let blockchain_stats = self.blockchain.get_chain_stats();
        let contract_stats = self.contracts.get_stats();
        let runtime_stats = self.runtime.stats();

        Ok(serde_json::json!({
            "blockchain": {
                "total_blocks": blockchain_stats.total_blocks,
                "total_transactions": blockchain_stats.total_transactions,
                "total_accounts": blockchain_stats.total_accounts,
                "total_supply": blockchain_stats.total_supply,
            },
            "contracts": {
                "total_contracts": contract_stats.total_contracts,
                "total_functions": contract_stats.total_functions,
                "total_executions": contract_stats.total_executions,
                "successful_executions": contract_stats.successful_executions,
                "failed_executions": contract_stats.failed_executions,
            },
            "runtime": {
                "fuel_used": runtime_stats.fuel_used,
                "fuel_remaining": runtime_stats.fuel_remaining,
                "execution_time_ms": runtime_stats.execution_time_ms,
                "state_size": runtime_stats.state_size,
            }
        }))
    }

    /// Health check
    pub fn health_check(&self) -> SlvrResult<serde_json::Value> {
        Ok(serde_json::json!({
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339(),
            "version": crate::VERSION,
        }))
    }
}

impl Default for ApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ApiHandler {
    fn clone(&self) -> Self {
        Self {
            blockchain: self.blockchain.clone(),
            contracts: self.contracts.clone(),
            accounts: self.accounts.clone(),
            runtime: self.runtime.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_handler_creation() {
        let handler = ApiHandler::new();
        assert!(handler.health_check().is_ok());
    }

    #[test]
    fn test_get_network_status() {
        let handler = ApiHandler::new();
        let result = handler.get_network_status();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_chain_stats() {
        let handler = ApiHandler::new();
        let result = handler.get_chain_stats();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deploy_contract() {
        let handler = ApiHandler::new();
        let result = handler.deploy_contract(
            "test".to_string(),
            "module test \"Test module\" { defun test-fn () -> integer 42 }".to_string(),
            "author".to_string(),
            "1.0.0".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_account() {
        let handler = ApiHandler::new();
        let result = handler.create_account("pubkey".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address() {
        let handler = ApiHandler::new();
        let result = handler.validate_address("0x1234567890abcdef".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_gas() {
        let handler = ApiHandler::new();
        let result = handler.estimate_gas_cost(100, 50);
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_stats() {
        let handler = ApiHandler::new();
        let result = handler.get_api_stats();
        assert!(result.is_ok());
    }
}
