// icn_blockchain/src/lib.rs

pub mod chain;

pub use crate::chain::Chain;

use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus; // Import Consensus trait from icn_consensus crate
use std::sync::{Arc, Mutex};

/// Represents the blockchain and its operations.
pub struct Blockchain<C: Consensus> {
    pub chain: Vec<Block>,
    pub consensus: Arc<Mutex<C>>,
}

impl<C: Consensus> Blockchain<C> {
    /// Creates a new blockchain with the given consensus algorithm.
    pub fn new(consensus: Arc<Mutex<C>>) -> Self {
        Blockchain {
            chain: vec![Block::new(0, vec![], "genesis".to_string(), "genesis".to_string())],
            consensus,
        }
    }

    /// Adds a new block to the blockchain after validating it.
    pub fn add_block(&mut self, transactions: Vec<String>, proposer_id: String) -> IcnResult<()> {
        let previous_block = self.chain.last().expect("Blockchain is empty");
        let new_block = Block::new(
            self.chain.len() as u64,
            transactions,
            previous_block.hash.clone(),
            proposer_id,
        );

        // Validate the block using the consensus mechanism
        let mut consensus = self.consensus.lock().expect("Failed to lock consensus for validation");
        if consensus.validate(&new_block)? {
            self.chain.push(new_block);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid block".to_string()))
        }
    }

    /// Validates the integrity of the blockchain.
    pub fn is_valid_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if !current_block.is_valid() {
                return false;
            }
        }
        true
    }
}
