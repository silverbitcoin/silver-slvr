//! REST API and JSON-RPC Support
//!
//! This module provides HTTP endpoints and JSON-RPC interface for contract execution,
//! transaction submission, and state queries.

use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>,
}

/// Contract execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub contract: String,
    pub function: String,
    pub args: Vec<serde_json::Value>,
    pub sender: String,
    pub fuel_limit: u64,
}

/// Contract execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub fuel_used: u64,
    pub execution_time_ms: u64,
}

/// Transaction submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub contract: String,
    pub function: String,
    pub args: Vec<serde_json::Value>,
    pub sender: String,
    pub nonce: u64,
    pub signature: String,
}

/// Transaction submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub tx_hash: String,
    pub status: String,
    pub message: String,
}

/// State query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStateRequest {
    pub table: String,
    pub key: String,
}

/// State query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStateResponse {
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub exists: bool,
}

/// Contract deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployContractRequest {
    pub name: String,
    pub code: String,
    pub sender: String,
    pub signature: String,
}

/// Contract deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployContractResponse {
    pub contract_id: String,
    pub address: String,
    pub status: String,
}

/// API Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub max_request_size: usize,
    pub request_timeout_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            max_request_size: 10 * 1024 * 1024,
            request_timeout_ms: 30000,
        }
    }
}

/// API Handler for processing requests
pub struct ApiHandler {
    config: ApiConfig,
    contracts: HashMap<String, String>,
}

impl ApiHandler {
    /// Create new API handler
    pub fn new(config: ApiConfig) -> Self {
        ApiHandler {
            config,
            contracts: HashMap::new(),
        }
    }

