// File: icn_blockchain/src/chain/mod.rs
// Description: This file defines the Chain structure for the blockchain, 
// including functions to manage blocks, validators, and consensus.

use std::sync::{Arc, RwLock};
use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus;
use rand::rngs::OsRng;
use rand::Rng;

/// Represents a validator in the blockchain network.
#[derive(Debug, Clone)]
pub struct Validator {
    /// The unique identifier of the validator.
    pub id: String,
    /// The amount of stake the validator has in the network.
    pub stake: u64,
    /// The reputation score of the validator.
    pub reputation: f64,
    /// The validator's uptime as a percentage.
    pub uptime: f64,
    /// The validator's past performance score.
    pub past_performance: f64,
}

impl Validator {
    /// Creates a new `Validator` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the validator.
    /// * `stake` - The amount of stake the validator has.
    /// * `reputation` - The reputation score of the validator.
    /// * `uptime` - The validator's uptime as a percentage.
    /// * `past_performance` - The validator's past performance score.
    pub fn new(id: String, stake: u64, reputation: f64, uptime: f64, past_performance: f64) -> Self {
        Validator {
            id,
            stake,
            reputation,
            uptime,
            past_performance,
        }
    }

    /// Validates a block based on the consensus rules.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, otherwise returns an error.
    pub fn validate(&self, block: &Block) -> IcnResult<bool> {
        if !self.verify_block_hash(block) {
            return Err(IcnError::Consensus("Invalid block hash".to_string()));
        }

        if !self.verify_transactions(block) {
            return Err(IcnError::Consensus("Invalid transactions".to_string()));
        }

        if !self.verify_timestamp(block) {
            return Err(IcnError::Consensus("Invalid timestamp".to_string()));
        }

        Ok(true)
    }

    /// Verifies the block's hash.
    fn verify_block_hash(&self, block: &Block) -> bool {
        block.hash == block.calculate_hash()
    }

    /// Verifies the transactions in the block.
    fn verify_transactions(&self, block: &Block) -> bool {
        !block.transactions.is_empty()
    }

    /// Verifies the block's timestamp.
    fn verify_timestamp(&self, block: &Block) -> bool {
        block.timestamp > 0
    }

    /// Casts a vote on a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to vote on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the vote is positive, otherwise returns `Ok(false)`.
    pub fn vote(&self, block: &Block) -> IcnResult<bool> {
        self.validate(block)
    }
}

/// Represents the blockchain, which consists of a series of blocks.
pub struct Chain<C: Consensus> {
    /// The list of blocks in the chain.
    pub blocks: Vec<Block>,
    /// The consensus mechanism used for the chain.
    pub consensus: Arc<RwLock<C>>,
    /// The list of active validators.
    pub validators: Vec<Validator>,
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
    pub fn new(consensus: Arc<RwLock<C>>) -> Self {
        Chain {
            blocks: Vec::new(),
            consensus,
            validators: Vec::new(),
        }
    }

    /// Adds a new block to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to be added to the chain.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added,
    ///   or an `IcnError` if validation fails.
    pub fn add_block(&mut self, block: Block) -> IcnResult<()> {
        let mut consensus = self.consensus.write().map_err(|_| {
            IcnError::Consensus("Failed to acquire write lock on consensus".to_string())
        })?;
        
        if consensus.validate(&block)? {
            self.blocks.push(block);
            consensus.update_state(self)?;
            Ok(())
        } else {
            Err(IcnError::Consensus("Block validation failed".to_string()))
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

    /// Returns the number of blocks in the chain.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of blocks in the chain.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Selects validators for block validation based on the consensus mechanism's rules.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<Validator>>` - Returns a vector of selected validators.
    pub fn select_validators(&self) -> IcnResult<Vec<Validator>> {
        let mut rng = OsRng;
        let mut selected_validators = Vec::new();

        for validator in &self.validators {
            let selection_score = validator.stake as f64 * validator.reputation * validator.uptime * validator.past_performance;
            if rng.gen::<f64>() < selection_score {
                selected_validators.push(validator.clone());
            }
        }

        if selected_validators.is_empty() {
            Err(IcnError::Consensus("No validators selected".to_string()))
        } else {
            Ok(selected_validators)
        }
    }

    /// Performs stake-weighted voting on a block.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to vote on.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the vote passes, otherwise returns `Ok(false)`.
    pub fn stake_weighted_vote(&self, block: &Block) -> IcnResult<bool> {
        let validators = self.select_validators()?;
        let mut total_stake = 0u64;
        let mut positive_stake = 0u64;

        for validator in validators {
            if validator.vote(block)? {
                positive_stake += validator.stake;
            }
            total_stake += validator.stake;
        }

        if total_stake == 0 {
            return Err(IcnError::Consensus("No stake in voting validators".to_string()));
        }

        // Calculate the percentage of positive votes weighted by stake
        let approval_percentage = (positive_stake as f64 / total_stake as f64) * 100.0;

        // Require a 2/3 majority for the vote to pass
        Ok(approval_percentage >= 66.67)
    }

    /// Validates the integrity of the entire blockchain.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the blockchain is valid, otherwise false.
    pub fn is_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if !current_block.is_valid() {
                return false;
            }
        }
        true
    }

