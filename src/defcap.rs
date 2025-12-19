//! Capability Definitions (Defcap) Support
//!
//! This module provides support for capability definitions which allow fine-grained
//! permission control and authorization patterns in smart contracts.

use crate::error::{SlvrError, SlvrResult};
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Represents a capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDef {
    /// Unique capability identifier
    pub id: String,
    /// Capability name
    pub name: String,
    /// Contract that defined this capability
    pub contract: String,
    /// Capability parameters
    pub params: Vec<(String, String)>, // (name, type)
    /// Capability description
    pub description: Option<String>,
    /// Whether this capability is managed (can be granted/revoked)
    pub managed: bool,
    /// Timestamp when capability was defined
    pub created_at: DateTime<Utc>,
}

/// Represents a granted capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantedCapability {
    /// Unique grant identifier
    pub id: String,
    /// Reference to capability definition
    pub capability_id: String,
    /// Principal (account/user) that was granted this capability
    pub principal: String,
    /// Capability parameters bound to this grant
    pub params: HashMap<String, Value>,
    /// Timestamp when capability was granted
    pub granted_at: DateTime<Utc>,
    /// Timestamp when capability expires (None = never expires)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether this grant is currently active
    pub active: bool,
    /// Metadata associated with this grant
    pub metadata: HashMap<String, String>,
}

/// Capability scope - defines what operations a capability allows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityScope {
    /// Capability allows reading data
    Read,
    /// Capability allows writing data
    Write,
    /// Capability allows deleting data
    Delete,
    /// Capability allows executing functions
    Execute,
    /// Capability allows transferring assets
    Transfer,
    /// Custom scope
    Custom(String),
}

impl std::fmt::Display for CapabilityScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityScope::Read => write!(f, "read"),
            CapabilityScope::Write => write!(f, "write"),
            CapabilityScope::Delete => write!(f, "delete"),
            CapabilityScope::Execute => write!(f, "execute"),
            CapabilityScope::Transfer => write!(f, "transfer"),
            CapabilityScope::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Capability with scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedCapability {
    pub capability: GrantedCapability,
    pub scopes: Vec<CapabilityScope>,
}

