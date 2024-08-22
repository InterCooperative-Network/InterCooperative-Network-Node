// File: icn_consensus/src/proof_of_cooperation.rs

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use log::{info, warn, error};

/// Constants for reputation calculation
const REPUTATION_DECAY_FACTOR: f64 = 0.95;
const CONSISTENCY_WEIGHT: f64 = 0.3;
const QUALITY_WEIGHT: f64 = 0.4;
const NETWORK_IMPACT_WEIGHT: f64 = 0.3;

/// The `ProofOfCooperation` struct implements the Proof of Cooperation consensus mechanism.
/// It manages peer registration, block validation, proposer selection, and reputation management.
#[derive(Clone, Debug)]
pub struct ProofOfCooperation {
    /// Set of known peers in the network
    known_peers: HashSet<String>,
    /// Map of peer IDs to their cooperation scores
    cooperation_scores: HashMap<String, f64>,
    /// Map of peer IDs to their reputation scores
    reputation_scores: HashMap<String, f64>,
    /// History of contributions for each peer (timestamp and score pairs)
    contribution_history: HashMap<String, Vec<(u64, f64)>>,
    /// Information about staked assets for each peer
    stake_info: HashMap<String, StakeInfo>,
    /// Computational power metrics for each peer
    computational_power: HashMap<String, ComputationalPower>,
    /// Storage provision data for each peer
    storage_provision: HashMap<String, StorageProvision>,
    /// Governance participation records for each peer
    governance_participation: HashMap<String, GovernanceParticipation>,
    /// Timestamp of the last block's creation
    last_block_time: u64,
}

/// Represents staking information for a peer
#[derive(Clone, Debug)]
struct StakeInfo {
    /// Amount of tokens staked
    amount: u64,
    /// Type of asset staked
    asset_type: String,
    /// Duration of the stake
    duration: u64,
}

/// Represents computational power metrics for a peer
#[derive(Clone, Debug)]
struct ComputationalPower {
    /// CPU power in arbitrary units
    cpu_power: u64,
    /// GPU power in arbitrary units
    gpu_power: u64,
    /// List of specialized hardware capabilities
    specialized_hardware: Vec<String>,
}

/// Represents storage provision data for a peer
#[derive(Clone, Debug)]
struct StorageProvision {
    /// Storage capacity in bytes
    capacity: u64,
    /// Reliability score (0.0 to 1.0)
    reliability: f64,
}

/// Represents governance participation records for a peer
#[derive(Clone, Debug)]
struct GovernanceParticipation {
    /// Number of proposals submitted
    proposals_submitted: u64,
    /// Number of votes cast
    votes_cast: u64,
    /// Number of discussions participated in
    discussions_participated: u64,
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
            stake_info: HashMap::new(),
            computational_power: HashMap::new(),
            storage_provision: HashMap::new(),
            governance_participation: HashMap::new(),
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
        self.stake_info.insert(peer_id.to_string(), StakeInfo {
            amount: 0,
            asset_type: "ICN".to_string(),
            duration: 0,
        });
        self.computational_power.insert(peer_id.to_string(), ComputationalPower {
            cpu_power: 0,
            gpu_power: 0,
            specialized_hardware: Vec::new(),
        });
        self.storage_provision.insert(peer_id.to_string(), StorageProvision {
            capacity: 0,
            reliability: 1.0,
        });
        self.governance_participation.insert(peer_id.to_string(), GovernanceParticipation {
            proposals_submitted: 0,
            votes_cast: 0,
            discussions_participated: 0,
        });
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
    /// This function performs multi-node validation, including stake-weighted voting.
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

        // Perform multi-node validation
        let validators = self.select_validators()?;
        let mut valid_votes = 0;
        let mut total_votes = 0;

        for validator in validators {
            let vote = self.stake_weighted_vote(&validator, block)?;
            if vote {
                valid_votes += 1;
            }
            total_votes += 1;
        }

