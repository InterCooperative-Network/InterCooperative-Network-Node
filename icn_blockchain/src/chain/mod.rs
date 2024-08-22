// icn_blockchain/src/chain/mod.rs

// Move these `use` statements to the top of the file
use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus; // Ensure this import is correct
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
    /// * `peer_id` - The ID of the peer.
    /// * `stake` - The stake of the peer.
    /// * `reputation` - The reputation of the peer.
    fn new(peer_id: String, stake: f64, reputation: f64) -> Self {
        Validator { peer_id, stake, reputation }
    }

    /// Validates a block based on the consensus rules.
    ///
    /// # Arguments
    ///
    /// * `_block` - The block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, otherwise returns an error.
    fn validate(&self, _block: &Block) -> IcnResult<bool> {
        // Placeholder logic; implement actual validation logic here
        Ok(true)
    }

    /// Casts a vote on a block.
    ///
    /// # Arguments
    ///
    /// * `_block` - The block to vote on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the vote is positive, otherwise returns an error.
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

        // The following line now works since `Consensus` is correctly imported at the top level
        self.consensus.validate(&new_block)?;
        self.blocks.push(new_block);
        Ok(())
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
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<Validator>>` - Returns a vector of selected validators.
    fn select_validators(&self) -> IcnResult<Vec<Validator>> {
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
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is valid, otherwise returns an error.
    pub fn validate_block(&self, block: &Block) -> IcnResult<()> {
        let validators = self.select_validators()?;
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
    ///
    /// # Arguments
    ///
    /// * `block` - The block to vote on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the vote passes, otherwise returns an error.
    pub fn stake_weighted_vote(&self, block: &Block) -> IcnResult<bool> {
        let mut total_weight = 0.0;
        let mut weighted_votes = 0.0;

        let validators = self.select_validators()?;
        for validator in validators.iter() {
            let weight = validator.stake + validator.reputation;
            let vote = validator.vote(block)?;
            total_weight += weight;
            weighted_votes += (vote as u64) as f64 * weight;
        }

        Ok(weighted_votes / total_weight > 0.5)
    }
}
