// icn_storage/src/block_storage.rs

use icn_blockchain::block::Block;  // Correctly reference icn_blockchain

use std::collections::HashMap;

/// The `BlockStorage` struct is responsible for storing and retrieving blocks.
/// It uses an in-memory `HashMap` to store blocks by their hash.
pub struct BlockStorage {
    storage: HashMap<String, Block>,
}

impl BlockStorage {
    /// Creates a new `BlockStorage` instance.
    pub fn new() -> Self {
        BlockStorage {
            storage: HashMap::new(),
        }
    }

    /// Adds a new block to the storage.
    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        let hash = block.hash.clone();
        if self.storage.contains_key(&hash) {
            return Err("Block with this hash already exists.".to_string());
        }
        self.storage.insert(hash, block);
        Ok(())
    }

    /// Retrieves a block by its hash.
    pub fn get_block(&self, hash: &str) -> Option<Block> {
        self.storage.get(hash).cloned()
    }
}
