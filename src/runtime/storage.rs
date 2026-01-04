use std::collections::HashMap;
use thiserror::Error;

use crate::runtime::metering::MeteringContext;

/// Error types for storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Key too large: {0} bytes, maximum is {1} bytes")]
    KeyTooLarge(usize, usize),

    #[error("Value too large: {0} bytes, maximum is {1} bytes")]
    ValueTooLarge(usize, usize),

    #[error("Metering error: {0}")]
    Metering(String),

    #[error("Storage limit exceeded")]
    StorageLimitExceeded,
}

/// Storage limits
#[derive(Debug, Clone, Copy)]
pub struct StorageLimits {
    /// Maximum key size
    pub max_key_size: usize,

    /// Maximum value size
    pub max_value_size: usize,

    /// Maximum storage items
    pub max_storage_items: usize,
}

impl Default for StorageLimits {
    fn default() -> Self {
        StorageLimits {
            max_key_size: 1024,             // 1 KB
            max_value_size: 16 * 1024,      // 16 KB
            max_storage_items: 1024 * 1024, // 1M items
        }
    }
}

/// Storage manager for contract storage
pub struct StorageManager {
    /// Contract storage (key -> value)
    storage: HashMap<Vec<u8>, Vec<u8>>,

    /// Contract address (used for namespace isolation)
    contract_address: [u8; 32],

    /// Storage limits
    limits: StorageLimits,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(contract_address: [u8; 32], limits: StorageLimits) -> Self {
        StorageManager {
            storage: HashMap::new(),
            contract_address,
            limits,
        }
    }

    /// Get key with namespace
    fn namespaced_key(&self, key: &[u8]) -> Vec<u8> {
        let mut namespaced = Vec::with_capacity(self.contract_address.len() + key.len());
        namespaced.extend_from_slice(&self.contract_address);
        namespaced.extend_from_slice(key);
        namespaced
    }

