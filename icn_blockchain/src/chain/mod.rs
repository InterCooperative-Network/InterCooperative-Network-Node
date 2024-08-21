use std::collections::VecDeque;
use icn_shared::{Block, IcnResult, IcnError};
use icn_consensus::Consensus;

/// Represents the blockchain, which is a sequence of blocks.
///
/// The `Chain` interacts with a consensus mechanism to validate and add new blocks.
/// The consensus mechanism is abstracted through the `Consensus` trait.
pub struct Chain<C: Consensus> {
    pub blocks: VecDeque<Block>, // Using VecDeque for efficient block operations
    pub consensus: C,
}

impl<C: Consensus> Chain<C> {
    /// Creates a new blockchain with the given consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `consensus` - The consensus mechanism to be used for the blockchain.
    ///
    /// # Returns
    ///
    /// * `Chain<C>` - A new `Chain` instance.
    pub fn new(consensus: C) -> Self {
        Chain {
            blocks: VecDeque::new(), // Initialize blocks as VecDeque
            consensus,
        }
    }

    /// Adds a new block to the blockchain.
    ///
    /// The block is created using the provided transactions and the proposer ID selected
    /// by the consensus mechanism. The block is then validated before being added to the chain.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of transactions to include in the block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added,
    ///   or an `IcnError` if validation fails.
    pub fn add_block(&mut self, transactions: Vec<String>, previous_hash: String, proposer_id: String) -> IcnResult<()> {
        let index = self.blocks.len() as u64;

        let new_block = Block::new(index, transactions, previous_hash, proposer_id);

        if self.consensus.validate(&new_block)? {
            self.blocks.push_back(new_block);
            Ok(())
        } else {
            Err(IcnError::Blockchain("Block validation failed.".to_string()))
        }
    }

    /// Returns the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * `Option<&Block>` - Returns an `Option` containing a reference to the latest block,
    ///   or `None` if the blockchain is empty.
    pub fn latest_block(&self) -> Option<&Block> {
        self.blocks.back()
    }
}
