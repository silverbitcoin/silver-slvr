//! Contract Versioning and Upgrades
//!
//! This module provides support for contract versioning, upgrades, and governance
//! mechanisms to allow contracts to evolve while maintaining state consistency.

use crate::error::{SlvrError, SlvrResult};
use crate::value::Value;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a contract version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractVersion {
    /// Unique version identifier
    pub id: String,
    /// Contract name
    pub contract_name: String,
    /// Version number (semantic versioning)
    pub version: String,
    /// Contract code hash
    pub code_hash: String,
    /// Contract bytecode
    pub bytecode: Vec<u8>,
    /// Version description/changelog
    pub description: Option<String>,
    /// Timestamp when version was created
    pub created_at: DateTime<Utc>,
    /// Whether this version is active
    pub active: bool,
    /// Metadata associated with this version
    pub metadata: HashMap<String, String>,
}

/// Upgrade proposal for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// Unique proposal identifier
    pub id: String,
    /// Contract to be upgraded
    pub contract_name: String,
    /// Current version
    pub from_version: String,
    /// Proposed new version
    pub to_version: String,
    /// Upgrade description
    pub description: Option<String>,
    /// Proposal status
    pub status: ProposalStatus,
    /// Timestamp when proposal was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when proposal was executed
    pub executed_at: Option<DateTime<Utc>>,
    /// Votes in favor
    pub votes_for: u64,
    /// Votes against
    pub votes_against: u64,
    /// Voting deadline
    pub voting_deadline: DateTime<Utc>,
    /// Migration script (if needed)
    pub migration_script: Option<String>,
}

/// Status of an upgrade proposal
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalStatus {
    /// Proposal is open for voting
    Voting,
    /// Proposal passed and is pending execution
    Approved,
    /// Proposal was executed
    Executed,
    /// Proposal was rejected
    Rejected,
    /// Proposal was cancelled
    Cancelled,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Voting => write!(f, "voting"),
            ProposalStatus::Approved => write!(f, "approved"),
            ProposalStatus::Executed => write!(f, "executed"),
            ProposalStatus::Rejected => write!(f, "rejected"),
            ProposalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// State migration for contract upgrades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMigration {
    /// Migration identifier
    pub id: String,
    /// Contract being migrated
    pub contract_name: String,
    /// From version
    pub from_version: String,
    /// To version
    pub to_version: String,
    /// Migration status
    pub status: MigrationStatus,
    /// Old state snapshot
    pub old_state: HashMap<String, Value>,
    /// New state after migration
    pub new_state: HashMap<String, Value>,
    /// Migration timestamp
    pub timestamp: DateTime<Utc>,
    /// Migration errors (if any)
    pub errors: Vec<String>,
}

/// Status of a state migration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationStatus {
    /// Migration is in progress
    InProgress,
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed,
    /// Migration was rolled back
    RolledBack,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::InProgress => write!(f, "in_progress"),
            MigrationStatus::Completed => write!(f, "completed"),
            MigrationStatus::Failed => write!(f, "failed"),
            MigrationStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Contract upgrade manager
#[derive(Debug, Clone)]
pub struct UpgradeManager {
    /// Contract versions indexed by contract name
    versions: HashMap<String, Vec<ContractVersion>>,
    /// Current active version for each contract
    active_versions: HashMap<String, String>,
    /// Upgrade proposals
    proposals: HashMap<String, UpgradeProposal>,
    /// State migrations
    migrations: HashMap<String, StateMigration>,
}

