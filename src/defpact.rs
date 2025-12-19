//! Multi-Step Transactions (Defpact) Support
//!
//! This module provides support for multi-step transactions (defpact) which allow
//! contracts to execute in multiple steps across different blocks, with state
//! persistence between steps.

use crate::error::{SlvrError, SlvrResult};
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a step in a multi-step transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactStep {
    /// Unique step identifier
    pub id: String,
    /// Step number (0-indexed)
    pub step_number: usize,
    /// Total number of steps in this pact
    pub total_steps: usize,
    /// Step name/identifier
    pub name: String,
    /// Step execution status
    pub status: PactStepStatus,
    /// Input parameters for this step
    pub inputs: HashMap<String, Value>,
    /// Output/result from this step
    pub output: Option<Value>,
    /// Timestamp when step was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when step was executed
    pub executed_at: Option<DateTime<Utc>>,
    /// Error message if step failed
    pub error: Option<String>,
    /// Fuel consumed by this step
    pub fuel_consumed: u64,
}

/// Status of a pact step
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PactStepStatus {
    /// Step is waiting to be executed
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed with an error
    Failed,
    /// Step was rolled back
    RolledBack,
}

impl std::fmt::Display for PactStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PactStepStatus::Pending => write!(f, "pending"),
            PactStepStatus::Running => write!(f, "running"),
            PactStepStatus::Completed => write!(f, "completed"),
            PactStepStatus::Failed => write!(f, "failed"),
            PactStepStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Represents a multi-step transaction (pact)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pact {
    /// Unique pact identifier
    pub id: String,
    /// Pact name
    pub name: String,
    /// Contract that defined this pact
    pub contract: String,
    /// Function that defined this pact
    pub function: String,
    /// All steps in this pact
    pub steps: Vec<PactStep>,
    /// Current step index
    pub current_step: usize,
    /// Overall pact status
    pub status: PactStatus,
    /// Timestamp when pact was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when pact was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Shared state across all steps
    pub shared_state: HashMap<String, Value>,
    /// Total fuel consumed by all steps
    pub total_fuel_consumed: u64,
    /// Maximum fuel allowed for entire pact
    pub max_fuel: u64,
    /// Yield value from previous step (for continuation)
    pub yield_value: Option<Value>,
}

/// Overall status of a pact
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PactStatus {
    /// Pact is waiting to start
    Pending,
    /// Pact is currently executing
    Running,
    /// Pact completed successfully
    Completed,
    /// Pact failed
    Failed,
    /// Pact was rolled back
    RolledBack,
}

impl std::fmt::Display for PactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PactStatus::Pending => write!(f, "pending"),
            PactStatus::Running => write!(f, "running"),
            PactStatus::Completed => write!(f, "completed"),
            PactStatus::Failed => write!(f, "failed"),
            PactStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Pact execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactContext {
    /// Current pact being executed
    pub pact: Pact,
    /// Execution history
    pub history: Vec<PactExecutionRecord>,
}

/// Record of a pact execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactExecutionRecord {
    /// Block height where step was executed
    pub block_height: u64,
    /// Block hash
    pub block_hash: String,
    /// Step that was executed
    pub step: PactStep,
    /// Continuation data for next step
    pub continuation: Option<Value>,
}

/// Pact manager for handling multi-step transactions
#[derive(Debug, Clone)]
pub struct PactManager {
    /// Active pacts indexed by ID
    pacts: HashMap<String, Pact>,
    /// Pact execution history
    history: Vec<PactExecutionRecord>,
}

