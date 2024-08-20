// icn_blockchain/src/block/mod.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use chrono::Utc;
use crate::transaction::Transaction; // Corrected import path

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub proposer_id: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        proposer_id: String,
    ) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            proposer_id,
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }


    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!(
            "{}{}{}{}{}{}", // Fixed formatting by adding the missing placeholder
            self.index,
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap(),
            self.previous_hash,
            self.proposer_id,
            self.nonce
        ));
        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = vec![0; difficulty];
        let target_str = String::from_utf8(target).unwrap(); // Moved outside the loop

        while !self.hash.starts_with(&target_str) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }

    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}
