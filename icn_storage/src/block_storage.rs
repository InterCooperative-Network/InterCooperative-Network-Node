use icn_blockchain::block::Block;  // Correctly reference icn_blockchain
use std::collections::HashMap;

/// The `BlockStorage` struct is responsible for storing and retrieving blocks.
/// It uses an in-memory `HashMap` to store blocks by their hash.
pub struct BlockStorage {
    storage: HashMap<String, Block>,
}

impl BlockStorage {
    /// Creates a new `BlockStorage` instance.
    ///
    /// # Returns
    ///
    /// * `BlockStorage` - A new instance of `BlockStorage`.
    pub fn new() -> Self {
        BlockStorage {
            storage: HashMap::new(),
        }
    }

    /// Adds a new block to the storage.
    ///
    /// # Arguments
    ///
    /// * `block` - The `Block` to be added to the storage.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Returns `Ok(())` if the block is successfully added,
    ///   or an error message if a block with the same hash already exists.
    ///
    /// # Errors
    ///
    /// * Returns an error if a block with the same hash already exists in the storage.
    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        let hash = block.hash.clone();
        if self.storage.contains_key(&hash) {
            return Err("Block with this hash already exists.".to_string());
        }
        self.storage.insert(hash, block);
        Ok(())
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
    ///
    /// # Example
    ///
    /// ```
    /// let storage = BlockStorage::new();
    /// let block = Block::new(...);
    /// storage.add_block(block.clone()).unwrap();
    /// let retrieved_block = storage.get_block(&block.hash);
    /// assert_eq!(retrieved_block, Some(block));
    /// ```
    pub fn get_block(&self, hash: &str) -> Option<Block> {
        self.storage.get(hash).cloned()
    }
}