impl Default for UpgradeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeManager {
    /// Create a new upgrade manager
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            active_versions: HashMap::new(),
            proposals: HashMap::new(),
            migrations: HashMap::new(),
        }
    }

    /// Register a new contract version
    pub fn register_version(
        &mut self,
        contract_name: String,
        version: String,
        code_hash: String,
        bytecode: Vec<u8>,
        description: Option<String>,
    ) -> SlvrResult<String> {
        let version_id = Uuid::new_v4().to_string();

        let contract_version = ContractVersion {
            id: version_id.clone(),
            contract_name: contract_name.clone(),
            version: version.clone(),
            code_hash,
            bytecode,
            description,
            created_at: Utc::now(),
            active: false,
            metadata: HashMap::new(),
        };

        self.versions
            .entry(contract_name)
            .or_default()
            .push(contract_version);

        Ok(version_id)
    }

    /// Get a contract version
    pub fn get_version(&self, contract_name: &str, version: &str) -> SlvrResult<ContractVersion> {
        self.versions
            .get(contract_name)
            .and_then(|versions| versions.iter().find(|v| v.version == version).cloned())
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Version not found: {}/{}", contract_name, version),
            })
    }

    /// Get all versions of a contract
    pub fn get_all_versions(&self, contract_name: &str) -> Vec<ContractVersion> {
        self.versions
            .get(contract_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get the active version of a contract
    pub fn get_active_version(&self, contract_name: &str) -> SlvrResult<ContractVersion> {
        let version_str =
            self.active_versions
                .get(contract_name)
                .ok_or_else(|| SlvrError::RuntimeError {
                    message: format!("No active version for contract: {}", contract_name),
                })?;

        self.get_version(contract_name, version_str)
    }

    /// Activate a contract version
    pub fn activate_version(&mut self, contract_name: String, version: String) -> SlvrResult<()> {
        // Verify version exists
        self.get_version(&contract_name, &version)?;

        // Deactivate previous version
        if let Some(versions) = self.versions.get_mut(&contract_name) {
            for v in versions.iter_mut() {
                v.active = false;
            }
            // Activate new version
            if let Some(v) = versions.iter_mut().find(|v| v.version == version) {
                v.active = true;
            }
        }

        self.active_versions.insert(contract_name, version);
        Ok(())
    }

    /// Create an upgrade proposal
    pub fn create_upgrade_proposal(
        &mut self,
        contract_name: String,
        from_version: String,
        to_version: String,
        description: Option<String>,
        voting_period_hours: i64,
        migration_script: Option<String>,
    ) -> SlvrResult<String> {
        // Verify both versions exist
        self.get_version(&contract_name, &from_version)?;
        self.get_version(&contract_name, &to_version)?;

        let proposal_id = Uuid::new_v4().to_string();

        let proposal = UpgradeProposal {
            id: proposal_id.clone(),
            contract_name,
            from_version,
            to_version,
            description,
            status: ProposalStatus::Voting,
            created_at: Utc::now(),
            executed_at: None,
            votes_for: 0,
            votes_against: 0,
            voting_deadline: Utc::now() + chrono::Duration::hours(voting_period_hours),
            migration_script,
        };

        self.proposals.insert(proposal_id.clone(), proposal);
        Ok(proposal_id)
    }

    /// Vote on an upgrade proposal
    pub fn vote_on_proposal(&mut self, proposal_id: &str, vote_for: bool) -> SlvrResult<()> {
        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
            if proposal.status != ProposalStatus::Voting {
                return Err(SlvrError::RuntimeError {
                    message: "Proposal is not open for voting".to_string(),
                });
            }

            if Utc::now() > proposal.voting_deadline {
                proposal.status = ProposalStatus::Rejected;
                return Err(SlvrError::RuntimeError {
                    message: "Voting period has ended".to_string(),
                });
            }

            if vote_for {
                proposal.votes_for += 1;
            } else {
                proposal.votes_against += 1;
            }

            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("Proposal not found: {}", proposal_id),
            })
        }
    }

    /// Finalize voting on a proposal
    pub fn finalize_proposal(&mut self, proposal_id: &str) -> SlvrResult<()> {
        if let Some(proposal) = self.proposals.get_mut(proposal_id) {
            if proposal.status != ProposalStatus::Voting {
                return Err(SlvrError::RuntimeError {
                    message: "Proposal is not in voting status".to_string(),
                });
            }

            if Utc::now() <= proposal.voting_deadline {
                return Err(SlvrError::RuntimeError {
                    message: "Voting period has not ended".to_string(),
                });
            }

            if proposal.votes_for > proposal.votes_against {
                proposal.status = ProposalStatus::Approved;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }

            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("Proposal not found: {}", proposal_id),
            })
        }
    }

    /// Execute an approved upgrade proposal
    pub fn execute_upgrade(&mut self, proposal_id: &str) -> SlvrResult<String> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Proposal not found: {}", proposal_id),
            })?
            .clone();

        if proposal.status != ProposalStatus::Approved {
            return Err(SlvrError::RuntimeError {
                message: "Proposal is not approved".to_string(),
            });
        }

        // Create state migration record
        let migration_id = Uuid::new_v4().to_string();
        let migration = StateMigration {
            id: migration_id.clone(),
            contract_name: proposal.contract_name.clone(),
            from_version: proposal.from_version.clone(),
            to_version: proposal.to_version.clone(),
            status: MigrationStatus::InProgress,
            old_state: HashMap::new(),
            new_state: HashMap::new(),
            timestamp: Utc::now(),
            errors: Vec::new(),
        };

        self.migrations.insert(migration_id.clone(), migration);

        // Activate new version
        self.activate_version(proposal.contract_name, proposal.to_version)?;

        // Update proposal status
        if let Some(p) = self.proposals.get_mut(proposal_id) {
            p.status = ProposalStatus::Executed;
            p.executed_at = Some(Utc::now());
        }

        Ok(migration_id)
    }

    /// Get an upgrade proposal
    pub fn get_proposal(&self, proposal_id: &str) -> SlvrResult<UpgradeProposal> {
        self.proposals
            .get(proposal_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Proposal not found: {}", proposal_id),
            })
    }

    /// Get all proposals for a contract
    pub fn get_contract_proposals(&self, contract_name: &str) -> Vec<UpgradeProposal> {
        self.proposals
            .values()
            .filter(|p| p.contract_name == contract_name)
            .cloned()
            .collect()
    }

    /// Get a state migration
    pub fn get_migration(&self, migration_id: &str) -> SlvrResult<StateMigration> {
        self.migrations
            .get(migration_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Migration not found: {}", migration_id),
            })
    }

    /// Get upgrade statistics
    pub fn get_stats(&self) -> UpgradeStats {
        let total_contracts = self.versions.len();
        let total_versions: usize = self.versions.values().map(|v| v.len()).sum();
        let total_proposals = self.proposals.len();
        let approved_proposals = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Approved)
            .count();
        let executed_proposals = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Executed)
            .count();

        UpgradeStats {
            total_contracts,
            total_versions,
            total_proposals,
            approved_proposals,
            executed_proposals,
            total_migrations: self.migrations.len(),
        }
    }
}

