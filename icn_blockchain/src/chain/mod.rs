// file: icn_blockchain/src/chain/mod.rs

use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus;
use rand::Rng;  // Import the rand crate to handle random number generation
use std::sync::Arc;

/// The `Validator` struct represents a validator in the blockchain consensus process.
/// It includes the peer ID, stake, reputation, and methods for validation and voting.
#[derive(Debug)]
struct Validator {
    peer_id: String,
    stake: f64,
    reputation: f64,
}

impl Validator {
    /// Creates a new `Validator` instance.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer acting as a validator.
    /// * `stake` - The stake held by the validator.
    /// * `reputation` - The reputation score of the validator.
    ///
    /// # Returns
    ///
    /// * `Validator` - A new instance of `Validator`.
    fn new(peer_id: String, stake: f64, reputation: f64) -> Self {
        Validator {
            peer_id,
            stake,
            reputation,
        }
    }

    /// Validates a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, `Ok(false)` otherwise.
    fn validate(&self, block: &Block) -> IcnResult<bool> {
        // Placeholder logic for block validation
        Ok(true)
    }

    /// Casts a vote on the validity of a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to vote on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` for a positive vote, `Ok(false)` for a negative vote.
    fn vote(&self, block: &Block) -> IcnResult<bool> {
        // Placeholder logic for voting on block validity
        Ok(true)
    }
}

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
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added, 
    ///   or an `IcnError` if validation fails.
    pub fn add_block(&mut self, transactions: Vec<String>, previous_hash: String, proposer_id: String) -> IcnResult<()> {
        let index = self.blocks.len() as u64;

        let new_block = Block::new(index, transactions, previous_hash, proposer_id);

        if self.validate_block(&new_block)? {
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
    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        let validators = self.select_validators(block)?;
        for validator in validators {
            let is_valid = validator.validate(block)?;
            if !is_valid {
                return Err(IcnError::Consensus("Block validation failed.".to_string()));
            }
        }
        Ok(true)
    }

    /// Selects validators for block validation based on the consensus mechanism.
    ///
    /// The selection process may involve factors such as stake, reputation, and recent activity.
    fn select_validators(&self, block: &Block) -> IcnResult<Vec<Validator>> {
        let mut validators = Vec::new();
        let eligible_peers = self.consensus.get_eligible_peers();  // Assuming this method is defined in the Consensus trait
        let mut rng = rand::thread_rng();

        for _ in 0..3 {
            if let Some(peer_id) = eligible_peers.choose(&mut rng) {
                let stake = 1.0;  // Placeholder for actual stake calculation
                let reputation = 1.0;  // Placeholder for actual reputation calculation
                validators.push(Validator::new(peer_id.to_string(), stake, reputation));
            }
        }

        if validators.is_empty() {
            return Err(IcnError::Consensus("No eligible validators found.".to_string()));
        }

        Ok(validators)
    }

    /// Performs stake-weighted voting on a block.
    ///
    /// Voting influence is proportional to the validator's stake and reputation, ensuring a fair and balanced decision.
    pub fn stake_weighted_vote(&self, block: &Block) -> IcnResult<bool> {
        let mut total_weight = 0.0;
        let mut weighted_votes = 0.0;

        // Assuming `validators` is a Vec<Validator> returned by `select_validators`
        let validators = self.select_validators(block)?;

        for validator in validators.iter() {
            let weight = validator.stake + validator.reputation;
            let vote = validator.vote(block)?;
            total_weight += weight;
            weighted_votes += vote as f64 * weight;
        }

        Ok(weighted_votes / total_weight > 0.5)
    }
}