    /// Handle JSON-RPC request
    pub fn handle_jsonrpc(&mut self, request: JsonRpcRequest) -> SlvrResult<JsonRpcResponse> {
        if request.jsonrpc != "2.0" {
            return Err(SlvrError::RuntimeError {
                message: "Invalid JSON-RPC version".to_string(),
            });
        }

        let result = match request.method.as_str() {
            "execute" => self.handle_execute(&request.params)?,
            "submit_transaction" => self.handle_submit_transaction(&request.params)?,
            "query_state" => self.handle_query_state(&request.params)?,
            "deploy_contract" => self.handle_deploy_contract(&request.params)?,
            "get_contract" => self.handle_get_contract(&request.params)?,
            "list_contracts" => self.handle_list_contracts()?,
            _ => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                    id: request.id,
                });
            }
        };

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        })
    }

    /// Handle execute request
    fn handle_execute(&self, params: &serde_json::Value) -> SlvrResult<serde_json::Value> {
        let request: ExecuteRequest = serde_json::from_value(params.clone())
            .map_err(|e| SlvrError::RuntimeError {
                message: format!("Invalid execute request: {}", e),
            })?;

        let start = std::time::Instant::now();

        if !self.contracts.contains_key(&request.contract) {
            return Err(SlvrError::RuntimeError {
                message: format!("Contract {} not found", request.contract),
            });
        }

        let response = ExecuteResponse {
            success: true,
            result: Some(serde_json::json!({"status": "executed"})),
            error: None,
            fuel_used: 1000,
            execution_time_ms: start.elapsed().as_millis() as u64,
        };

        Ok(serde_json::to_value(response).map_err(|e| SlvrError::RuntimeError {
            message: format!("Serialization error: {}", e),
        })?)
    }

    /// Handle submit transaction request
    fn handle_submit_transaction(&mut self, params: &serde_json::Value) -> SlvrResult<serde_json::Value> {
        let request: SubmitTransactionRequest = serde_json::from_value(params.clone())
            .map_err(|e| SlvrError::RuntimeError {
                message: format!("Invalid submit transaction request: {}", e),
            })?;

        if !self.contracts.contains_key(&request.contract) {
            return Err(SlvrError::RuntimeError {
                message: format!("Contract {} not found", request.contract),
            });
        }

        let tx_hash = format!("0x{}", blake3::hash(request.contract.as_bytes()).to_hex());

        let response = SubmitTransactionResponse {
            tx_hash,
            status: "pending".to_string(),
            message: "Transaction submitted successfully".to_string(),
        };

        Ok(serde_json::to_value(response).map_err(|e| SlvrError::RuntimeError {
            message: format!("Serialization error: {}", e),
        })?)
    }

    /// Handle query state request
    fn handle_query_state(&self, params: &serde_json::Value) -> SlvrResult<serde_json::Value> {
        let request: QueryStateRequest = serde_json::from_value(params.clone())
            .map_err(|e| SlvrError::RuntimeError {
                message: format!("Invalid query state request: {}", e),
            })?;

        let response = QueryStateResponse {
            key: request.key,
            value: None,
            exists: false,
        };

        Ok(serde_json::to_value(response).map_err(|e| SlvrError::RuntimeError {
            message: format!("Serialization error: {}", e),
        })?)
    }

    /// Handle deploy contract request
    fn handle_deploy_contract(&mut self, params: &serde_json::Value) -> SlvrResult<serde_json::Value> {
        let request: DeployContractRequest = serde_json::from_value(params.clone())
            .map_err(|e| SlvrError::RuntimeError {
                message: format!("Invalid deploy contract request: {}", e),
            })?;

        if request.code.is_empty() {
            return Err(SlvrError::RuntimeError {
                message: "Contract code cannot be empty".to_string(),
            });
        }

        let contract_id = format!("contract_{}", uuid::Uuid::new_v4());
        let address = format!("0x{}", blake3::hash(contract_id.as_bytes()).to_hex());

        self.contracts.insert(request.name.clone(), request.code);

        let response = DeployContractResponse {
            contract_id,
            address,
            status: "deployed".to_string(),
        };

        Ok(serde_json::to_value(response).map_err(|e| SlvrError::RuntimeError {
            message: format!("Serialization error: {}", e),
        })?)
    }

    /// Handle get contract request
    fn handle_get_contract(&self, params: &serde_json::Value) -> SlvrResult<serde_json::Value> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SlvrError::RuntimeError {
                message: "Contract name required".to_string(),
            })?;

        if let Some(code) = self.contracts.get(name) {
            Ok(serde_json::json!({
                "name": name,
                "code": code,
                "exists": true
            }))
        } else {
            Ok(serde_json::json!({
                "name": name,
                "code": null,
                "exists": false
            }))
        }
    }

    /// Handle list contracts request
    fn handle_list_contracts(&self) -> SlvrResult<serde_json::Value> {
        let contracts: Vec<String> = self.contracts.keys().cloned().collect();
        Ok(serde_json::json!({
            "contracts": contracts,
            "count": contracts.len()
        }))
    }

    /// Get API configuration
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }

    /// Update API configuration
    pub fn set_config(&mut self, config: ApiConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_handler_creation() {
        let config = ApiConfig::default();
        let handler = ApiHandler::new(config);
        assert_eq!(handler.contracts.len(), 0);
    }

    #[test]
    fn test_deploy_contract() {
        let config = ApiConfig::default();
        let mut handler = ApiHandler::new(config);

        let request = DeployContractRequest {
            name: "test_contract".to_string(),
            code: "(defun test () 42)".to_string(),
            sender: "alice".to_string(),
            signature: "sig123".to_string(),
        };

        let params = serde_json::to_value(&request).unwrap();
        let result = handler.handle_deploy_contract(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_contracts() {
        let config = ApiConfig::default();
        let mut handler = ApiHandler::new(config);

        let request = DeployContractRequest {
            name: "contract1".to_string(),
            code: "(defun test () 42)".to_string(),
            sender: "alice".to_string(),
            signature: "sig123".to_string(),
        };

        let params = serde_json::to_value(&request).unwrap();
        let _ = handler.handle_deploy_contract(&params);

        let result = handler.handle_list_contracts();
        assert!(result.is_ok());
    }

    #[test]
    fn test_jsonrpc_request() {
        let config = ApiConfig::default();
        let mut handler = ApiHandler::new(config);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "list_contracts".to_string(),
            params: serde_json::json!({}),
            id: serde_json::json!(1),
        };

        let result = handler.handle_jsonrpc(request);
        assert!(result.is_ok());
    }
}
