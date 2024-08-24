// File: icn_storage/src/block_storage.rs

use std::collections::HashMap;
use icn_shared::{Block, IcnError, IcnResult};
use sha2::{Sha256, Digest};
use serde_json;

/// `BlockStorage` manages the storage of blockchain blocks.
///
/// This struct provides methods for adding, retrieving, and verifying the
/// integrity of blocks in the blockchain. It uses an in-memory HashMap for
/// storage, making it efficient for quick access but not persistent across
/// program restarts.
pub struct BlockStorage {
    /// Stores blocks with their hash as the key
    storage: HashMap<String, Block>,
    /// Stores integrity checksums for each block
    integrity_checks: HashMap<String, String>,
}

impl BlockStorage {
    /// Creates a new instance of `BlockStorage`.
    ///
    /// # Returns
    ///
    /// * `BlockStorage` - A new, empty instance of `BlockStorage`.
    pub fn new() -> Self {
        BlockStorage {
            storage: HashMap::new(),
            integrity_checks: HashMap::new(),
        }
    }

    /// Stores a block in the storage.
    ///
    /// This method calculates a checksum for the block before storing it,
    /// which can be used later to verify the block's integrity.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to store.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully stored,
    ///   or an `IcnError` if there's an issue (e.g., duplicate block).
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

    /// Retrieves a block from the storage.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// * `Option<Block>` - Returns `Some(Block)` if found, or `None` if not.
    pub fn retrieve_block(&self, hash: &str) -> Option<Block> {
        self.storage.get(hash).cloned()
    }

    /// Verifies the integrity of a block in the storage.
    ///
    /// This method recalculates the checksum for the stored block and compares
    /// it with the stored checksum to ensure the block hasn't been tampered with.
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash of the block to verify.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block's integrity is verified,
    ///   `Ok(false)` if it fails verification, or an `IcnError` if the block is not found.
    pub fn verify_integrity(&self, hash: &str) -> IcnResult<bool> {
        let block = self.retrieve_block(hash)
            .ok_or_else(|| IcnError::Storage("Block not found".to_string()))?;
        
        let stored_checksum = self.integrity_checks.get(hash)
            .ok_or_else(|| IcnError::Storage("Checksum not found".to_string()))?;
        
        let current_checksum = self.calculate_checksum(&block)?;

        Ok(stored_checksum == &current_checksum)
    }

    /// Calculates the checksum for a block.
    ///
    /// This method uses SHA-256 to create a unique checksum based on the block's contents.
    ///
    /// # Arguments
    ///
    /// * `block` - The block for which to calculate the checksum.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the checksum as a string, or an `IcnError` if serialization fails.
    fn calculate_checksum(&self, block: &Block) -> IcnResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(&block.index.to_be_bytes());
        hasher.update(&block.timestamp.to_be_bytes());
        hasher.update(serde_json::to_string(&block.transactions)
            .map_err(|e| IcnError::Storage(format!("Failed to serialize transactions: {}", e)))?);
        hasher.update(&block.previous_hash);
        hasher.update(&block.proposer_id);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Returns the number of blocks stored.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of blocks stored.
    pub fn block_count(&self) -> usize {
        self.storage.len()
    }

    /// Checks if a block exists in the storage.
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
        assert!(retrieved_block.is_some());
        assert_eq!(retrieved_block.unwrap(), block);
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