        let validation_threshold = (total_votes as f64 * 2.0 / 3.0).ceil() as usize;
        let is_valid = valid_votes >= validation_threshold;

        if is_valid {
            self.update_reputation(&block.proposer_id, true)?;
        } else {
            self.update_reputation(&block.proposer_id, false)?;
        }

        Ok(is_valid)
    }

/// Selects validators for block validation based on stake and reputation.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<String>>` - Returns a vector of selected validator peer IDs.
    fn select_validators(&self) -> IcnResult<Vec<String>> {
        let mut rng = rand::thread_rng();
        let mut validators: Vec<String> = self.known_peers.iter()
            .filter(|&peer_id| {
                let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
                stake > 0 && reputation > 0.5
            })
            .cloned()
            .collect();

        if validators.len() < 3 {
            return Err(IcnError::Consensus("Not enough eligible validators".to_string()));
        }

        validators.sort_by(|a, b| {
            let a_score = self.calculate_validator_score(a);
            let b_score = self.calculate_validator_score(b);
            b_score.partial_cmp(&a_score).unwrap()
        });

        Ok(validators.into_iter().take(10).collect())
    }

    /// Calculates a score for a validator based on stake and reputation.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to calculate the score for.
    ///
    /// # Returns
    ///
    /// * `f64` - The calculated validator score.
    fn calculate_validator_score(&self, peer_id: &str) -> f64 {
        let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0) as f64;
        let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
        stake * reputation
    }

    /// Performs a stake-weighted vote for block validation.
    ///
    /// # Arguments
    ///
    /// * `validator_id` - The ID of the validator performing the vote.
    /// * `block` - A reference to the block being validated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the validator approves the block, `Ok(false)` otherwise.
    fn stake_weighted_vote(&self, validator_id: &str, block: &Block) -> IcnResult<bool> {
        let stake = self.stake_info.get(validator_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for validator: {}", validator_id)))?
            .amount as f64;
        let reputation = self.reputation_scores.get(validator_id)
            .cloned()
            .unwrap_or(0.0);

        let voting_power = (stake * reputation).sqrt();
        let random_threshold = rand::thread_rng().gen::<f64>();

        Ok(voting_power > random_threshold)
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
            .map(|((_, coop_score), (_, rep_score))| coop_score * rep_score)
            .sum();
        let random_value: f64 = rng.gen::<f64>() * total_score;

        let mut cumulative_score = 0.0;
        for (peer_id, coop_score) in &self.cooperation_scores {
            let rep_score = self.reputation_scores.get(peer_id).unwrap_or(&0.0);
            cumulative_score += coop_score * rep_score;
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

        let recent_contributions: Vec<(u64, f64)> = history.iter()
            .rev()
            .take(100)
            .cloned()
            .collect();

        let mean: f64 = recent_contributions.iter().map(|&(_, score)| score).sum::<f64>() / recent_contributions.len() as f64;
        let variance: f64 = recent_contributions.iter().map(|&(_, score)| (score - mean).powi(2)).sum::<f64>() / recent_contributions.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

/// Updates the reputation score based on historical cooperation scores, consistency, and recent actions.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose reputation is to be updated.
    /// * `positive_action` - A boolean indicating whether the recent action was positive or negative.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the reputation is successfully updated, or an error message if it fails.
    pub fn update_reputation(&mut self, peer_id: &str, positive_action: bool) -> IcnResult<()> {
        let coop_score = *self.cooperation_scores
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        let consistency = self.calculate_consistency(peer_id)?;
        let network_impact = self.calculate_network_impact(peer_id)?;

        let rep_score = self.reputation_scores
            .entry(peer_id.to_string())
            .or_insert(1.0);

        let quality_factor = if positive_action { 1.1 } else { 0.9 };

        let new_rep_score = (
            CONSISTENCY_WEIGHT * consistency +
            QUALITY_WEIGHT * coop_score * quality_factor +
            NETWORK_IMPACT_WEIGHT * network_impact
        ) * REPUTATION_DECAY_FACTOR + (1.0 - REPUTATION_DECAY_FACTOR) * *rep_score;

        *rep_score = new_rep_score.max(0.0).min(1.0);

        Ok(())
    }

    /// Calculates the network impact of a peer based on their contributions to the network.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose network impact is to be calculated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - Returns the network impact score as a floating-point number, or an error message if it fails.
    fn calculate_network_impact(&self, peer_id: &str) -> IcnResult<f64> {
        let stake_info = self.stake_info.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for peer: {}", peer_id)))?;
        
        let comp_power = self.computational_power.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No computational power info for peer: {}", peer_id)))?;
        
        let storage = self.storage_provision.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No storage provision info for peer: {}", peer_id)))?;
        
        let governance = self.governance_participation.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No governance participation info for peer: {}", peer_id)))?;

        let stake_impact = (stake_info.amount as f64).log10() / 10.0;
        let comp_impact = (comp_power.cpu_power as f64 + comp_power.gpu_power as f64).log10() / 10.0;
        let storage_impact = (storage.capacity as f64).log10() / 20.0 * storage.reliability;
        let governance_impact = (governance.proposals_submitted + governance.votes_cast) as f64 / 100.0;

        let total_impact = stake_impact + comp_impact + storage_impact + governance_impact;
        Ok(total_impact.min(1.0))
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
        let score = self.cooperation_scores
            .get_mut(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        *score = (*score + new_score.max(0.0).min(1.0)) / 2.0;
        self.record_contribution(peer_id, new_score)?;
        
        Ok(())
    }

    /// Returns a list of eligible peers for the selection process.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector containing the IDs of eligible peers.
    pub fn get_eligible_peers(&self) -> Vec<String> {
        self.known_peers.iter()
            .filter(|&peer_id| {
                let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
                stake > 0 && reputation > 0.5
            })
            .cloned()
            .collect()
    }

    /// Handles a blockchain fork by selecting the most valid chain.
    ///
    /// This decision can be influenced by the quality of the blocks in each chain, such as cooperation and reputation scores.
    ///
    /// # Arguments
    ///
    /// * `chain_a` - A reference to the first blockchain.
    /// * `chain_b` - A reference to the second blockchain.
    ///
    /// # Returns
    ///
    /// * `&[Block]` - A reference to the selected blockchain.
    pub fn handle_fork<'a>(&self, chain_a: &'a [Block], chain_b: &'a [Block]) -> &'a [Block] {
        let score_a = self.calculate_chain_score(chain_a);
        let score_b = self.calculate_chain_score(chain_b);

        if score_a >= score_b {
            chain_a
        } else {
            chain_b
        }
    }

    /// Calculates a score for a given blockchain based on the reputation of block proposers.
    ///
    /// # Arguments
    ///
    /// * `chain` - A slice of Blocks representing the blockchain.
    ///
    /// # Returns
    ///
    /// * `f64` - The calculated score for the blockchain.
    fn calculate_chain_score(&self, chain: &[Block]) -> f64 {
        chain.iter()
            .map(|block| self.reputation_scores.get(&block.proposer_id).cloned().unwrap_or(0.0))
            .sum::<f64>() / chain.len() as f64
    }

/// Updates the stake information for a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose stake information is to be updated.
    /// * `amount` - The new stake amount.
    /// * `asset_type` - The type of asset being staked.
    /// * `duration` - The duration of the stake.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the stake information is successfully updated, or an error message if it fails.
    pub fn update_stake(&mut self, peer_id: &str, amount: u64, asset_type: String, duration: u64) -> IcnResult<()> {
        let stake_info = self.stake_info
            .entry(peer_id.to_string())
            .or_insert(StakeInfo {
                amount: 0,
                asset_type: "ICN".to_string(),
                duration: 0,
            });

        stake_info.amount = amount;
        stake_info.asset_type = asset_type;
        stake_info.duration = duration;

        info!("Updated stake for peer {}: amount={}, asset_type={}, duration={}", peer_id, amount, asset_type, duration);
        Ok(())
    }

    /// Updates the computational power information for a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose computational power information is to be updated.
    /// * `cpu_power` - The new CPU power value.
    /// * `gpu_power` - The new GPU power value.
    /// * `specialized_hardware` - A vector of specialized hardware capabilities.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the computational power information is successfully updated, or an error message if it fails.
    pub fn update_computational_power(&mut self, peer_id: &str, cpu_power: u64, gpu_power: u64, specialized_hardware: Vec<String>) -> IcnResult<()> {
        let comp_power = self.computational_power
            .entry(peer_id.to_string())
            .or_insert(ComputationalPower {
                cpu_power: 0,
                gpu_power: 0,
                specialized_hardware: Vec::new(),
            });

        comp_power.cpu_power = cpu_power;
        comp_power.gpu_power = gpu_power;
        comp_power.specialized_hardware = specialized_hardware.clone();

        info!("Updated computational power for peer {}: cpu_power={}, gpu_power={}, specialized_hardware={:?}", peer_id, cpu_power, gpu_power, specialized_hardware);
        Ok(())
    }

    /// Updates the storage provision information for a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose storage provision information is to be updated.
    /// * `capacity` - The new storage capacity in bytes.
    /// * `reliability` - The new reliability score (0.0 to 1.0).
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the storage provision information is successfully updated, or an error message if it fails.
    pub fn update_storage_provision(&mut self, peer_id: &str, capacity: u64, reliability: f64) -> IcnResult<()> {
        let storage_info = self.storage_provision
            .entry(peer_id.to_string())
            .or_insert(StorageProvision {
                capacity: 0,
                reliability: 1.0,
            });

        storage_info.capacity = capacity;
        storage_info.reliability = reliability.max(0.0).min(1.0);

        info!("Updated storage provision for peer {}: capacity={}, reliability={}", peer_id, capacity, reliability);
        Ok(())
    }

    /// Updates the governance participation information for a peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose governance participation information is to be updated.
    /// * `proposals_submitted` - The number of proposals submitted by the peer.
    /// * `votes_cast` - The number of votes cast by the peer.
    /// * `discussions_participated` - The number of discussions the peer has participated in.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the governance participation information is successfully updated, or an error message if it fails.
    pub fn update_governance_participation(&mut self, peer_id: &str, proposals_submitted: u64, votes_cast: u64, discussions_participated: u64) -> IcnResult<()> {
        let governance_info = self.governance_participation
            .entry(peer_id.to_string())
            .or_insert(GovernanceParticipation {
                proposals_submitted: 0,
                votes_cast: 0,
                discussions_participated: 0,
            });

        governance_info.proposals_submitted = proposals_submitted;
        governance_info.votes_cast = votes_cast;
        governance_info.discussions_participated = discussions_participated;

        info!("Updated governance participation for peer {}: proposals_submitted={}, votes_cast={}, discussions_participated={}", 
              peer_id, proposals_submitted, votes_cast, discussions_participated);
        Ok(())
    }

    /// Implements a basic Sybil resistance mechanism by checking if a peer meets certain criteria.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to check for Sybil resistance.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the peer passes the Sybil resistance check, `false` otherwise.
    pub fn sybil_resistant_check(&self, peer_id: &str) -> bool {
        let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
        let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
        let governance_participation = self.governance_participation.get(peer_id)
            .map(|info| info.proposals_submitted + info.votes_cast)
            .unwrap_or(0);

        stake >= 1000 && reputation >= 0.7 && governance_participation >= 10
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

        poc.update_stake("peer1", 1000, "ICN".to_string(), 30).unwrap();
        poc.update_stake("peer2", 2000, "ICN".to_string(), 60).unwrap();

        let proposer = poc.select_proposer().unwrap();
        assert!(vec!["peer1", "peer2"].contains(&proposer.as_str()));
    }

    #[test]
    fn test_update_reputation() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");

        poc.update_cooperation_score("peer1", 0.8).unwrap();
        poc.update_reputation("peer1", true).unwrap();

        let reputation = poc.reputation_scores.get("peer1").cloned().unwrap_or(0.0);
        assert!(reputation > 0.5, "Reputation should increase after positive action");

        poc.update_cooperation_score("peer1", 0.2).unwrap();
        poc.update_reputation("peer1", false).unwrap();

        let new_reputation = poc.reputation_scores.get("peer1").cloned().unwrap_or(0.0);
        assert!(new_reputation < reputation, "Reputation should decrease after negative action");
    }

    #[test]
    fn test_stake_weighted_vote() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        poc.register_peer("peer2");

        poc.update_stake("peer1", 1000, "ICN".to_string(), 30).unwrap();
        poc.update_stake("peer2", 5000, "ICN".to_string(), 60).unwrap();

        poc.update_reputation("peer1", true).unwrap();
        poc.update_reputation("peer2", true).unwrap();

        let block = Block::new(0, vec![], "previous_hash".to_string(), "proposer".to_string());

        let vote1 = poc.stake_weighted_vote("peer1", &block).unwrap();
        let vote2 = poc.stake_weighted_vote("peer2", &block).unwrap();

        // This test is probabilistic, so we run it multiple times
        let mut peer2_votes = 0;
        for _ in 0..100 {
            if poc.stake_weighted_vote("peer2", &block).unwrap() {
                peer2_votes += 1;
            }
        }

        assert!(peer2_votes > 50, "Peer2 with higher stake should have more voting power");
    }

    #[test]
    fn test_sybil_resistant_check() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("good_peer");
        poc.register_peer("bad_peer");

        // Set up a good peer
        poc.update_stake("good_peer", 2000, "ICN".to_string(), 60).unwrap();
        poc.update_reputation("good_peer", true).unwrap();
        poc.update_governance_participation("good_peer", 5, 10, 3).unwrap();

        // Set up a bad peer
        poc.update_stake("bad_peer", 500, "ICN".to_string(), 30).unwrap();
        poc.update_reputation("bad_peer", false).unwrap();
        poc.update_governance_participation("bad_peer", 1, 2, 1).unwrap();

        assert!(poc.sybil_resistant_check("good_peer"), "Good peer should pass Sybil resistance check");
        assert!(!poc.sybil_resistant_check("bad_peer"), "Bad peer should fail Sybil resistance check");
    }

    #[test]
    fn test_handle_fork() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        poc.register_peer("peer2");

        poc.update_reputation("peer1", true).unwrap();
        poc.update_reputation("peer2", false).unwrap();

        let block1 = Block::new(0, vec![], "previous_hash".to_string(), "peer1".to_string());
        let block2 = Block::new(1, vec![], block1.hash.clone(), "peer1".to_string());
        let chain_a = vec![block1.clone(), block2];

        let block3 = Block::new(1, vec![], block1.hash, "peer2".to_string());
        let chain_b = vec![block1, block3];

        let selected_chain = poc.handle_fork(&chain_a, &chain_b);
        assert_eq!(selected_chain.len(), 2, "Should select the longer chain with higher reputation");
    }

    #[test]
    fn test_calculate_network_impact() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");

        poc.update_stake("peer1", 10000, "ICN".to_string(), 90).unwrap();
        poc.update_computational_power("peer1", 1000, 2000, vec!["GPU".to_string()]).unwrap();
        poc.update_storage_provision("peer1", 1_000_000_000, 0.99).unwrap();
        poc.update_governance_participation("peer1", 10, 50, 20).unwrap();

        let impact = poc.calculate_network_impact("peer1").unwrap();
        assert!(impact > 0.5, "Peer with significant contributions should have high network impact");

        poc.register_peer("peer2");
        let low_impact = poc.calculate_network_impact("peer2").unwrap();
        assert!(low_impact < impact, "Peer with minimal contributions should have lower network impact");
    }
}