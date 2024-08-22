// file: icn_storage/src/block_storage.rs

use std::collections::HashMap;
use icn_shared::{Block, IcnResult, IcnError};

/// The `BlockStorage` struct is responsible for managing the storage of blocks in the blockchain.
/// It ensures data integrity and availability across the network.
pub struct BlockStorage {
    storage: HashMap<String, Block>,
    integrity_checks: HashMap<String, String>,  // Store checksums or other integrity verification data
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
    /// * `Result<(), IcnError>` - Returns `Ok(())` if the block is successfully stored.
    pub fn store_block(&mut self, block: Block) -> IcnResult<()> {
        let block_hash = block.hash.clone();
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
        // Simple example: creating a checksum based on block contents (can be replaced with a more complex algorithm)
        let checksum = format!("{:x}", md5::compute(block.hash.clone() + &block.previous_hash));
        Ok(checksum)
    }
}
