// file: icn_storage/src/block_storage.rs

use std::collections::HashMap;
use icn_shared::{Block, IcnResult, IcnError};
use sha2::{Sha256, Digest};

/// The `BlockStorage` struct is responsible for managing the storage of blocks in the blockchain.
/// It ensures data integrity and availability across the network.
pub struct BlockStorage {
    storage: HashMap<String, Block>,
    integrity_checks: HashMap<String, String>, // Store checksums for integrity verification
}

impl BlockStorage {
    /// Creates a new `BlockStorage` instance.
    pub fn new() -> Self {
        BlockStorage {
            storage: HashMap::new(),
            integrity_checks: HashMap::new(),
        }
    }

    /// Stores a block in the storage.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to store.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully stored.
    pub fn store_block(&mut self, block: Block) -> IcnResult<()> {
        let block_hash = block.hash.clone();
        if self.storage.contains_key(&block_hash) {
            return Err(IcnError::Storage("Block with this hash already exists".to_string()));
        }

        let checksum = self.calculate_checksum(&block)?;
        self.storage.insert(block_hash.clone(), block);
        self.integrity_checks.insert(block_hash, checksum);
        Ok(())
    }

    /// Retrieves a block from storage by its hash.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<Block>` - Returns an `Option` containing the block, or `None` if not found.
    pub fn retrieve_block(&self, hash: &str) -> Option<Block> {
        self.storage.get(hash).cloned()
    }

    /// Verifies the integrity of a block using stored checksums.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to verify.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block's integrity is intact, `Ok(false)` otherwise.
    pub fn verify_integrity(&self, hash: &str) -> IcnResult<bool> {
        let block = self.retrieve_block(hash)
            .ok_or_else(|| IcnError::Storage("Block not found".to_string()))?;
        
        let stored_checksum = self.integrity_checks.get(hash)
            .ok_or_else(|| IcnError::Storage("Checksum not found".to_string()))?;
        
        let current_checksum = self.calculate_checksum(&block)?;

        Ok(stored_checksum == &current_checksum)
    }

    /// Calculates a checksum for a block to verify data integrity.
    ///
    /// # Arguments
    ///
    /// * `block` - The block for which to calculate the checksum.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the calculated checksum as a string.
    fn calculate_checksum(&self, block: &Block) -> IcnResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(&block.hash);
        hasher.update(&block.previous_hash);
        // Add more fields as necessary to ensure comprehensive integrity checking
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Returns the total number of blocks in storage.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of blocks in storage.
    pub fn block_count(&self) -> usize {
        self.storage.len()
    }

    /// Checks if a block with the given hash exists in storage.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to check.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the block exists, `false` otherwise.
    pub fn block_exists(&self, hash: &str) -> bool {
        self.storage.contains_key(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_block() {
        let mut storage = BlockStorage::new();
        let block = Block::new(0, vec![], "genesis".to_string(), "proposer".to_string());
        let block_hash = block.hash.clone();

        assert!(storage.store_block(block.clone()).is_ok());
        let retrieved_block = storage.retrieve_block(&block_hash);
        assert_eq!(retrieved_block, Some(block));
    }

    #[test]
    fn test_block_integrity() {
        let mut storage = BlockStorage::new();
        let block = Block::new(0, vec![], "genesis".to_string(), "proposer".to_string());
        let block_hash = block.hash.clone();

        assert!(storage.store_block(block).is_ok());
        assert!(storage.verify_integrity(&block_hash).unwrap());
    }

    #[test]
    fn test_block_count_and_exists() {
        let mut storage = BlockStorage::new();
        let block1 = Block::new(0, vec![], "genesis".to_string(), "proposer".to_string());
        let block2 = Block::new(1, vec![], block1.hash.clone(), "proposer".to_string());

        assert!(storage.store_block(block1.clone()).is_ok());
        assert!(storage.store_block(block2.clone()).is_ok());

        assert_eq!(storage.block_count(), 2);
        assert!(storage.block_exists(&block1.hash));
        assert!(storage.block_exists(&block2.hash));
        assert!(!storage.block_exists("nonexistent_hash"));
    }
}