//! Runtime environment for the Slvr language
//!
//! Manages execution state, fuel metering, and database operations.

use crate::error::{SlvrError, SlvrResult};
use crate::value::Value;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Runtime environment for Slvr execution
pub struct Runtime {
    /// Global state/database (thread-safe)
    state: Arc<DashMap<String, Value>>,
    /// Fuel remaining (atomic for thread safety)
    fuel: Arc<AtomicU64>,
    /// Maximum fuel
    max_fuel: u64,
    /// Execution start time
    start_time: SystemTime,
    /// Transaction ID
    tx_id: String,
    /// Execution context
    context: ExecutionContext,
}

/// Execution context information
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Current caller address
    pub caller: String,
    /// Current transaction hash
    pub tx_hash: String,
    /// Block height
    pub block_height: u64,
    /// Block timestamp
    pub block_timestamp: u64,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            caller: "system".to_string(),
            tx_hash: "0x0".to_string(),
            block_height: 0,
            block_timestamp: 0,
        }
    }
}

impl Runtime {
    /// Create a new runtime with default fuel
    pub fn new(max_fuel: u64) -> Self {
        Self::with_context(max_fuel, ExecutionContext::default())
    }

    /// Create a new runtime with execution context
    pub fn with_context(max_fuel: u64, context: ExecutionContext) -> Self {
        Self {
            state: Arc::new(DashMap::new()),
            fuel: Arc::new(AtomicU64::new(max_fuel)),
            max_fuel,
            start_time: SystemTime::now(),
            tx_id: format!("tx_{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            context,
        }
    }

    /// Get remaining fuel
    pub fn fuel(&self) -> u64 {
        self.fuel.load(Ordering::SeqCst)
    }

    /// Get used fuel
    pub fn fuel_used(&self) -> u64 {
        self.max_fuel - self.fuel()
    }

    /// Get fuel percentage used
    pub fn fuel_percentage(&self) -> f64 {
        (self.fuel_used() as f64 / self.max_fuel as f64) * 100.0
    }

    /// Consume fuel
    pub fn consume_fuel(&self, amount: u64) -> SlvrResult<()> {
        let current = self.fuel.load(Ordering::SeqCst);
        if current < amount {
            return Err(SlvrError::FuelExceeded {
                used: self.fuel_used(),
                limit: self.max_fuel,
            });
        }
        self.fuel.fetch_sub(amount, Ordering::SeqCst);
        Ok(())
    }

    /// Get execution time in milliseconds
    pub fn execution_time_ms(&self) -> u128 {
        self.start_time
            .elapsed()
            .unwrap_or_default()
            .as_millis()
    }

    /// Get transaction ID
    pub fn tx_id(&self) -> &str {
        &self.tx_id
    }

    /// Get execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Read from state
    pub fn read(&self, key: &str) -> Option<Value> {
        self.state.get(key).map(|v| v.clone())
    }

    /// Read with default value
    pub fn read_or(&self, key: &str, default: Value) -> Value {
        self.read(key).unwrap_or(default)
    }

    /// Write to state
    pub fn write(&self, key: String, value: Value) -> SlvrResult<()> {
        self.consume_fuel(100)?; // Fuel cost for write operation
        self.state.insert(key, value);
        Ok(())
    }

    /// Update existing value
    pub fn update(&self, key: &str, value: Value) -> SlvrResult<Option<Value>> {
        self.consume_fuel(100)?; // Fuel cost for update operation
        Ok(self.state.insert(key.to_string(), value))
    }

    /// Delete from state
    pub fn delete(&self, key: &str) -> SlvrResult<Option<Value>> {
        self.consume_fuel(50)?; // Fuel cost for delete operation
        Ok(self.state.remove(key).map(|(_, v)| v))
    }

    /// Check if key exists
    pub fn exists(&self, key: &str) -> bool {
        self.state.contains_key(key)
    }

    /// Get all keys matching pattern
    pub fn keys_matching(&self, pattern: &str) -> Vec<String> {
        self.state
            .iter()
            .filter(|entry| entry.key().contains(pattern))
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get state size
    pub fn state_size(&self) -> usize {
        self.state.len()
    }

    /// Clear all state
    pub fn clear_state(&self) {
        self.state.clear();
    }

    /// Get state snapshot
    pub fn snapshot(&self) -> Vec<(String, Value)> {
        self.state
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Restore from snapshot
    pub fn restore(&self, snapshot: Vec<(String, Value)>) {
        self.state.clear();
        for (key, value) in snapshot {
            self.state.insert(key, value);
        }
    }

    /// Get execution statistics
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            fuel_used: self.fuel_used(),
            fuel_remaining: self.fuel(),
            fuel_total: self.max_fuel,
            execution_time_ms: self.execution_time_ms(),
            state_size: self.state_size(),
            tx_id: self.tx_id.clone(),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(1_000_000_000)
    }
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            fuel: Arc::clone(&self.fuel),
            max_fuel: self.max_fuel,
            start_time: self.start_time,
            tx_id: self.tx_id.clone(),
            context: self.context.clone(),
        }
    }
}

