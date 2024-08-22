// file: icn_storage/src/lib.rs

use std::sync::{Arc, RwLock};
use icn_shared::{Block, IcnResult, IcnError};

pub mod block_storage;
pub mod state_storage;

use block_storage::BlockStorage;
use state_storage::StateStorage;

/// The `Storage` struct provides an interface to the underlying storage systems
/// for blocks and state. It ensures thread-safe access and modification.
pub struct Storage {
    block_storage: Arc<RwLock<BlockStorage>>,
    state_storage: Arc<RwLock<StateStorage>>,
}

impl Storage {
    /// Creates a new instance of the `Storage` struct with initialized
    /// block and state storage systems.
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

    /// Adds a new block to the block storage.
    ///
    /// # Arguments
    ///
    /// * `block` - The `Block` to be added to the storage.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added,
    ///   or an `IcnError` if a block with the same hash already exists.
    pub fn add_block(&self, block: Block) -> IcnResult<()> {
        let mut storage = self.block_storage.write()
            .map_err(|_| IcnError::Storage("Failed to acquire write lock for block storage".to_string()))?;
        storage.store_block(block)
    }

    /// Retrieves a block by its hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - A string representing the hash of the block to be retrieved.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Option<Block>>` - Returns `Ok(Some(Block))` if a block with the given hash is found,
    ///   or `Ok(None)` if no such block exists.
    pub fn get_block(&self, hash: &str) -> IcnResult<Option<Block>> {
        let storage = self.block_storage.read()
            .map_err(|_| IcnError::Storage("Failed to acquire read lock for block storage".to_string()))?;
        Ok(storage.retrieve_block(hash))
    }

    /// Updates the state storage with the latest state.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which the state is stored.
    /// * `value` - The value of the state to be stored.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the state is successfully updated,
    ///   or an `IcnError` if the update fails.
    pub fn update_state(&self, key: &str, value: &str) -> IcnResult<()> {
        let mut storage = self.state_storage.write()
            .map_err(|_| IcnError::Storage("Failed to acquire write lock for state storage".to_string()))?;
        storage.update_state(key, value)
            .map_err(|e| IcnError::Storage(e))
    }

    /// Retrieves a value from the state storage by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the state to retrieve.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Option<String>>` - Returns `Ok(Some(String))` if the value is found,
    ///   or `Ok(None)` if no value is found for the given key.
    pub fn get_state(&self, key: &str) -> IcnResult<Option<String>> {
        let storage = self.state_storage.read()
            .map_err(|_| IcnError::Storage("Failed to acquire read lock for state storage".to_string()))?;
        Ok(storage.get_state(key))
    }

    /// Verifies the integrity of a block in storage.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to verify.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block's integrity is intact, `Ok(false)` otherwise.
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
        assert_eq!(retrieved_block, Some(block));
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