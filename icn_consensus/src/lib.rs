use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// The `Consensus` trait defines the interface for consensus mechanisms
/// within the InterCooperative Network blockchain system.
///
/// Implementing this trait allows different consensus algorithms to be
/// used interchangeably within the blockchain.
pub trait Consensus: Clone + Send + Sync {
    /// Validates a block according to the consensus rules.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block that needs to be validated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid,
    ///   or an error message if validation fails.
    fn validate(&self, block: &Block) -> IcnResult<bool>;

    /// Selects a proposer for the next block based on the consensus mechanism's rules.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the ID of the selected proposer,
    ///   or an error message if selection fails.
    fn select_proposer(&self) -> IcnResult<String>;

    /// Retrieves the list of eligible peers for proposer selection.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector containing the IDs of eligible peers.
    fn get_eligible_peers(&self) -> Vec<String>;
}

/// The `ProofOfCooperation` struct implements the Proof of Cooperation consensus mechanism.
/// It manages the registration of peers, validation of blocks, selection of proposers,
/// and updates of cooperation and reputation scores.
#[derive(Clone, Debug)]
pub struct ProofOfCooperation {
    /// A set of known peers in the network.
    known_peers: HashSet<String>,
    /// A map of peer IDs to their cooperation scores.
    cooperation_scores: HashMap<String, f64>,
    /// A map of peer IDs to their reputation scores.
    reputation_scores: HashMap<String, f64>,
    /// A history of contributions for each peer, stored as a vector of timestamp and score pairs.
    contribution_history: HashMap<String, Vec<(u64, f64)>>,
    /// The timestamp of the last block's creation.
    last_block_time: u64,
}

impl ProofOfCooperation {
    /// Creates a new `ProofOfCooperation` instance.
    ///
    /// # Returns
    ///
    /// * `ProofOfCooperation` - A new instance of the Proof of Cooperation consensus mechanism.
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            reputation_scores: HashMap::new(),
            contribution_history: HashMap::new(),
            last_block_time: 0,
        }
    }

    /// Registers a peer in the consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to be registered.
    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 1.0);
        self.reputation_scores.insert(peer_id.to_string(), 1.0);
        self.contribution_history.insert(peer_id.to_string(), Vec::new());
        info!("Registered peer: {}", peer_id);
    }

    /// Checks if a peer is registered in the consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to check.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the peer is registered, `false` otherwise.
    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }

    /// Validates a block according to the Proof of Cooperation consensus mechanism.
    ///
    /// This function checks the proposer ID, ensures that the block is not proposed too soon after the previous block,
    /// and may include additional validation logic such as checking signatures or cooperation.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block that needs to be validated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, or an error message if validation fails.
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

        Ok(true)
    }

    /// Selects a proposer based on cooperation and reputation scores.
    ///
    /// The selection process is weighted by both cooperation and reputation scores, ensuring that
    /// nodes that contribute positively to the network have a higher chance of being selected as the proposer.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the ID of the selected proposer, or an error message if selection fails.
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

    /// Records a peer's contribution to the network, tracking consistency and quality over time.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer.
    /// * `score` - The contribution score to record.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the contribution is successfully recorded, or an error message if it fails.
    fn record_contribution(&mut self, peer_id: &str, score: f64) -> IcnResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();
        let history = self.contribution_history
            .get_mut(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        history.push((timestamp, score));
        Ok(())
    }

    /// Calculates the consistency of a peer's contributions over time.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose consistency is to be calculated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - Returns the consistency score as a floating-point number, or an error message if it fails.
    fn calculate_consistency(&self, peer_id: &str) -> IcnResult<f64> {
        let history = self.contribution_history
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        if history.is_empty() {
            return Ok(1.0);
        }

        let mean: f64 = history.iter().map(|&(_, score)| score).sum::<f64>() / history.len() as f64;
        let variance: f64 = history.iter().map(|&(_, score)| (score - mean).powi(2)).sum::<f64>() / history.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

    /// Updates the reputation score based on historical cooperation scores and consistency.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose reputation is to be updated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the reputation is successfully updated, or an error message if it fails.
    pub fn update_reputation(&mut self, peer_id: &str) -> IcnResult<()> {
        let coop_score = *self.cooperation_scores
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        let consistency = self.calculate_consistency(peer_id)?;

        let rep_score = self.reputation_scores
            .entry(peer_id.to_string())
            .or_insert(1.0);

        *rep_score = (*rep_score + coop_score * consistency) / 2.0;

        Ok(())
    }

    /// Updates the cooperation score for a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose cooperation score is to be updated.
    /// * `new_score` - The new cooperation score to assign.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the cooperation score is successfully updated, or an error message if it fails.
    pub fn update_cooperation_score(&mut self, peer_id: &str, new_score: f64) -> IcnResult<()> {
        if let Some(score) = self.cooperation_scores.get_mut(peer_id) {
            *score = (*score + new_score) / 2.0;
            Ok(())
        } else {
            Err(IcnError::Consensus(format!("Unknown peer: {}", peer_id)))
        }
    }

    /// Returns a list of eligible peers for the selection process.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector containing the IDs of eligible peers.
    fn get_eligible_peers(&self) -> Vec<String> {
        self.known_peers.iter().cloned().collect()
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
        poc.update_cooperation_score("peer1", 0.5).unwrap();
        assert!(poc.cooperation_scores["peer1"] < 1.5);
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