impl PactManager {
    /// Create a new pact manager
    pub fn new() -> Self {
        Self {
            pacts: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Create a new pact
    pub fn create_pact(
        &mut self,
        name: String,
        contract: String,
        function: String,
        steps: Vec<String>,
        max_fuel: u64,
    ) -> SlvrResult<String> {
        let pact_id = Uuid::new_v4().to_string();
        
        let total_steps = steps.len();
        let pact_steps: Vec<PactStep> = steps
            .into_iter()
            .enumerate()
            .map(|(idx, step_name)| PactStep {
                id: Uuid::new_v4().to_string(),
                step_number: idx,
                total_steps,
                name: step_name,
                status: PactStepStatus::Pending,
                inputs: HashMap::new(),
                output: None,
                created_at: Utc::now(),
                executed_at: None,
                error: None,
                fuel_consumed: 0,
            })
            .collect();

        let pact = Pact {
            id: pact_id.clone(),
            name,
            contract,
            function,
            steps: pact_steps,
            current_step: 0,
            status: PactStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            shared_state: HashMap::new(),
            total_fuel_consumed: 0,
            max_fuel,
            yield_value: None,
        };

        self.pacts.insert(pact_id.clone(), pact);
        Ok(pact_id)
    }

    /// Get a pact by ID
    pub fn get_pact(&self, pact_id: &str) -> SlvrResult<Pact> {
        self.pacts
            .get(pact_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Pact not found: {}", pact_id),
            })
    }

    /// Execute the next step in a pact
    pub fn execute_next_step(
        &mut self,
        pact_id: &str,
        inputs: HashMap<String, Value>,
        fuel_limit: u64,
    ) -> SlvrResult<Value> {
        let mut pact = self.get_pact(pact_id)?;

        // Check if pact is already completed
        if pact.status == PactStatus::Completed {
            return Err(SlvrError::RuntimeError {
                message: "Pact already completed".to_string(),
            });
        }

        // Check fuel limit
        if pact.total_fuel_consumed + fuel_limit > pact.max_fuel {
            return Err(SlvrError::FuelExceeded {
                used: pact.total_fuel_consumed + fuel_limit,
                limit: pact.max_fuel,
            });
        }

        // Get current step
        if pact.current_step >= pact.steps.len() {
            return Err(SlvrError::RuntimeError {
                message: "No more steps to execute".to_string(),
            });
        }

        let mut step = pact.steps[pact.current_step].clone();
        step.status = PactStepStatus::Running;
        step.inputs = inputs;
        step.executed_at = Some(Utc::now());

        // Simulate step execution (in real implementation, this would call the actual step function)
        let output = Value::String(format!("Step {} executed", step.step_number));
        step.output = Some(output.clone());
        step.status = PactStepStatus::Completed;
        step.fuel_consumed = fuel_limit;

        // Update pact
        pact.steps[pact.current_step] = step.clone();
        pact.total_fuel_consumed += fuel_limit;
        pact.yield_value = Some(output.clone());

        // Move to next step or complete
        pact.current_step += 1;
        if pact.current_step >= pact.steps.len() {
            pact.status = PactStatus::Completed;
            pact.completed_at = Some(Utc::now());
        } else {
            pact.status = PactStatus::Running;
        }

        // Record execution
        let record = PactExecutionRecord {
            block_height: 0,
            block_hash: String::new(),
            step: step.clone(),
            continuation: Some(output.clone()),
        };
        self.history.push(record);

        // Update pact in manager
        self.pacts.insert(pact_id.to_string(), pact);

        Ok(output)
    }

    /// Rollback a pact to a previous step
    pub fn rollback_pact(&mut self, pact_id: &str, target_step: usize) -> SlvrResult<()> {
        let mut pact = self.get_pact(pact_id)?;

        if target_step >= pact.steps.len() {
            return Err(SlvrError::RuntimeError {
                message: "Invalid target step".to_string(),
            });
        }

        // Mark all steps after target as rolled back
        for i in target_step..pact.steps.len() {
            pact.steps[i].status = PactStepStatus::RolledBack;
        }

        pact.current_step = target_step;
        pact.status = PactStatus::RolledBack;

        self.pacts.insert(pact_id.to_string(), pact);
        Ok(())
    }

    /// Get execution history
    pub fn get_history(&self) -> Vec<PactExecutionRecord> {
        self.history.clone()
    }

    /// Get all active pacts
    pub fn get_active_pacts(&self) -> Vec<Pact> {
        self.pacts
            .values()
            .filter(|p| p.status == PactStatus::Running || p.status == PactStatus::Pending)
            .cloned()
            .collect()
    }

    /// Get pact statistics
    pub fn get_pact_stats(&self, pact_id: &str) -> SlvrResult<PactStats> {
        let pact = self.get_pact(pact_id)?;

        let completed_steps = pact.steps.iter().filter(|s| s.status == PactStepStatus::Completed).count();
        let failed_steps = pact.steps.iter().filter(|s| s.status == PactStepStatus::Failed).count();
        let total_fuel = pact.steps.iter().map(|s| s.fuel_consumed).sum();

        Ok(PactStats {
            pact_id: pact_id.to_string(),
            total_steps: pact.steps.len(),
            completed_steps,
            failed_steps,
            current_step: pact.current_step,
            total_fuel_consumed: total_fuel,
            max_fuel: pact.max_fuel,
            status: pact.status,
            created_at: pact.created_at,
            completed_at: pact.completed_at,
        })
    }
}

/// Statistics for a pact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactStats {
    pub pact_id: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub current_step: usize,
    pub total_fuel_consumed: u64,
    pub max_fuel: u64,
    pub status: PactStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pact_creation() {
        let mut manager = PactManager::new();
        let pact_id = manager
            .create_pact(
                "test_pact".to_string(),
                "test_contract".to_string(),
                "test_function".to_string(),
                vec!["step1".to_string(), "step2".to_string(), "step3".to_string()],
                1_000_000,
            )
            .unwrap();

        let pact = manager.get_pact(&pact_id).unwrap();
        assert_eq!(pact.steps.len(), 3);
        assert_eq!(pact.status, PactStatus::Pending);
        assert_eq!(pact.current_step, 0);
    }

