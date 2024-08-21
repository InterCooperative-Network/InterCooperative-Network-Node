use std::collections::{HashMap, HashSet};
use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// The `ProofOfCooperation` struct implements the Proof of Cooperation consensus mechanism.
#[derive(Debug)]
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, f64>,
    reputation_scores: HashMap<String, f64>,  // New field for reputation scores
    last_block_time: u64,
}

impl ProofOfCooperation {
    /// Creates a new `ProofOfCooperation` instance.
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            reputation_scores: HashMap::new(),
            last_block_time: 0,
        }
    }

    /// Registers a peer in the consensus mechanism.
    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 1.0);
        self.reputation_scores.insert(peer_id.to_string(), 1.0);  // Initialize reputation score
        info!("Registered peer: {}", peer_id);
    }

    /// Checks if a peer is registered in the consensus mechanism.
    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }

    /// Validates a block according to the Proof of Cooperation consensus mechanism.
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

        self.last_block_time = current_time;

        // Example of additional validation logic (e.g., checking signatures, ensuring cooperation)
        // Here, we could also validate the quality of transactions, adherence to protocol rules, etc.

        Ok(true)
    }

    /// Selects a proposer based on cooperation and reputation scores.
    pub fn select_proposer(&self) -> IcnResult<String> {
        let mut rng = rand::thread_rng();
        let total_score: f64 = self.cooperation_scores
            .iter()
            .zip(self.reputation_scores.iter())
            .map(|((_, coop_score), (_, rep_score))| coop_score + rep_score)
            .sum();
        let random_value: f64 = rng.gen::<f64>() * total_score;

        let mut cumulative_score = 0.0;
        for (peer_id, coop_score) in &self.cooperation_scores {
            let rep_score = self.reputation_scores.get(peer_id).unwrap_or(&0.0);
            cumulative_score += coop_score + rep_score;
            if cumulative_score >= random_value {
                return Ok(peer_id.clone());
            }
        }

        Err(IcnError::Consensus("Failed to select a proposer".to_string()))
    }

    /// Updates the cooperation score of a peer.
    ///
    /// Performance is a multiplier that adjusts the cooperation score.
    pub fn update_cooperation_score(&mut self, peer_id: &str, performance: f64) -> IcnResult<()> {
        let score = self.cooperation_scores
            .get_mut(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        *score = (*score * performance).max(0.1).min(2.0);  // Adjust score with a min and max range
        self.update_reputation(peer_id)?;  // Update reputation based on new cooperation score
        Ok(())
    }

    /// Updates the reputation score based on historical cooperation scores.
    pub fn update_reputation(&mut self, peer_id: &str) -> IcnResult<()> {
        let coop_score = *self.cooperation_scores
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        let rep_score = self.reputation_scores
            .entry(peer_id.to_string())
            .or_insert(1.0);

        // For simplicity, we're averaging the reputation, but this could be replaced with a more complex formula
        *rep_score = (*rep_score + coop_score) / 2.0;

        Ok(())
    }

    /// Handles a blockchain fork by selecting the most valid chain.
    ///
    /// This decision can be influenced by the quality of the blocks in each chain, such as cooperation and reputation scores.
    pub fn handle_fork<'a>(&self, chain_a: &'a [Block], chain_b: &'a [Block]) -> &'a [Block] {
        // Basic chain selection logic: select the longer chain
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
    use icn_shared::Block;

    #[test]
    fn test_register_and_validate_peer() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        
        let block = Block::new(0, vec![], "previous_hash".to_string(), "peer1".to_string());
        assert!(poc.validate(&block).is_ok());

        let invalid_block = Block::new(0, vec![], "previous_hash".to_string(), "unknown_peer".to_string());
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

    #[test]
    fn test_update_reputation() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        
        poc.update_cooperation_score("peer1", 1.5).unwrap();
        poc.update_reputation("peer1").unwrap();
        assert!(poc.reputation_scores["peer1"] > 1.0);

        poc.update_cooperation_score("peer1", 0.5).unwrap();
        poc.update_reputation("peer1").unwrap();
        assert!(poc.reputation_scores["peer1"] < 1.0);
    }
}
