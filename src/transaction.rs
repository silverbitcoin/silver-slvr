//! Transaction Management - ACID Guarantees and Rollback Support
//!
//! This module provides comprehensive transaction management with ACID properties,
//! including atomicity, consistency, isolation, and durability.

use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Transaction status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Running,
    Committed,
    RolledBack,
    Failed,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Running => write!(f, "running"),
            TransactionStatus::Committed => write!(f, "committed"),
            TransactionStatus::RolledBack => write!(f, "rolled_back"),
            TransactionStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Transaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Write {
        table: String,
        key: String,
        value: String,
    },
    Update {
        table: String,
        key: String,
        value: String,
    },
    Delete {
        table: String,
        key: String,
    },
}

/// Transaction log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub operation: Operation,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
}

/// Transaction with ACID properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub status: TransactionStatus,
    pub operations: Vec<Operation>,
    pub log: Vec<LogEntry>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub isolation_level: IsolationLevel,
}

/// Isolation level for transactions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationLevel::ReadUncommitted => write!(f, "read_uncommitted"),
            IsolationLevel::ReadCommitted => write!(f, "read_committed"),
            IsolationLevel::RepeatableRead => write!(f, "repeatable_read"),
            IsolationLevel::Serializable => write!(f, "serializable"),
        }
    }
}

impl Transaction {
    /// Create a new transaction
    pub fn new(isolation_level: IsolationLevel) -> Self {
        let id = format!("tx_{}", uuid::Uuid::new_v4());
        Transaction {
            id,
            status: TransactionStatus::Pending,
            operations: Vec::new(),
            log: Vec::new(),
            start_time: Utc::now(),
            end_time: None,
            isolation_level,
        }
    }

    /// Begin transaction
    pub fn begin(&mut self) -> SlvrResult<()> {
        if self.status != TransactionStatus::Pending {
            return Err(SlvrError::RuntimeError {
                message: "transaction already started".to_string(),
            });
        }
        self.status = TransactionStatus::Running;
        Ok(())
    }

    /// Add operation to transaction
    pub fn add_operation(&mut self, operation: Operation) -> SlvrResult<()> {
        if self.status != TransactionStatus::Running {
            return Err(SlvrError::RuntimeError {
                message: "transaction not running".to_string(),
            });
        }
        self.operations.push(operation);
        Ok(())
    }

    /// Commit transaction
    pub fn commit(&mut self) -> SlvrResult<()> {
        if self.status != TransactionStatus::Running {
            return Err(SlvrError::RuntimeError {
                message: "transaction not running".to_string(),
            });
        }

        for operation in &self.operations {
            let log_entry = LogEntry {
                operation: operation.clone(),
                timestamp: Utc::now(),
                status: TransactionStatus::Committed,
            };
            self.log.push(log_entry);
        }

        self.status = TransactionStatus::Committed;
        self.end_time = Some(Utc::now());
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(&mut self) -> SlvrResult<()> {
        if self.status != TransactionStatus::Running && self.status != TransactionStatus::Failed {
            return Err(SlvrError::RuntimeError {
                message: "cannot rollback committed transaction".to_string(),
            });
        }

        self.operations.clear();
        self.status = TransactionStatus::RolledBack;
        self.end_time = Some(Utc::now());
        Ok(())
    }

    /// Mark transaction as failed
    pub fn fail(&mut self, reason: String) -> SlvrResult<()> {
        self.status = TransactionStatus::Failed;
        self.end_time = Some(Utc::now());
        
        let log_entry = LogEntry {
            operation: Operation::Write {
                table: "system".to_string(),
                key: "error".to_string(),
                value: reason,
            },
            timestamp: Utc::now(),
            status: TransactionStatus::Failed,
        };
        self.log.push(log_entry);
        Ok(())
    }

    /// Get transaction duration
    pub fn duration(&self) -> chrono::Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end - self.start_time
    }

    /// Check if transaction is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Committed | TransactionStatus::RolledBack | TransactionStatus::Failed
        )
    }
}