    /// Set a storage value
    pub fn set(
        &mut self,
        key: &[u8],
        value: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<(), StorageError> {
        // Validate key and value sizes
        if key.len() > self.limits.max_key_size {
            return Err(StorageError::KeyTooLarge(
                key.len(),
                self.limits.max_key_size,
            ));
        }

        if value.len() > self.limits.max_value_size {
            return Err(StorageError::ValueTooLarge(
                value.len(),
                self.limits.max_value_size,
            ));
        }

        // Check storage limit
        if !self.storage.contains_key(&self.namespaced_key(key))
            && self.storage.len() >= self.limits.max_storage_items
        {
            return Err(StorageError::StorageLimitExceeded);
        }

        // Charge for storage operation
        metering
            .charge_storage_write(key, value)
            .map_err(|e| StorageError::Metering(e.to_string()))?;

        // Set storage
        let namespaced_key = self.namespaced_key(key);
        self.storage.insert(namespaced_key, value.to_vec());

        Ok(())
    }

    /// Get a storage value
    pub fn get(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        // Validate key size
        if key.len() > self.limits.max_key_size {
            return Err(StorageError::KeyTooLarge(
                key.len(),
                self.limits.max_key_size,
            ));
        }

        // Charge for storage operation
        metering
            .charge_storage_read(key)
            .map_err(|e| StorageError::Metering(e.to_string()))?;

        // Get storage
        let namespaced_key = self.namespaced_key(key);
        Ok(self.storage.get(&namespaced_key).cloned())
    }

    /// Check if a storage key exists
    pub fn contains(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<bool, StorageError> {
        // Validate key size
        if key.len() > self.limits.max_key_size {
            return Err(StorageError::KeyTooLarge(
                key.len(),
                self.limits.max_key_size,
            ));
        }

        // Charge for storage operation (same as reading)
        metering
            .charge_storage_read(key)
            .map_err(|e| StorageError::Metering(e.to_string()))?;

        // Check storage
        let namespaced_key = self.namespaced_key(key);
        Ok(self.storage.contains_key(&namespaced_key))
    }

    /// Remove a storage value
    pub fn remove(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<(), StorageError> {
        // Validate key size
        if key.len() > self.limits.max_key_size {
            return Err(StorageError::KeyTooLarge(
                key.len(),
                self.limits.max_key_size,
            ));
        }

        // Charge for storage operation
        metering
            .charge_storage_delete(key)
            .map_err(|e| StorageError::Metering(e.to_string()))?;

        // Remove from storage
        let namespaced_key = self.namespaced_key(key);
        self.storage.remove(&namespaced_key);

        Ok(())
    }

    /// Clear storage (used for contract destruction)
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    /// Get all storage keys (for debugging/testing)
    pub fn keys(&self) -> Vec<Vec<u8>> {
        let prefix_len = self.contract_address.len();

        self.storage
            .keys()
            .filter_map(|key| {
                if key.len() > prefix_len {
                    Some(key[prefix_len..].to_vec())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all storage entries (for debugging/testing)
    pub fn entries(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let prefix_len = self.contract_address.len();

        self.storage
            .iter()
            .filter_map(|(key, value)| {
                if key.len() > prefix_len {
                    Some((key[prefix_len..].to_vec(), value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get storage for a specific key prefix (for iteration)
    pub fn prefix_iter(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        let namespaced_prefix = self.namespaced_key(prefix);

        self.storage
            .iter()
            .filter_map(|(key, value)| {
                if key.starts_with(&namespaced_prefix) {
                    Some((key[self.contract_address.len()..].to_vec(), value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Import storage (used for testing or migrating contracts)
    pub fn import(&mut self, entries: Vec<(Vec<u8>, Vec<u8>)>) {
        for (key, value) in entries {
            let namespaced_key = self.namespaced_key(&key);
            self.storage.insert(namespaced_key, value);
        }
    }
}

/// Child storage for contract sub-storage
pub struct ChildStorage {
    /// Parent storage manager
    parent: StorageManager,

    /// Child storage prefix
    prefix: Vec<u8>,
}

impl ChildStorage {
    /// Create a new child storage
    pub fn new(parent: StorageManager, prefix: Vec<u8>) -> Self {
        ChildStorage { parent, prefix }
    }

    /// Prefix a key with the child storage prefix
    fn prefixed_key(&self, key: &[u8]) -> Vec<u8> {
        let mut prefixed = Vec::with_capacity(self.prefix.len() + key.len());
        prefixed.extend_from_slice(&self.prefix);
        prefixed.extend_from_slice(key);
        prefixed
    }

    /// Set a storage value
    pub fn set(
        &mut self,
        key: &[u8],
        value: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<(), StorageError> {
        let prefixed_key = self.prefixed_key(key);
        self.parent.set(&prefixed_key, value, metering)
    }

    /// Get a storage value
    pub fn get(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let prefixed_key = self.prefixed_key(key);
        self.parent.get(&prefixed_key, metering)
    }

    /// Check if a storage key exists
    pub fn contains(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<bool, StorageError> {
        let prefixed_key = self.prefixed_key(key);
        self.parent.contains(&prefixed_key, metering)
    }

    /// Remove a storage value
    pub fn remove(
        &mut self,
        key: &[u8],
        metering: &mut MeteringContext,
    ) -> Result<(), StorageError> {
        let prefixed_key = self.prefixed_key(key);
        self.parent.remove(&prefixed_key, metering)
    }

    /// Clear child storage
    pub fn clear(&mut self, metering: &mut MeteringContext) -> Result<(), StorageError> {
        for key in self.parent.prefix_iter(&self.prefix) {
            self.parent.remove(&key.0, metering)?;
        }

        Ok(())
    }

    /// Get all storage keys in this child storage
    pub fn keys(&self) -> Vec<Vec<u8>> {
        self.parent
            .prefix_iter(&self.prefix)
            .into_iter()
            .map(|(key, _)| key[self.prefix.len()..].to_vec())
            .collect()
    }

    /// Get all storage entries in this child storage
    pub fn entries(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.parent
            .prefix_iter(&self.prefix)
            .into_iter()
            .map(|(key, value)| (key[self.prefix.len()..].to_vec(), value))
            .collect()
    }
}
