//icn_shared/src/block.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a block in the blockchain.
///
/// Each block contains a list of transactions, a reference to the previous block's hash,
/// a unique hash calculated from its contents, and the ID of the proposer who created the block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>, // Transactions are represented as Strings
    pub previous_hash: String,
    pub hash: String,
    pub proposer_id: String,
    pub nonce: u64,
}

impl Block {
    /// Creates a new `Block` instance.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the block in the blockchain.
    /// * `transactions` - A vector of transactions included in the block.
    /// * `previous_hash` - The hash of the previous block in the chain.
    /// * `proposer_id` - The ID of the node proposing the block.
    ///
    /// # Returns
    ///
    /// * `Block` - A new `Block` instance with a calculated hash.
    pub fn new(
        index: u64,
        transactions: Vec<String>,
        previous_hash: String,
        proposer_id: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            proposer_id,
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of the block based on its contents.
    ///
    /// The hash is computed using SHA-256 and includes the block's index, timestamp,
    /// transactions, previous hash, proposer ID, and nonce.
    ///
    /// # Returns
    ///
    /// * `String` - The calculated hash of the block.
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(serde_json::to_string(&self.transactions).unwrap());
        hasher.update(&self.previous_hash);
        hasher.update(&self.proposer_id);
        hasher.update(self.nonce.to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validates the block's hash to ensure it matches the calculated hash.
    ///
    /// This method checks the integrity of the block by verifying that its hash is consistent
    /// with its contents.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the block is valid, `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}
