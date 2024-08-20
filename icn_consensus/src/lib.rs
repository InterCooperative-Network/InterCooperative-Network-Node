// icn_consensus/src/lib.rs
use icn_blockchain::block::Block;
use icn_shared::{IcnError, IcnResult};

pub mod proof_of_cooperation;
use proof_of_cooperation::ProofOfCooperation;

/// The Consensus struct manages the consensus mechanism for the blockchain.
pub struct Consensus {
    proof_of_cooperation: ProofOfCooperation,
}

impl Consensus {
    /// Creates a new Consensus instance with a new ProofOfCooperation.
    pub fn new() -> Self {
        Consensus {
            proof_of_cooperation: ProofOfCooperation::new(),
        }
    }

    /// Validates a `Block` using the `ProofOfCooperation` algorithm
    ///
    /// # Arguments
    ///
    /// * `block` - The block to be validated
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - `true` if the block is valid, otherwise an `IcnError`
    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        let proposer_id = &block.proposer_id;
        if self.proof_of_cooperation.is_registered(proposer_id) {
            self.proof_of_cooperation.validate(block)
        } else {
            Err(IcnError::Consensus(format!("Unknown proposer: {}", proposer_id)))
        }
    }

    /// Handles a potential fork in the blockchain by selecting the most valid chain
    /// according to the `ProofOfCooperation` algorithm
    ///
    /// # Arguments
    ///
    /// * `chain_a` - The first chain to be compared
    /// * `chain_b` - The second chain to be compared
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<Block>>` - The chosen chain as a vector of blocks, or an `IcnError`
    pub fn handle_fork(&self, chain_a: &[Block], chain_b: &[Block]) -> IcnResult<Vec<Block>> {
        let chosen_chain = self.proof_of_cooperation.handle_fork(chain_a, chain_b);
        Ok(chosen_chain.to_vec())
    }

    /// Registers a new peer in the `ProofOfCooperation` consensus mechanism
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to be registered
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or failure
    pub fn register_peer(&mut self, peer_id: &str) -> IcnResult<()> {
        self.proof_of_cooperation.register_peer(peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_blockchain::block::Block;

    #[test]
    fn test_consensus_creation() {
        let mut consensus = Consensus::new();
        let proposer_id = "peer1".to_string();
        
        consensus.register_peer(&proposer_id).unwrap();  // Ensure proposer is registered

        let block = Block::new(0, 0, vec![], proposer_id.clone(), String::new(), String::new());

        // Validate the block
        let validation_result = consensus.validate_block(&block);

        // Clone the result to avoid borrowing issues
        let is_valid = validation_result.as_ref().is_ok();

        // Debugging output to understand why the validation might fail
        if let Err(e) = validation_result {
            println!("Validation failed: {:?}", e);
        }

        assert!(is_valid);
    }

    #[test]
    fn test_handle_fork() {
        let consensus = Consensus::new();
        let chain_a = vec![Block::new(0, 0, vec![], String::new(), String::new(), String::new())];
        let chain_b = vec![
            Block::new(0, 0, vec![], String::new(), String::new(), String::new()),
            Block::new(1, 0, vec![], String::new(), String::new(), String::new()),
        ];
        let result = consensus.handle_fork(&chain_a, &chain_b).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_register_peer() {
        let mut consensus = Consensus::new();
        assert!(consensus.register_peer("peer1").is_ok());
    }
}