    /// Adds a new validator to the network.
    ///
    /// # Arguments
    ///
    /// * `validator` - The validator to add.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the validator is successfully added, otherwise an error.
    pub fn add_validator(&mut self, validator: Validator) -> IcnResult<()> {
        if self.validators.iter().any(|v| v.id == validator.id) {
            return Err(IcnError::Consensus("Validator already exists".to_string()));
        }
        self.validators.push(validator);
        Ok(())
    }

    /// Updates a validator's information.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the validator to update.
    /// * `stake` - The new stake amount.
    /// * `reputation` - The new reputation score.
    /// * `uptime` - The new uptime percentage.
    /// * `past_performance` - The new past performance score.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the validator is successfully updated, otherwise an error.
    pub fn update_validator(&mut self, id: &str, stake: u64, reputation: f64, uptime: f64, past_performance: f64) -> IcnResult<()> {
        if let Some(validator) = self.validators.iter_mut().find(|v| v.id == id) {
            validator.stake = stake;
            validator.reputation = reputation;
            validator.uptime = uptime;
            validator.past_performance = past_performance;
            Ok(())
        } else {
            Err(IcnError::Consensus("Validator not found".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_consensus::ProofOfCooperation;

    fn create_test_block() -> Block {
        Block::new(0, vec![], "genesis".to_string(), "test_proposer".to_string())
    }

    #[test]
    fn test_chain_creation() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let chain = Chain::new(consensus);
        assert_eq!(chain.block_count(), 0);
    }

    #[test]
    fn test_add_block() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let mut chain = Chain::new(consensus);
        let block = create_test_block();
        assert!(chain.add_block(block).is_ok());
        assert_eq!(chain.block_count(), 1);
    }

    #[test]
    fn test_chain_validity() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let mut chain = Chain::new(consensus);
        let block1 = create_test_block();
        let block2 = Block::new(1, vec![], block1.hash.clone(), "test_proposer".to_string());
        
        assert!(chain.add_block(block1).is_ok());
        assert!(chain.add_block(block2).is_ok());
        assert!(chain.is_valid());
    }

    #[test]
    fn test_validator_management() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let mut chain = Chain::new(consensus);
        let validator = Validator::new("test_validator".to_string(), 100, 0.95, 0.99, 0.98);
        
        assert!(chain.add_validator(validator).is_ok());
        assert!(chain.update_validator("test_validator", 200, 0.96, 0.995, 0.99).is_ok());
        assert!(chain.update_validator("non_existent", 100, 0.9, 0.9, 0.9).is_err());
    }

    #[test]
    fn test_stake_weighted_vote() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let mut chain = Chain::new(consensus);
        
        chain.add_validator(Validator::new("validator1".to_string(), 100, 1.0, 1.0, 1.0)).unwrap();
        chain.add_validator(Validator::new("validator2".to_string(), 200, 1.0, 1.0, 1.0)).unwrap();
        chain.add_validator(Validator::new("validator3".to_string(), 300, 1.0, 1.0, 1.0)).unwrap();

        let block = create_test_block();
        let vote_result = chain.stake_weighted_vote(&block);
        assert!(vote_result.is_ok());
    }

    #[test]
    fn test_select_validators() {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        let mut chain = Chain::new(consensus);
        
        chain.add_validator(Validator::new("validator1".to_string(), 100, 1.0, 1.0, 1.0)).unwrap();
        chain.add_validator(Validator::new("validator2".to_string(), 200, 1.0, 1.0, 1.0)).unwrap();
        
        let selected_validators = chain.select_validators();
        assert!(selected_validators.is_ok());
        assert!(!selected_validators.unwrap().is_empty());
    }
}
