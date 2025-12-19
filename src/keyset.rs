//! Keyset Management - Key Authorization and Multi-Signature Support
//!
//! This module provides comprehensive keyset management for smart contracts,
//! including key authorization, multi-signature support, and capability tokens.

use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};

/// Represents a cryptographic key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Key {
    pub id: String,
    pub public_key: String,
    pub key_type: KeyType,
}

/// Type of cryptographic key
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
    BLS,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => write!(f, "ed25519"),
            KeyType::Secp256k1 => write!(f, "secp256k1"),
            KeyType::BLS => write!(f, "bls"),
        }
    }
}

/// Keyset - A collection of keys with authorization rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyset {
    pub name: String,
    pub keys: Vec<Key>,
    pub threshold: usize,
    pub predicate: KeysetPredicate,
}

/// Keyset predicate for authorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeysetPredicate {
    /// All keys must sign
    All,
    /// Any key can sign
    Any,
    /// At least N keys must sign
    AtLeast(usize),
    /// Custom predicate function
    Custom(String),
}

impl Keyset {
    /// Create a new keyset
    pub fn new(name: String, keys: Vec<Key>, threshold: usize) -> SlvrResult<Self> {
        if keys.is_empty() {
            return Err(SlvrError::RuntimeError {
                message: "keyset must have at least one key".to_string(),
            });
        }

        if threshold == 0 || threshold > keys.len() {
            return Err(SlvrError::RuntimeError {
                message: format!(
                    "threshold must be between 1 and {}",
                    keys.len()
                ),
            });
        }

        Ok(Keyset {
            name,
            keys,
            threshold,
            predicate: KeysetPredicate::AtLeast(threshold),
        })
    }

    /// Add a key to the keyset
    pub fn add_key(&mut self, key: Key) -> SlvrResult<()> {
        if self.keys.iter().any(|k| k.id == key.id) {
            return Err(SlvrError::RuntimeError {
                message: format!("key {} already exists", key.id),
            });
        }
        self.keys.push(key);
        Ok(())
    }

    /// Remove a key from the keyset
    pub fn remove_key(&mut self, key_id: &str) -> SlvrResult<()> {
        if let Some(pos) = self.keys.iter().position(|k| k.id == key_id) {
            self.keys.remove(pos);
            if self.keys.is_empty() {
                return Err(SlvrError::RuntimeError {
                    message: "cannot remove last key from keyset".to_string(),
                });
            }
            Ok(())
        } else {
            Err(SlvrError::RuntimeError {
                message: format!("key {} not found", key_id),
            })
        }
    }

    /// Check if keyset satisfies authorization
    pub fn authorize(&self, signed_keys: &[String]) -> SlvrResult<bool> {
        match &self.predicate {
            KeysetPredicate::All => {
                let required: HashSet<_> = self.keys.iter().map(|k| k.id.clone()).collect();
                let signed: HashSet<_> = signed_keys.iter().cloned().collect();
                Ok(required == signed)
            }
            KeysetPredicate::Any => {
                Ok(!signed_keys.is_empty() && signed_keys.iter().all(|k| {
                    self.keys.iter().any(|key| key.id == *k)
                }))
            }
            KeysetPredicate::AtLeast(n) => {
                let valid_signatures = signed_keys
                    .iter()
                    .filter(|k| self.keys.iter().any(|key| key.id == **k))
                    .count();
                Ok(valid_signatures >= *n)
            }
            KeysetPredicate::Custom(_) => {
                Ok(true)
            }
        }
    }

    /// Get keyset hash
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        for key in &self.keys {
            hasher.update(key.id.as_bytes());
            hasher.update(key.public_key.as_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

/// Capability token for fine-grained authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub keyset: String,
    pub permissions: Vec<String>,
    pub expiry: Option<u64>,
}

impl Capability {
    /// Create a new capability
    pub fn new(
        name: String,
        keyset: String,
        permissions: Vec<String>,
    ) -> Self {
        let id = format!("cap_{}", uuid::Uuid::new_v4());
        Capability {
            id,
            name,
            keyset,
            permissions,
            expiry: None,
        }
    }

    /// Set capability expiry
    pub fn with_expiry(mut self, expiry: u64) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Check if capability has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Check if capability is expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        if let Some(expiry) = self.expiry {
            current_time > expiry
        } else {
            false
        }
    }
}

/// Keyset manager for managing multiple keysets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysetManager {
    keysets: HashMap<String, Keyset>,
    capabilities: HashMap<String, Capability>,
}