/// Statistics for upgrade manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeStats {
    pub total_contracts: usize,
    pub total_versions: usize,
    pub total_proposals: usize,
    pub approved_proposals: usize,
    pub executed_proposals: usize,
    pub total_migrations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_version() {
        let mut manager = UpgradeManager::new();
        let _version_id = manager
            .register_version(
                "token".to_string(),
                "1.0.0".to_string(),
                "hash1".to_string(),
                vec![1, 2, 3],
                Some("Initial version".to_string()),
            )
            .unwrap();

        let version = manager.get_version("token", "1.0.0").unwrap();
        assert_eq!(version.version, "1.0.0");
    }

    #[test]
    fn test_activate_version() {
        let mut manager = UpgradeManager::new();
        manager
            .register_version(
                "token".to_string(),
                "1.0.0".to_string(),
                "hash1".to_string(),
                vec![1, 2, 3],
                None,
            )
            .unwrap();

        manager
            .activate_version("token".to_string(), "1.0.0".to_string())
            .unwrap();

        let active = manager.get_active_version("token").unwrap();
        assert_eq!(active.version, "1.0.0");
        assert!(active.active);
    }

    #[test]
    fn test_upgrade_proposal() {
        let mut manager = UpgradeManager::new();
        manager
            .register_version(
                "token".to_string(),
                "1.0.0".to_string(),
                "hash1".to_string(),
                vec![1, 2, 3],
                None,
            )
            .unwrap();
        manager
            .register_version(
                "token".to_string(),
                "2.0.0".to_string(),
                "hash2".to_string(),
                vec![4, 5, 6],
                None,
            )
            .unwrap();

        let proposal_id = manager
            .create_upgrade_proposal(
                "token".to_string(),
                "1.0.0".to_string(),
                "2.0.0".to_string(),
                Some("Major upgrade".to_string()),
                24,
                None,
            )
            .unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
    }

    #[test]
    fn test_voting() {
        let mut manager = UpgradeManager::new();
        manager
            .register_version(
                "token".to_string(),
                "1.0.0".to_string(),
                "hash1".to_string(),
                vec![1, 2, 3],
                None,
            )
            .unwrap();
        manager
            .register_version(
                "token".to_string(),
                "2.0.0".to_string(),
                "hash2".to_string(),
                vec![4, 5, 6],
                None,
            )
            .unwrap();

        let proposal_id = manager
            .create_upgrade_proposal(
                "token".to_string(),
                "1.0.0".to_string(),
                "2.0.0".to_string(),
                None,
                24,
                None,
            )
            .unwrap();

        manager.vote_on_proposal(&proposal_id, true).unwrap();
        manager.vote_on_proposal(&proposal_id, true).unwrap();
        manager.vote_on_proposal(&proposal_id, false).unwrap();

        let proposal = manager.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 2);
        assert_eq!(proposal.votes_against, 1);
    }

    #[test]
    fn test_upgrade_stats() {
        let mut manager = UpgradeManager::new();
        manager
            .register_version(
                "token".to_string(),
                "1.0.0".to_string(),
                "hash1".to_string(),
                vec![1, 2, 3],
                None,
            )
            .unwrap();
        manager
            .register_version(
                "token".to_string(),
                "2.0.0".to_string(),
                "hash2".to_string(),
                vec![4, 5, 6],
                None,
            )
            .unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_contracts, 1);
        assert_eq!(stats.total_versions, 2);
    }
}
