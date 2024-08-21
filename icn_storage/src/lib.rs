// file: icn_storage/src/lib.rs

pub mod block_storage;
pub mod state_storage;

use icn_shared::Block;  // Correctly reference Block from icn_shared
use block_storage::BlockStorage;
use state_storage::StateStorage;
use std::sync::{Arc, RwLock};

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
    /// * `Result<(), String>` - Returns `Ok(())` if the block is successfully added,
    ///   or an error message if a block with the same hash already exists.
    pub fn add_block(&self, block: Block) -> Result<(), String> {
        let mut storage = self.block_storage.write().unwrap();
        storage.add_block(block)
    }

    /// Retrieves a block by its hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - A string representing the hash of the block to be retrieved.
    ///
    /// # Returns
    ///
    /// * `Option<Block>` - Returns `Some(Block)` if a block with the given hash is found,
    ///   or `None` if no such block exists.
    pub fn get_block(&self, hash: &str) -> Option<Block> {
        let storage = self.block_storage.read().unwrap();
        storage.get_block(hash)
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
    /// * `Result<(), String>` - Returns `Ok(())` if the state is successfully updated,
    ///   or an error message if the update fails.
    pub fn update_state(&self, key: &str, value: &str) -> Result<(), String> {
        let mut storage = self.state_storage.write().unwrap();
        storage.update_state(key, value)
    }

    /// Retrieves a value from the state storage by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the state to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - Returns `Some(String)` if the value is found,
    ///   or `None` if no value is found for the given key.
    pub fn get_state(&self, key: &str) -> Option<String> {
        let storage = self.state_storage.read().unwrap();
        storage.get_state(key)
    }
}