/// Runtime execution statistics
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub fuel_used: u64,
    pub fuel_remaining: u64,
    pub fuel_total: u64,
    pub execution_time_ms: u128,
    pub state_size: usize,
    pub tx_id: String,
}

impl std::fmt::Display for RuntimeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Runtime Stats:\n  Fuel: {}/{} ({:.2}%)\n  Time: {}ms\n  State Size: {} entries\n  TX ID: {}",
            self.fuel_used,
            self.fuel_total,
            (self.fuel_used as f64 / self.fuel_total as f64) * 100.0,
            self.execution_time_ms,
            self.state_size,
            self.tx_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new(1_000_000);
        assert_eq!(runtime.fuel(), 1_000_000);
        assert_eq!(runtime.fuel_used(), 0);
    }

    #[test]
    fn test_fuel_consumption() {
        let runtime = Runtime::new(1_000);
        match runtime.consume_fuel(100) {
            Ok(_) => {
                assert_eq!(runtime.fuel(), 900);
                assert_eq!(runtime.fuel_used(), 100);
            }
            Err(e) => panic!("Fuel consumption failed: {}", e),
        }
    }

    #[test]
    fn test_fuel_exceeded() {
        let runtime = Runtime::new(100);
        let result = runtime.consume_fuel(200);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_operations() {
        let runtime = Runtime::new(1_000_000);
        
        match runtime.write("key1".to_string(), Value::Integer(42)) {
            Ok(_) => {
                assert_eq!(runtime.read("key1"), Some(Value::Integer(42)));
                
                match runtime.delete("key1") {
                    Ok(_) => assert_eq!(runtime.read("key1"), None),
                    Err(e) => panic!("Delete operation failed: {}", e),
                }
            }
            Err(e) => panic!("Write operation failed: {}", e),
        }
    }

    #[test]
    fn test_state_snapshot() {
        let runtime = Runtime::new(1_000_000);
        
        match runtime.write("key1".to_string(), Value::Integer(1)) {
            Ok(_) => {
                match runtime.write("key2".to_string(), Value::Integer(2)) {
                    Ok(_) => {
                        let snapshot = runtime.snapshot();
                        assert_eq!(snapshot.len(), 2);
                        
                        runtime.clear_state();
                        assert_eq!(runtime.state_size(), 0);
                        
                        runtime.restore(snapshot);
                        assert_eq!(runtime.state_size(), 2);
                    }
                    Err(e) => panic!("Second write failed: {}", e),
                }
            }
            Err(e) => panic!("First write failed: {}", e),
        }
    }

    #[test]
    fn test_runtime_stats() {
        let runtime = Runtime::new(1_000_000);
        match runtime.consume_fuel(100_000) {
            Ok(_) => {
                let stats = runtime.stats();
                assert_eq!(stats.fuel_used, 100_000);
                assert_eq!(stats.fuel_remaining, 900_000);
            }
            Err(e) => panic!("Fuel consumption failed: {}", e),
        }
    }
}