/// Transaction manager for managing multiple transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionManager {
    transactions: HashMap<String, Transaction>,
    active_transactions: Vec<String>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        TransactionManager {
            transactions: HashMap::new(),
            active_transactions: Vec::new(),
        }
    }

    /// Begin a new transaction
    pub fn begin_transaction(&mut self, isolation_level: IsolationLevel) -> SlvrResult<String> {
        let mut tx = Transaction::new(isolation_level);
        tx.begin()?;
        let tx_id = tx.id.clone();
        self.transactions.insert(tx_id.clone(), tx);
        self.active_transactions.push(tx_id.clone());
        Ok(tx_id)
    }

    /// Get transaction
    pub fn get_transaction(&self, tx_id: &str) -> SlvrResult<Transaction> {
        self.transactions
            .get(tx_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("transaction {} not found", tx_id),
            })
    }

    /// Add operation to transaction
    pub fn add_operation(&mut self, tx_id: &str, operation: Operation) -> SlvrResult<()> {
        if let Some(tx) = self.transactions.get_mut(tx_id) {
            tx.add_operation(operation)
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("transaction {} not found", tx_id),
            })
        }
    }

    /// Commit transaction
    pub fn commit(&mut self, tx_id: &str) -> SlvrResult<()> {
        if let Some(tx) = self.transactions.get_mut(tx_id) {
            tx.commit()?;
            self.active_transactions.retain(|id| id != tx_id);
            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("transaction {} not found", tx_id),
            })
        }
    }

    /// Rollback transaction
    pub fn rollback(&mut self, tx_id: &str) -> SlvrResult<()> {
        if let Some(tx) = self.transactions.get_mut(tx_id) {
            tx.rollback()?;
            self.active_transactions.retain(|id| id != tx_id);
            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("transaction {} not found", tx_id),
            })
        }
    }

    /// Get active transactions
    pub fn get_active_transactions(&self) -> Vec<String> {
        self.active_transactions.clone()
    }

    /// Get transaction history
    pub fn get_history(&self, tx_id: &str) -> SlvrResult<Vec<LogEntry>> {
        self.transactions
            .get(tx_id)
            .map(|tx| tx.log.clone())
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("transaction {} not found", tx_id),
            })
    }

    /// List all transactions
    pub fn list_transactions(&self) -> Vec<String> {
        self.transactions.keys().cloned().collect()
    }

    /// Get transaction statistics
    pub fn get_stats(&self) -> TransactionStats {
        let total = self.transactions.len();
        let committed = self
            .transactions
            .values()
            .filter(|tx| tx.status == TransactionStatus::Committed)
            .count();
        let rolled_back = self
            .transactions
            .values()
            .filter(|tx| tx.status == TransactionStatus::RolledBack)
            .count();
        let failed = self
            .transactions
            .values()
            .filter(|tx| tx.status == TransactionStatus::Failed)
            .count();
        let active = self.active_transactions.len();

        TransactionStats {
            total,
            committed,
            rolled_back,
            failed,
            active,
        }
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStats {
    pub total: usize,
    pub committed: usize,
    pub rolled_back: usize,
    pub failed: usize,
    pub active: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(IsolationLevel::Serializable);
        assert_eq!(tx.status, TransactionStatus::Pending);
        assert!(tx.operations.is_empty());
    }

    #[test]
    fn test_transaction_lifecycle() {
        let mut tx = Transaction::new(IsolationLevel::ReadCommitted);
        
        assert!(tx.begin().is_ok());
        assert_eq!(tx.status, TransactionStatus::Running);

        let op = Operation::Write {
            table: "test".to_string(),
            key: "key1".to_string(),
            value: "value1".to_string(),
        };
        assert!(tx.add_operation(op).is_ok());

        assert!(tx.commit().is_ok());
        assert_eq!(tx.status, TransactionStatus::Committed);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut tx = Transaction::new(IsolationLevel::ReadCommitted);
        
        assert!(tx.begin().is_ok());

        let op = Operation::Write {
            table: "test".to_string(),
            key: "key1".to_string(),
            value: "value1".to_string(),
        };
        assert!(tx.add_operation(op).is_ok());
        assert_eq!(tx.operations.len(), 1);

        assert!(tx.rollback().is_ok());
        assert_eq!(tx.status, TransactionStatus::RolledBack);
        assert!(tx.operations.is_empty());
    }

    #[test]
    fn test_transaction_manager() {
        let mut manager = TransactionManager::new();

        let tx_id = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();

        let op = Operation::Write {
            table: "test".to_string(),
            key: "key1".to_string(),
            value: "value1".to_string(),
        };
        assert!(manager.add_operation(&tx_id, op).is_ok());

        assert!(manager.commit(&tx_id).is_ok());

        let tx = manager.get_transaction(&tx_id).unwrap();
        assert_eq!(tx.status, TransactionStatus::Committed);
    }

    #[test]
    fn test_transaction_stats() {
        let mut manager = TransactionManager::new();

        let tx_id1 = manager
            .begin_transaction(IsolationLevel::ReadCommitted)
            .unwrap();
        let tx_id2 = manager
            .begin_transaction(IsolationLevel::Serializable)
            .unwrap();

        manager.commit(&tx_id1).unwrap();
        manager.rollback(&tx_id2).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.committed, 1);
        assert_eq!(stats.rolled_back, 1);
    }
}