/// Capability manager for handling capability definitions and grants
#[derive(Debug, Clone)]
pub struct CapabilityManager {
    /// Capability definitions indexed by ID
    definitions: HashMap<String, CapabilityDef>,
    /// Granted capabilities indexed by ID
    grants: HashMap<String, GrantedCapability>,
    /// Index of grants by principal
    grants_by_principal: HashMap<String, Vec<String>>,
    /// Index of grants by capability
    grants_by_capability: HashMap<String, Vec<String>>,
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            grants: HashMap::new(),
            grants_by_principal: HashMap::new(),
            grants_by_capability: HashMap::new(),
        }
    }

    /// Define a new capability
    pub fn define_capability(
        &mut self,
        name: String,
        contract: String,
        params: Vec<(String, String)>,
        description: Option<String>,
        managed: bool,
    ) -> SlvrResult<String> {
        let cap_id = Uuid::new_v4().to_string();

        let capability = CapabilityDef {
            id: cap_id.clone(),
            name,
            contract,
            params,
            description,
            managed,
            created_at: Utc::now(),
        };

        self.definitions.insert(cap_id.clone(), capability);
        Ok(cap_id)
    }

    /// Get a capability definition
    pub fn get_capability_def(&self, cap_id: &str) -> SlvrResult<CapabilityDef> {
        self.definitions
            .get(cap_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Capability not found: {}", cap_id),
            })
    }

    /// Grant a capability to a principal
    pub fn grant_capability(
        &mut self,
        capability_id: String,
        principal: String,
        params: HashMap<String, Value>,
        expires_in: Option<Duration>,
    ) -> SlvrResult<String> {
        // Verify capability exists
        self.get_capability_def(&capability_id)?;

        let grant_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let grant = GrantedCapability {
            id: grant_id.clone(),
            capability_id: capability_id.clone(),
            principal: principal.clone(),
            params,
            granted_at: now,
            expires_at: expires_in.map(|d| now + d),
            active: true,
            metadata: HashMap::new(),
        };

        self.grants.insert(grant_id.clone(), grant);

        // Update indices
        self.grants_by_principal
            .entry(principal)
            .or_insert_with(Vec::new)
            .push(grant_id.clone());

        self.grants_by_capability
            .entry(capability_id)
            .or_insert_with(Vec::new)
            .push(grant_id.clone());

        Ok(grant_id)
    }

    /// Revoke a capability grant
    pub fn revoke_capability(&mut self, grant_id: &str) -> SlvrResult<()> {
        if let Some(grant) = self.grants.get_mut(grant_id) {
            grant.active = false;
            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("Grant not found: {}", grant_id),
            })
        }
    }

    /// Check if a principal has a capability
    pub fn has_capability(&self, principal: &str, capability_id: &str) -> bool {
        if let Some(grant_ids) = self.grants_by_principal.get(principal) {
            grant_ids.iter().any(|grant_id| {
                if let Some(grant) = self.grants.get(grant_id) {
                    grant.active
                        && grant.capability_id == capability_id
                        && grant.expires_at.map_or(true, |exp| exp > Utc::now())
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    /// Get all capabilities for a principal
    pub fn get_principal_capabilities(&self, principal: &str) -> Vec<GrantedCapability> {
        self.grants_by_principal
            .get(principal)
            .map(|grant_ids| {
                grant_ids
                    .iter()
                    .filter_map(|grant_id| {
                        self.grants.get(grant_id).cloned().filter(|g| {
                            g.active && g.expires_at.map_or(true, |exp| exp > Utc::now())
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all grants for a capability
    pub fn get_capability_grants(&self, capability_id: &str) -> Vec<GrantedCapability> {
        self.grants_by_capability
            .get(capability_id)
            .map(|grant_ids| {
                grant_ids
                    .iter()
                    .filter_map(|grant_id| {
                        self.grants.get(grant_id).cloned().filter(|g| {
                            g.active && g.expires_at.map_or(true, |exp| exp > Utc::now())
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a specific grant
    pub fn get_grant(&self, grant_id: &str) -> SlvrResult<GrantedCapability> {
        self.grants
            .get(grant_id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Grant not found: {}", grant_id),
            })
    }

    /// Update grant metadata
    pub fn update_grant_metadata(
        &mut self,
        grant_id: &str,
        metadata: HashMap<String, String>,
    ) -> SlvrResult<()> {
        if let Some(grant) = self.grants.get_mut(grant_id) {
            grant.metadata = metadata;
            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("Grant not found: {}", grant_id),
            })
        }
    }

    /// Clean up expired grants
    pub fn cleanup_expired_grants(&mut self) {
        let now = Utc::now();
        let expired: Vec<String> = self
            .grants
            .iter()
            .filter(|(_, grant)| {
                grant.expires_at.map_or(false, |exp| exp <= now)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for grant_id in expired {
            self.grants.remove(&grant_id);
        }
    }

    /// Get capability statistics
    pub fn get_stats(&self) -> CapabilityStats {
        let now = Utc::now();
        let active_grants = self
            .grants
            .values()
            .filter(|g| g.active && g.expires_at.map_or(true, |exp| exp > now))
            .count();

        let expired_grants = self
            .grants
            .values()
            .filter(|g| g.expires_at.map_or(false, |exp| exp <= now))
            .count();

        CapabilityStats {
            total_definitions: self.definitions.len(),
            total_grants: self.grants.len(),
            active_grants,
            expired_grants,
            principals_with_capabilities: self.grants_by_principal.len(),
        }
    }
}

/// Statistics for capability manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStats {
    pub total_definitions: usize,
    pub total_grants: usize,
    pub active_grants: usize,
    pub expired_grants: usize,
    pub principals_with_capabilities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_definition() {
        let mut manager = CapabilityManager::new();
        let cap_id = manager
            .define_capability(
                "transfer".to_string(),
                "token".to_string(),
                vec![("amount".to_string(), "integer".to_string())],
                Some("Transfer capability".to_string()),
                true,
            )
            .unwrap();

        let cap = manager.get_capability_def(&cap_id).unwrap();
        assert_eq!(cap.name, "transfer");
        assert!(cap.managed);
    }

    #[test]
    fn test_grant_capability() {
        let mut manager = CapabilityManager::new();
        let cap_id = manager
            .define_capability(
                "transfer".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        let _grant_id = manager
            .grant_capability(cap_id.clone(), "alice".to_string(), HashMap::new(), None)
            .unwrap();

        assert!(manager.has_capability("alice", &cap_id));
        assert!(!manager.has_capability("bob", &cap_id));
    }

    #[test]
    fn test_revoke_capability() {
        let mut manager = CapabilityManager::new();
        let cap_id = manager
            .define_capability(
                "transfer".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        let grant_id = manager
            .grant_capability(cap_id.clone(), "alice".to_string(), HashMap::new(), None)
            .unwrap();

        assert!(manager.has_capability("alice", &cap_id));

        manager.revoke_capability(&grant_id).unwrap();
        assert!(!manager.has_capability("alice", &cap_id));
    }

    #[test]
    fn test_capability_expiry() {
        let mut manager = CapabilityManager::new();
        let cap_id = manager
            .define_capability(
                "transfer".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        let _grant_id = manager
            .grant_capability(
                cap_id.clone(),
                "alice".to_string(),
                HashMap::new(),
                Some(Duration::seconds(-1)), // Already expired
            )
            .unwrap();

        assert!(!manager.has_capability("alice", &cap_id));
    }

    #[test]
    fn test_get_principal_capabilities() {
        let mut manager = CapabilityManager::new();
        let cap1 = manager
            .define_capability(
                "read".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        let cap2 = manager
            .define_capability(
                "write".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        manager
            .grant_capability(cap1, "alice".to_string(), HashMap::new(), None)
            .unwrap();
        manager
            .grant_capability(cap2, "alice".to_string(), HashMap::new(), None)
            .unwrap();

        let caps = manager.get_principal_capabilities("alice");
        assert_eq!(caps.len(), 2);
    }

    #[test]
    fn test_capability_stats() {
        let mut manager = CapabilityManager::new();
        let cap_id = manager
            .define_capability(
                "transfer".to_string(),
                "token".to_string(),
                vec![],
                None,
                true,
            )
            .unwrap();

        manager
            .grant_capability(cap_id.clone(), "alice".to_string(), HashMap::new(), None)
            .unwrap();
        manager
            .grant_capability(cap_id, "bob".to_string(), HashMap::new(), None)
            .unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_definitions, 1);
        assert_eq!(stats.total_grants, 2);
        assert_eq!(stats.active_grants, 2);
    }
}