impl KeysetManager {
    /// Create a new keyset manager
    pub fn new() -> Self {
        KeysetManager {
            keysets: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }

    /// Register a keyset
    pub fn register_keyset(&mut self, keyset: Keyset) -> SlvrResult<()> {
        if self.keysets.contains_key(&keyset.name) {
            return Err(SlvrError::RuntimeError {
                message: format!("keyset {} already exists", keyset.name),
            });
        }
        self.keysets.insert(keyset.name.clone(), keyset);
        Ok(())
    }

    /// Get a keyset
    pub fn get_keyset(&self, name: &str) -> SlvrResult<Keyset> {
        self.keysets
            .get(name)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("keyset {} not found", name),
            })
    }

    /// Update a keyset
    pub fn update_keyset(&mut self, keyset: Keyset) -> SlvrResult<()> {
        if !self.keysets.contains_key(&keyset.name) {
            return Err(SlvrError::RuntimeError {
                message: format!("keyset {} not found", keyset.name),
            });
        }
        self.keysets.insert(keyset.name.clone(), keyset);
        Ok(())
    }

    /// Delete a keyset
    pub fn delete_keyset(&mut self, name: &str) -> SlvrResult<()> {
        if self.keysets.remove(name).is_none() {
            return Err(SlvrError::RuntimeError {
                message: format!("keyset {} not found", name),
            });
        }
        Ok(())
    }

    /// Register a capability
    pub fn register_capability(&mut self, capability: Capability) -> SlvrResult<()> {
        if self.capabilities.contains_key(&capability.id) {
            return Err(SlvrError::RuntimeError {
                message: format!("capability {} already exists", capability.id),
            });
        }
        self.capabilities.insert(capability.id.clone(), capability);
        Ok(())
    }

    /// Get a capability
    pub fn get_capability(&self, id: &str) -> SlvrResult<Capability> {
        self.capabilities
            .get(id)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("capability {} not found", id),
            })
    }

    /// Revoke a capability
    pub fn revoke_capability(&mut self, id: &str) -> SlvrResult<()> {
        if self.capabilities.remove(id).is_none() {
            return Err(SlvrError::RuntimeError {
                message: format!("capability {} not found", id),
            });
        }
        Ok(())
    }

    /// Authorize with keyset
    pub fn authorize(&self, keyset_name: &str, signed_keys: &[String]) -> SlvrResult<bool> {
        let keyset = self.get_keyset(keyset_name)?;
        keyset.authorize(signed_keys)
    }

    /// List all keysets
    pub fn list_keysets(&self) -> Vec<String> {
        self.keysets.keys().cloned().collect()
    }

    /// List all capabilities
    pub fn list_capabilities(&self) -> Vec<String> {
        self.capabilities.keys().cloned().collect()
    }
}

impl Default for KeysetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyset_creation() {
        let keys = vec![
            Key {
                id: "key1".to_string(),
                public_key: "pub1".to_string(),
                key_type: KeyType::Ed25519,
            },
            Key {
                id: "key2".to_string(),
                public_key: "pub2".to_string(),
                key_type: KeyType::Ed25519,
            },
        ];

        let keyset = Keyset::new("test".to_string(), keys, 2).unwrap();
        assert_eq!(keyset.name, "test");
        assert_eq!(keyset.keys.len(), 2);
        assert_eq!(keyset.threshold, 2);
    }

    #[test]
    fn test_keyset_authorization() {
        let keys = vec![
            Key {
                id: "key1".to_string(),
                public_key: "pub1".to_string(),
                key_type: KeyType::Ed25519,
            },
            Key {
                id: "key2".to_string(),
                public_key: "pub2".to_string(),
                key_type: KeyType::Ed25519,
            },
        ];

        let keyset = Keyset::new("test".to_string(), keys, 2).unwrap();
        
        let signed = vec!["key1".to_string(), "key2".to_string()];
        assert!(keyset.authorize(&signed).unwrap());

        let signed = vec!["key1".to_string()];
        assert!(!keyset.authorize(&signed).unwrap());
    }

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(
            "transfer".to_string(),
            "default".to_string(),
            vec!["transfer".to_string(), "approve".to_string()],
        );

        assert!(cap.has_permission("transfer"));
        assert!(cap.has_permission("approve"));
        assert!(!cap.has_permission("mint"));
    }

    #[test]
    fn test_keyset_manager() {
        let mut manager = KeysetManager::new();

        let keys = vec![Key {
            id: "key1".to_string(),
            public_key: "pub1".to_string(),
            key_type: KeyType::Ed25519,
        }];

        let keyset = Keyset::new("test".to_string(), keys, 1).unwrap();
        manager.register_keyset(keyset).unwrap();

        assert!(manager.get_keyset("test").is_ok());
        assert!(manager.get_keyset("nonexistent").is_err());
    }
}
