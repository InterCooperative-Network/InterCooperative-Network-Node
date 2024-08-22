// file: icn_blockchain/src/chain/mod.rs

use icn_shared::{Block, IcnError};
use icn_consensus::Consensus;
use std::sync::Arc;

/// The `Chain` struct represents the blockchain, which consists of a series of blocks.
/// It manages the addition of new blocks, block validation, and access to the latest block.
pub struct Chain<C: Consensus> {
    pub blocks: Vec<Block>,
    pub consensus: Arc<C>,  // Use Arc<C> instead of C directly
}

impl<C: Consensus> Chain<C> {
    /// Creates a new blockchain with the given consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `consensus` - An `Arc` to the consensus mechanism to be used for the blockchain.
    ///
    /// # Returns
    ///
    /// * `Chain<C>` - A new `Chain` instance.
    pub fn new(consensus: Arc<C>) -> Self {
        Chain {
            blocks: Vec::new(),
            consensus,  // Now stores an Arc<C>
        }
    }

    /// Adds a new block to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of transactions to include in the block.
    /// * `previous_hash` - The hash of the previous block in the chain.
    /// * `proposer_id` - The ID of the proposer of the block.
    ///
    /// # Returns
    ///
    /// * `Result<(), IcnError>` - Returns `Ok(())` if the block is successfully added, 
    ///   or an `IcnError` if validation fails.
    pub fn add_block(&mut self, transactions: Vec<String>, previous_hash: String, proposer_id: String) -> Result<(), IcnError> {
        let index = self.blocks.len() as u64;

        let new_block = Block::new(index, transactions, previous_hash, proposer_id);

        // Propagate the custom error type directly, removing the need for String conversion
        if self.consensus.validate(&new_block)? {
            self.blocks.push(new_block);
            Ok(())
        } else {
            Err(IcnError::Consensus("Block validation failed.".to_string()))
        }
    }

    /// Returns the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * `Option<&Block>` - Returns an `Option` containing a reference to the latest block,
    ///   or `None` if the blockchain is empty.
    pub fn latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Validates a block according to the consensus mechanism.
    ///
    /// This method ensures that the block meets the criteria set by the consensus mechanism, including
    /// cooperation scores, reputation scores, and other factors.
    pub fn validate_block(&self, block: &Block) -> Result<(), IcnError> {
        let validators = self.consensus.select_validators(block)?;
        for validator in validators {
            let is_valid = validator.validate(block)?;
            if !is_valid {
                return Err(IcnError::Consensus("Block validation failed.".to_string()));
            }
        }
        Ok(())
    }

    /// Selects validators for block validation based on the consensus mechanism.
    ///
    /// The selection process may involve factors such as stake, reputation, and recent activity.
    fn select_validators(&self, block: &Block) -> Result<Vec<Validator>, IcnError> {
        // Logic to select validators based on stake, reputation, and recent activity
        Ok(vec![]) // Placeholder
    }

    /// Performs stake-weighted voting on a block.
    ///
    /// Voting influence is proportional to the validator's stake and reputation, ensuring a fair and balanced decision.
    pub fn stake_weighted_vote(&self, block: &Block) -> Result<bool, IcnError> {
        let mut total_weight = 0.0;
        let mut weighted_votes = 0.0;

        for validator in self.validators.iter() {
            let weight = validator.stake + validator.reputation;
            let vote = validator.vote(block)?;
            total_weight += weight;
            weighted_votes += vote as f64 * weight;
        }

        Ok(weighted_votes / total_weight > 0.5)
    }
}
