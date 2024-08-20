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
    /// Creates a new Consensus instance.
    pub fn new() -> Self {
        Consensus {
            proof_of_cooperation: ProofOfCooperation::new(),
        }
    }

    /// Validates a block using the current consensus mechanism.
    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        self.proof_of_cooperation.validate(block)
    }

    /// Handles a potential fork by comparing two chains.
    pub fn handle_fork(&self, chain_a: &[Block], chain_b: &[Block]) -> IcnResult<Vec<Block>> {
        let chosen_chain = self.proof_of_cooperation.handle_fork(chain_a, chain_b);
        Ok(chosen_chain.to_vec())
    }

    /// Registers a new peer in the consensus mechanism.
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
        
        consensus.register_peer(&proposer_id).unwrap();

        let block = Block::new(0, 0, vec![], proposer_id.clone(), String::new(), String::new());
        assert!(consensus.validate_block(&block).unwrap());
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