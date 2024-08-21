// icn_consensus/src/proof_of_cooperation.rs

use std::collections::{HashMap, HashSet};
use icn_shared::{IcnError, IcnResult, Block};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

/// The `ProofOfCooperation` struct implements the Proof of Cooperation consensus mechanism.
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, f64>,
    last_block_time: u64,
}

// Manually implement the Clone trait for ProofOfCooperation
impl Clone for ProofOfCooperation {
    fn clone(&self) -> Self {
        ProofOfCooperation {
            known_peers: self.known_peers.clone(),
            cooperation_scores: self.cooperation_scores.clone(),
            last_block_time: self.last_block_time,
        }
    }
}

impl ProofOfCooperation {
    /// Creates a new `ProofOfCooperation` instance.
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            last_block_time: 0,
        }
    }

    /// Registers a peer in the consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to register.
    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 1.0);
    }

    /// Checks if a peer is registered in the consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to check.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the peer is registered, otherwise `false`.
    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }

    /// Validates a block according to the Proof of Cooperation consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - `true` if the block is valid, otherwise an `IcnError`.
    pub fn validate(&mut self, block: &Block) -> IcnResult<bool> {
        if !self.is_registered(&block.proposer_id) {
            return Err(IcnError::Consensus(format!("Unknown proposer: {}", block.proposer_id)));
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();

        if current_time < self.last_block_time + 10 {
            return Err(IcnError::Consensus("Block proposed too soon".to_string()));
        }

        // Update last block time
        self.last_block_time = current_time;

        // Additional validation logic here (e.g., checking signatures, ensuring cooperation)
        Ok(true)
    }

    /// Selects a proposer based on cooperation scores.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - The selected proposer ID or an `IcnError` if selection fails.
    pub fn select_proposer(&self) -> IcnResult<String> {
        let mut rng = rand::thread_rng();
        let total_score: f64 = self.cooperation_scores.values().sum();
        let random_value: f64 = rng.gen::<f64>() * total_score;

        let mut cumulative_score = 0.0;
        for (peer_id, score) in &self.cooperation_scores {
            cumulative_score += score;
            if cumulative_score >= random_value {
                return Ok(peer_id.clone());
            }
        }

        Err(IcnError::Consensus("Failed to select a proposer".to_string()))
    }

    /// Updates the cooperation score of a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer.
    /// * `performance` - The performance factor to update the score with.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - An empty result indicating success or an `IcnError` if the peer is unknown.
    pub fn update_cooperation_score(&mut self, peer_id: &str, performance: f64) -> IcnResult<()> {
        let score = self.cooperation_scores
            .get_mut(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        *score = (*score * performance).max(0.1).min(2.0);
        Ok(())
    }

    /// Handles a blockchain fork by selecting the most valid chain.
    ///
    /// # Arguments
    ///
    /// * `chain_a` - The first chain to compare.
    /// * `chain_b` - The second chain to compare.
    ///
    /// # Returns
    ///
    /// * `&[Block]` - The selected chain.
    pub fn handle_fork<'a>(&self, chain_a: &'a [Block], chain_b: &'a [Block]) -> &'a [Block] {
        // Simple longest chain rule for now
        if chain_a.len() >= chain_b.len() {
            chain_a
        } else {
            chain_b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_blockchain::block::Block;

    #[test]
    fn test_register_and_validate_peer() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        
        let block = Block::new(0, 0, vec![], "peer1".to_string(), "hash".to_string(), "peer1".to_string());
        assert!(poc.validate(&block).is_ok());

        let invalid_block = Block::new(0, 0, vec![], "unknown_peer".to_string(), "hash".to_string(), "unknown_peer".to_string());
        assert!(poc.validate(&invalid_block).is_err());
    }

    #[test]
    fn test_select_proposer() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        poc.register_peer("peer2");
        
        let proposer = poc.select_proposer().unwrap();
        assert!(vec!["peer1", "peer2"].contains(&proposer.as_str()));
    }

    #[test]
    fn test_update_cooperation_score() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        
        poc.update_cooperation_score("peer1", 1.5).unwrap();
        assert!(poc.cooperation_scores["peer1"] > 1.0);

        poc.update_cooperation_score("peer1", 0.5).unwrap();
        assert!(poc.cooperation_scores["peer1"] < 1.0);
    }
}