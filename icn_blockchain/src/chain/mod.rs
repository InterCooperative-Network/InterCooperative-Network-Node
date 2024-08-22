// file: icn_blockchain/src/chain/mod.rs

use icn_shared::{Block, IcnError};
use icn_consensus::Consensus;
use std::sync::Arc;
use rand::Rng;

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
    /// * `peer_id` - A string representing the ID of the peer.
    /// * `stake` - The amount of stake the validator has.
    /// * `reputation` - The reputation score of the validator.
    ///
    /// # Returns
    ///
    /// * `Validator` - A new `Validator` instance.
    fn new(peer_id: String, stake: f64, reputation: f64) -> Self {
        Validator { peer_id, stake, reputation }
    }

    /// Validates a block according to the validator's criteria.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid,
    ///   or an error message if validation fails.
    fn validate(&self, _block: &Block) -> IcnResult<bool> {
        // Placeholder logic; implement actual validation logic here
        Ok(true)
    }

    /// Casts a vote on whether a block should be accepted.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block being voted on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the vote is in favor,
    ///   or `Ok(false)` if the vote is against.
    fn vote(&self, _block: &Block) -> IcnResult<bool> {
        // Placeholder logic; implement actual voting logic here
        Ok(true)
    }
}

/// The `Chain` struct represents the blockchain, which consists of a series of blocks.
/// It manages the addition of new blocks, block validation, and access to the latest block.
pub struct Chain<C: Consensus> {
    pub blocks: Vec<Block>,
    pub consensus: Arc<C>,
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
            consensus,
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

    /// Selects validators for block validation based on the consensus mechanism.
    ///
    /// The selection process may involve factors such as stake, reputation, and recent activity.
    fn select_validators(&self, _block: &Block) -> IcnResult<Vec<Validator>> {
        let eligible_peers = self.consensus.get_eligible_peers();
        let mut rng = rand::thread_rng();

        let validators: Vec<Validator> = eligible_peers
            .iter()
            .map(|peer_id| {
                let stake = rng.gen_range(0.5..1.5);
                let reputation = rng.gen_range(0.5..1.5);
                Validator::new(peer_id.clone(), stake, reputation)
            })
            .collect();

        Ok(validators)
    }

    /// Validates a block according to the consensus mechanism.
    ///
    /// This method ensures that the block meets the criteria set by the consensus mechanism, including
    /// cooperation scores, reputation scores, and other factors.
    pub fn validate_block(&self, block: &Block) -> IcnResult<()> {
        let validators = self.select_validators(block)?;
        for validator in validators {
            let is_valid = validator.validate(block)?;
            if !is_valid {
                return Err(IcnError::Consensus("Block validation failed.".to_string()));
            }
        }
        Ok(())
    }

    /// Performs stake-weighted voting on a block.
    ///
    /// Voting influence is proportional to the validator's stake and reputation, ensuring a fair and balanced decision.
    pub fn stake_weighted_vote(&self, block: &Block) -> IcnResult<bool> {
        let mut total_weight = 0.0;
        let mut weighted_votes = 0.0;

        let validators = self.select_validators(block)?;
        for validator in validators.iter() {
            let weight = validator.stake + validator.reputation;
            let vote = validator.vote(block)?;
            total_weight += weight;
            weighted_votes += (vote as u64) as f64 * weight;
        }

        Ok(weighted_votes / total_weight > 0.5)
    }
}
