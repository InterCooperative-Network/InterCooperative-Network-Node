use icn_shared::{Block, IcnError, IcnResult};
use crate::transaction::Transaction; // Assuming Transaction is defined in this crate
use icn_consensus::Consensus;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// Represents the blockchain, which is a sequence of blocks.
pub struct Chain<C: Consensus> {
    pub blocks: VecDeque<Block>,
    pub consensus: C,
}

impl<C: Consensus> Chain<C> {
    /// Creates a new blockchain with the given consensus mechanism.
    pub fn new(consensus: C) -> Self {
        Chain {
            blocks: VecDeque::new(),
            consensus,
        }
    }

    /// Adds a new block to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of `Transaction` instances to include in the block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added, or an `IcnError` if validation fails.
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> IcnResult<()> {
        let proposer_id = self.consensus.select_proposer()?;

        let previous_block = self.latest_block();
        let previous_hash = match previous_block {
            Some(block) => block.hash.clone(),
            None => "0".to_string(), // Genesis block case
        };

        let index = match previous_block {
            Some(block) => block.index + 1,
            None => 0, // Genesis block case
        };

        // Convert transactions to strings for inclusion in the block
        let transactions_strings = transactions.iter().map(|tx| serde_json::to_string(tx).unwrap()).collect();

        let new_block = Block::new(
            index,
            transactions_strings,
            previous_hash,
            proposer_id,
        );

        if self.consensus.validate(&new_block)? {
            self.blocks.push_back(new_block);
            info!("Block added successfully.");
            Ok(())
        } else {
            Err(IcnError::Blockchain("Block validation failed.".to_string()))
        }
    }

    /// Returns the latest block in the blockchain.
    pub fn latest_block(&self) -> Option<&Block> {
        self.blocks.back()
    }
}
