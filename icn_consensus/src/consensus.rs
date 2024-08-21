//file: icn_consensus/src/consensus.rs

use icn_blockchain::block::Block;

/// The `Consensus` trait defines the interface for consensus mechanisms
/// within the InterCooperative Network blockchain system.
///
/// Implementing this trait allows different consensus algorithms to be
/// used interchangeably within the blockchain.
pub trait Consensus {
    /// Validates a block according to the consensus rules.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block that needs to be validated.
    ///
    /// # Returns
    ///
    /// * `Result<bool, String>` - Returns `Ok(true)` if the block is valid,
    ///   or an error message if validation fails.
    fn validate(&self, block: &Block) -> Result<bool, String>;

    /// Selects a proposer for the next block based on the consensus mechanism's rules.
    ///
    /// # Returns
    ///
    /// * `Result<String, String>` - Returns the ID of the selected proposer,
    ///   or an error message if selection fails.
    fn select_proposer(&self) -> Result<String, String>;
}
