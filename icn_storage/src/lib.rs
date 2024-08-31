// File: icn_storage/src/lib.rs

//! This module defines the storage components for the InterCooperative Network (ICN).
//! 
//! It provides a centralized interface for managing both block and state storage,
//! offering thread-safe access to these storage components through the use of
//! `Arc` and `RwLock`. The `Storage` struct serves as the main entry point for
//! all storage-related operations in the ICN node.

use std::sync::{Arc, RwLock};
use icn_shared::{Block, IcnResult, IcnError};

pub mod block_storage;
pub mod state_storage;

use block_storage::BlockStorage;
use state_storage::StateStorage;

/// `Storage` is the central structure that manages block and state storage for the ICN node.
/// 
/// This struct provides methods for adding blocks, retrieving states, and verifying data integrity.
/// It uses `Arc` and `RwLock` to ensure thread-safe access to the underlying storage components.
pub struct Storage {
    /// Thread-safe access to block storage
    block_storage: Arc<RwLock<BlockStorage>>,
    /// Thread-safe access to state storage
    state_storage: Arc<RwLock<StateStorage>>,
}

impl Storage {
    /// Creates a new instance of `Storage`.
    ///
    /// This method initializes both block and state storage components.
    ///
    /// # Returns
    ///
    /// * `Storage` - A new instance of `Storage`.
    pub fn new() -> Self {
        Storage {
            block_storage: Arc::new(RwLock::new(BlockStorage::new())),
            state_storage: Arc::new(RwLock::new(StateStorage::new())),
        }
    }

    /// Adds a block to the block storage.
    ///
    /// This method acquires a write lock on the block storage before adding the block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to add.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added, or an `IcnError` otherwise.
    pub fn add_block(&self, block: Block) -> IcnResult<()> {
        let mut storage = self.block_storage.write()
            .map_err(|_| IcnError::Storage("Failed to acquire write lock for block storage".to_string()))?;
        storage.store_block(block)
    }

    /// Retrieves a block from the block storage.
    ///
    /// This method acquires a read lock on the block storage before retrieving the block.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Option<Block>>` - Returns the block if found, or `None` if not found, or an `IcnError` if lock acquisition fails.
    pub fn get_block(&self, hash: &str) -> IcnResult<Option<Block>> {
        let storage = self.block_storage.read()
            .map_err(|_| IcnError::Storage("Failed to acquire read lock for block storage".to_string()))?;
        Ok(storage.retrieve_block(hash))
    }

    /// Updates a state in the state storage.
    ///
    /// This method acquires a write lock on the state storage before updating the state.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the state to update.
    /// * `value` - The value to set for the key.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the state is successfully updated, or an `IcnError` otherwise.
    pub fn update_state(&self, key: &str, value: &str) -> IcnResult<()> {
        let mut storage = self.state_storage.write()
            .map_err(|_| IcnError::Storage("Failed to acquire write lock for state storage".to_string()))?;
        storage.update_state(key, value)
    }

    /// Retrieves a state from the state storage.
    ///
    /// This method acquires a read lock on the state storage before retrieving the state.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the state to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Option<String>>` - Returns the state value if found, or `None` if not found, or an `IcnError` if lock acquisition fails.
    pub fn get_state(&self, key: &str) -> IcnResult<Option<String>> {
        let storage = self.state_storage.read()
            .map_err(|_| IcnError::Storage("Failed to acquire read lock for state storage".to_string()))?;
        Ok(storage.get_state(key))
    }

    /// Verifies the integrity of a block in the block storage.
    ///
    /// This method acquires a read lock on the block storage before verifying the block's integrity.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to verify.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block's integrity is verified, `Ok(false)` if it fails verification, or an `IcnError` if lock acquisition fails or the block is not found.
    pub fn verify_block_integrity(&self, hash: &str) -> IcnResult<bool> {
        let storage = self.block_storage.read()
            .map_err(|_| IcnError::Storage("Failed to acquire read lock for block storage".to_string()))?;
        storage.verify_integrity(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_block() {
        let storage = Storage::new();
        let block = Block::new(0, vec![], "genesis".to_string(), "proposer".to_string());
        let block_hash = block.hash.clone();

        assert!(storage.add_block(block.clone()).is_ok());
        let retrieved_block = storage.get_block(&block_hash).unwrap();
        assert!(retrieved_block.is_some());
        assert_eq!(retrieved_block.unwrap(), block);
    }

    #[test]
    fn test_update_and_retrieve_state() {
        let storage = Storage::new();
        assert!(storage.update_state("key1", "value1").is_ok());
        let retrieved_value = storage.get_state("key1").unwrap();
        assert_eq!(retrieved_value, Some("value1".to_string()));
    }

    #[test]
    fn test_verify_block_integrity() {
        let storage = Storage::new();
        let block = Block::new(0, vec![], "genesis".to_string(), "proposer".to_string());
        let block_hash = block.hash.clone();

        assert!(storage.add_block(block).is_ok());
        assert!(storage.verify_block_integrity(&block_hash).unwrap());
    }
}