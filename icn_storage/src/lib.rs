// icn_storage/src/lib.rs

pub mod block_storage;
pub mod state_storage;

use icn_blockchain::block::Block;  // Correctly reference icn_blockchain
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
    pub fn new() -> Self {
        Storage {
            block_storage: Arc::new(RwLock::new(BlockStorage::new())),
            state_storage: Arc::new(RwLock::new(StateStorage::new())),
        }
    }

    /// Adds a new block to the block storage.
    pub fn add_block(&self, block: Block) -> Result<(), String> {
        let mut storage = self.block_storage.write().unwrap();
        storage.add_block(block)
    }

    /// Retrieves a block by its hash.
    pub fn get_block(&self, hash: &str) -> Option<Block> {
        let storage = self.block_storage.read().unwrap();
        storage.get_block(hash)
    }

    /// Updates the state storage with the latest state.
    pub fn update_state(&self, key: &str, value: &str) -> Result<(), String> {
        let mut storage = self.state_storage.write().unwrap();
        storage.update_state(key, value)
    }

    /// Retrieves a value from the state storage by its key.
    pub fn get_state(&self, key: &str) -> Option<String> {
        let storage = self.state_storage.read().unwrap();
        storage.get_state(key)
    }
}