    #[test]
    fn test_pact_step_execution() {
        let mut manager = PactManager::new();
        let pact_id = manager
            .create_pact(
                "test_pact".to_string(),
                "test_contract".to_string(),
                "test_function".to_string(),
                vec!["step1".to_string(), "step2".to_string()],
                1_000_000,
            )
            .unwrap();

        let result = manager
            .execute_next_step(&pact_id, HashMap::new(), 10_000)
            .unwrap();

        assert!(matches!(result, Value::String(_)));

        let pact = manager.get_pact(&pact_id).unwrap();
        assert_eq!(pact.current_step, 1);
        assert_eq!(pact.status, PactStatus::Running);
    }

    #[test]
    fn test_pact_completion() {
        let mut manager = PactManager::new();
        let pact_id = manager
            .create_pact(
                "test_pact".to_string(),
                "test_contract".to_string(),
                "test_function".to_string(),
                vec!["step1".to_string()],
                1_000_000,
            )
            .unwrap();

        manager
            .execute_next_step(&pact_id, HashMap::new(), 10_000)
            .unwrap();

        let pact = manager.get_pact(&pact_id).unwrap();
        assert_eq!(pact.status, PactStatus::Completed);
        assert!(pact.completed_at.is_some());
    }

    #[test]
    fn test_pact_rollback() {
        let mut manager = PactManager::new();
        let pact_id = manager
            .create_pact(
                "test_pact".to_string(),
                "test_contract".to_string(),
                "test_function".to_string(),
                vec!["step1".to_string(), "step2".to_string(), "step3".to_string()],
                1_000_000,
            )
            .unwrap();

        manager
            .execute_next_step(&pact_id, HashMap::new(), 10_000)
            .unwrap();
        manager
            .execute_next_step(&pact_id, HashMap::new(), 10_000)
            .unwrap();

        manager.rollback_pact(&pact_id, 0).unwrap();

        let pact = manager.get_pact(&pact_id).unwrap();
        assert_eq!(pact.status, PactStatus::RolledBack);
        assert_eq!(pact.current_step, 0);
    }

    #[test]
    fn test_pact_stats() {
        let mut manager = PactManager::new();
        let pact_id = manager
            .create_pact(
                "test_pact".to_string(),
                "test_contract".to_string(),
                "test_function".to_string(),
                vec!["step1".to_string(), "step2".to_string()],
                1_000_000,
            )
            .unwrap();

        manager
            .execute_next_step(&pact_id, HashMap::new(), 10_000)
            .unwrap();

        let stats = manager.get_pact_stats(&pact_id).unwrap();
        assert_eq!(stats.total_steps, 2);
        assert_eq!(stats.completed_steps, 1);
        assert_eq!(stats.current_step, 1);
    }
}
