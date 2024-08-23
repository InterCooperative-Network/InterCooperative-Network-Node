// File: icn_consensus/src/proof_of_cooperation.rs

//! This module implements the Proof of Cooperation (PoC) consensus mechanism for the InterCooperative Network (ICN).
//! 
//! The PoC consensus mechanism is designed to incentivize cooperation and positive contributions to the network.
//! It uses a combination of factors including stake, reputation, and various forms of network participation to
//! determine a node's influence and rewards within the network.
//!
//! Key components of this module include:
//! - Peer registration and management
//! - Calculation and updating of cooperation and reputation scores
//! - Block validation using a stake-weighted voting system
//! - Proposer selection based on reputation and stake
//! - Management of various forms of network contribution (computational power, storage, governance participation)

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use rand::{Rng, thread_rng};
use log::{info, error};

/// Constants used in the Proof of Cooperation consensus mechanism
const REPUTATION_DECAY_FACTOR: f64 = 0.95;
const CONSISTENCY_WEIGHT: f64 = 0.3;
const QUALITY_WEIGHT: f64 = 0.4;
const NETWORK_IMPACT_WEIGHT: f64 = 0.3;
const MAX_RECENT_CONTRIBUTIONS: usize = 100;
const MIN_STAKE_FOR_SYBIL_RESISTANCE: u64 = 1000;
const MIN_REPUTATION_FOR_SYBIL_RESISTANCE: f64 = 0.7;
const MIN_GOVERNANCE_PARTICIPATION_FOR_SYBIL_RESISTANCE: u64 = 10;

/// The main struct implementing the Proof of Cooperation consensus mechanism
#[derive(Clone, Debug)]
pub struct ProofOfCooperation {
    /// Set of known peer IDs in the network
    known_peers: HashSet<String>,
    /// Mapping of peer IDs to their current cooperation scores
    cooperation_scores: HashMap<String, f64>,
    /// Mapping of peer IDs to their current reputation scores
    reputation_scores: HashMap<String, f64>,
    /// History of recent contributions for each peer
    contribution_history: HashMap<String, VecDeque<(u64, f64)>>,
    /// Information about each peer's stake in the network
    stake_info: HashMap<String, StakeInfo>,
    /// Information about each peer's computational power contribution
    computational_power: HashMap<String, ComputationalPower>,
    /// Information about each peer's storage provision
    storage_provision: HashMap<String, StorageProvision>,
    /// Information about each peer's participation in governance
    governance_participation: HashMap<String, GovernanceParticipation>,
    /// Timestamp of the last block added to the chain
    last_block_time: u64,
}

/// Struct representing a peer's stake information
#[derive(Clone, Debug)]
struct StakeInfo {
    /// Amount of stake
    amount: u64,
    /// Type of asset staked
    asset_type: String,
    /// Duration of the stake
    duration: u64,
}

/// Struct representing a peer's computational power contribution
#[derive(Clone, Debug)]
struct ComputationalPower {
    /// Measure of CPU power contributed
    cpu_power: u64,
    /// Measure of GPU power contributed
    gpu_power: u64,
    /// List of specialized hardware contributed
    specialized_hardware: Vec<String>,
}

/// Struct representing a peer's storage provision
#[derive(Clone, Debug)]
struct StorageProvision {
    /// Amount of storage capacity provided
    capacity: u64,
    /// Measure of the storage reliability
    reliability: f64,
}

/// Struct representing a peer's participation in governance
#[derive(Clone, Debug)]
struct GovernanceParticipation {
    /// Number of proposals submitted by the peer
    proposals_submitted: u64,
    /// Number of votes cast by the peer
    votes_cast: u64,
    /// Number of discussions the peer has participated in
    discussions_participated: u64,
}

impl ProofOfCooperation {
    /// Creates a new instance of the ProofOfCooperation consensus mechanism
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

    /// Registers a new peer in the network
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to register
    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 1.0);
        self.reputation_scores.insert(peer_id.to_string(), 1.0);
        self.contribution_history.insert(peer_id.to_string(), VecDeque::new());
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

    /// Validates a block according to the Proof of Cooperation consensus rules
    ///
    /// # Arguments
    ///
    /// * `block` - The block to validate
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Ok(true) if the block is valid, Ok(false) if invalid, or an IcnError if there was a problem during validation
    pub fn validate(&mut self, block: &Block) -> IcnResult<bool> {
        if !self.known_peers.contains(&block.proposer_id) {
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

        let validators = self.select_validators()?;
        let mut valid_votes = 0;
        let total_votes = validators.len();

        for validator in validators {
            let vote = self.stake_weighted_vote(&validator, block)?;
            if vote {
                valid_votes += 1;
            }
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

    /// Selects validators for block validation
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<String>>` - A vector of peer IDs selected as validators, or an IcnError if there was a problem
    fn select_validators(&self) -> IcnResult<Vec<String>> {
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

    /// Calculates the score of a validator based on stake and reputation
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose score is to be calculated
    ///
    /// # Returns
    ///
    /// * `f64` - The calculated score
    fn calculate_validator_score(&self, peer_id: &str) -> f64 {
        let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0) as f64;
        let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
        stake * reputation
    }

    /// Conducts a stake-weighted vote for block validation
    ///
    /// # Arguments
    ///
    /// * `validator_id` - The ID of the validator
    /// * `block` - The block being validated
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - The result of the vote (true for valid, false for invalid), or an IcnError if there was a problem
    fn stake_weighted_vote(&self, validator_id: &str, _block: &Block) -> IcnResult<bool> {
        let stake = self.stake_info.get(validator_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for validator: {}", validator_id)))?
            .amount as f64;
        let reputation = self.reputation_scores.get(validator_id)
            .cloned()
            .unwrap_or(0.0);

        let voting_power = (stake * reputation).sqrt();
        let random_threshold = thread_rng().gen::<f64>();

        Ok(voting_power > random_threshold)
    }

    /// Selects a proposer for the next block
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - The ID of the selected proposer, or an IcnError if there was a problem
    pub fn select_proposer(&self) -> IcnResult<String> {
        let eligible_peers = self.get_eligible_peers();
        if eligible_peers.is_empty() {
            return Err(IcnError::Consensus("No eligible proposers available".to_string()));
        }

        let total_score: f64 = eligible_peers.iter()
            .map(|peer_id| self.calculate_validator_score(peer_id))
            .sum();

        let mut rng = thread_rng();
        let mut cumulative_weight = 0.0;
        let random_value = rng.gen::<f64>() * total_score;

        for peer_id in &eligible_peers {
            cumulative_weight += self.calculate_validator_score(peer_id);
            if cumulative_weight >= random_value {
                return Ok(peer_id.clone());
            }
        }

        Err(IcnError::Consensus("Failed to select proposer".to_string()))
    }

    /// Records a contribution made by a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer
    /// * `score` - The score of the contribution
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
    fn record_contribution(&mut self, peer_id: &str, score: f64) -> IcnResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();
        
        let history = self.contribution_history
            .entry(peer_id.to_string())
            .or_insert_with(VecDeque::new);
        
        history.push_back((timestamp, score));

        while history.len() > MAX_RECENT_CONTRIBUTIONS {
            history.pop_front();
        }

        Ok(())
    }

    /// Calculates the consistency of a peer's contributions
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - The calculated consistency score, or an IcnError if there was a problem
    fn calculate_consistency(&self, peer_id: &str) -> IcnResult<f64> {
        let history = self.contribution_history
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        if history.is_empty() {
            return Ok(1.0);
        }

        let recent_contributions: Vec<(u64, f64)> = history.iter().cloned().collect();

        let mean: f64 = recent_contributions.iter().map(|&(_, score)| score).sum::<f64>() / recent_contributions.len() as f64;
        let variance: f64 = recent_contributions.iter().map(|&(_, score)| (score - mean).powi(2)).sum::<f64>() / recent_contributions.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

    /// Updates the reputation of a peer based on their actions
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer
    /// * `positive_action` - A boolean indicating whether the action was positive or negative
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
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
    
        /// Calculates the network impact of a peer
        ///
        /// # Arguments
        ///
        /// * `peer_id` - The ID of the peer
        ///
        /// # Returns
        ///
        /// * `IcnResult<f64>` - The calculated network impact score, or an IcnError if there was a problem
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
    
        /// Updates the cooperation score of a peer
        ///
        /// # Arguments
        ///
        /// * `peer_id` - The ID of the peer
        /// * `new_score` - The new cooperation score
        ///
        /// # Returns
        ///
        /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
        pub fn update_cooperation_score(&mut self, peer_id: &str, new_score: f64) -> IcnResult<()> {
            let score = self.cooperation_scores
                .entry(peer_id.to_string())
                .or_insert(1.0);
            
            *score = (*score + new_score.max(0.0).min(1.0)) / 2.0;
            self.record_contribution(peer_id, new_score)?;
            
            Ok(())
        }
    
        /// Retrieves a list of eligible peers for validation
        ///
        /// # Returns
        ///
        /// * `Vec<String>` - A vector of peer IDs that are eligible for validation
        pub fn get_eligible_peers(&self) -> Vec<String> {
            self.known_peers.iter()
                .filter(|&peer_id| {
                    let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                    let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
                    stake >= MIN_STAKE_FOR_SYBIL_RESISTANCE && reputation >= MIN_REPUTATION_FOR_SYBIL_RESISTANCE
                })
                .cloned()
                .collect()
        }
    
        /// Handles a fork in the blockchain by selecting the chain with the higher reputation
        ///
        /// # Arguments
        ///
        /// * `chain_a` - The first chain to compare
        /// * `chain_b` - The second chain to compare
        ///
        /// # Returns
        ///
        /// * `&[Block]` - A reference to the selected chain
        pub fn handle_fork<'a>(&self, chain_a: &'a [Block], chain_b: &'a [Block]) -> &'a [Block] {
            let score_a = self.calculate_chain_score(chain_a);
            let score_b = self.calculate_chain_score(chain_b);
    
            if score_a >= score_b {
                chain_a
            } else {
                chain_b
            }
        }
    
        /// Calculates the score of a chain based on the reputation of the proposers
        ///
        /// # Arguments
        ///
        /// * `chain` - The chain whose score is to be calculated
        ///
        /// # Returns
        ///
        /// * `f64` - The calculated score of the chain
        fn calculate_chain_score(&self, chain: &[Block]) -> f64 {
            chain.iter()
                .map(|block| self.reputation_scores.get(&block.proposer_id).cloned().unwrap_or(0.0))
                .sum::<f64>() / chain.len() as f64
        }
    
        /// Updates the stake information of a peer
        ///
        /// # Arguments
        ///
        /// * `peer_id` - The ID of the peer
        /// * `amount` - The new stake amount
        /// * `asset_type` - The type of asset staked
        /// * `duration` - The duration of the stake
        ///
        /// # Returns
        ///
        /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
        pub fn update_stake(&mut self, peer_id: &str, amount: u64, asset_type: String, duration: u64) -> IcnResult<()> {
            let stake_info = self.stake_info
                .entry(peer_id.to_string())
                .or_insert(StakeInfo {
                    amount: 0,
                    asset_type: asset_type.clone(),
                    duration: 0,
                });
    
            stake_info.amount = amount;
            stake_info.asset_type = asset_type;
            stake_info.duration = duration;
    
            info!("Updated stake for peer {}: amount={}, asset_type={}, duration={}", peer_id, amount, stake_info.asset_type, duration);
            Ok(())
        }
    
        /// Updates the computational power information of a peer
        ///
        /// # Arguments
        ///
        /// * `peer_id` - The ID of the peer
        /// * `cpu_power` - The CPU power of the peer
        /// * `gpu_power` - The GPU power of the peer
        /// * `specialized_hardware` - A vector of specialized hardware used by the peer
        ///
        /// # Returns
        ///
        /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
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

    /// Updates the storage provision information of a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer
    /// * `capacity` - The storage capacity provided by the peer
    /// * `reliability` - The reliability of the storage
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
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

    /// Updates the governance participation information of a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer
    /// * `proposals_submitted` - The number of proposals submitted by the peer
    /// * `votes_cast` - The number of votes cast by the peer
    /// * `discussions_participated` - The number of discussions participated in by the peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
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

    /// Checks if a peer passes the Sybil resistance criteria
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to check
    ///
    /// # Returns
    ///
    /// * `bool` - true if the peer passes the Sybil resistance criteria, false otherwise
    pub fn sybil_resistant_check(&self, peer_id: &str) -> bool {
        let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
        let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
        let governance_participation = self.governance_participation.get(peer_id)
            .map(|info| info.proposals_submitted + info.votes_cast)
            .unwrap_or(0);

        stake >= MIN_STAKE_FOR_SYBIL_RESISTANCE &&
        reputation >= MIN_REPUTATION_FOR_SYBIL_RESISTANCE &&
        governance_participation >= MIN_GOVERNANCE_PARTICIPATION_FOR_SYBIL_RESISTANCE
    }

    /// Calculates the reward for a peer based on their contributions and reputation
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose reward is to be calculated
    ///
    /// # Returns
    ///
    /// * `IcnResult<u64>` - The calculated reward amount, or an IcnError if there was a problem
    pub fn calculate_reward(&self, peer_id: &str) -> IcnResult<u64> {
        let cooperation_score = self.cooperation_scores.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        let reputation_score = self.reputation_scores.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        let network_impact = self.calculate_network_impact(peer_id)?;

        let base_reward = 100;

        let adjusted_reward = (base_reward as f64 * cooperation_score * reputation_score * (1.0 + network_impact)).round() as u64;

        Ok(adjusted_reward)
    }

    /// Distributes rewards to all eligible peers
    ///
    /// # Returns
    ///
    /// * `IcnResult<HashMap<String, u64>>` - A mapping of peer IDs to their rewards, or an IcnError if there was a problem
    pub fn distribute_rewards(&self) -> IcnResult<HashMap<String, u64>> {
        let mut rewards = HashMap::new();

        for peer_id in self.get_eligible_peers() {
            let reward = self.calculate_reward(&peer_id)?;
            rewards.insert(peer_id, reward);
        }

        Ok(rewards)
    }

    /// Applies a penalty to a peer based on the severity of their misconduct
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to penalize
    /// * `severity` - The severity of the penalty, ranging from 0.0 to 1.0
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if successful, or an IcnError if there was a problem
    pub fn apply_penalty(&mut self, peer_id: &str, severity: f64) -> IcnResult<()> {
        let severity = severity.max(0.0).min(1.0);

        if let Some(score) = self.cooperation_scores.get_mut(peer_id) {
            *score *= 1.0 - severity;
        }

        if let Some(score) = self.reputation_scores.get_mut(peer_id) {
            *score *= 1.0 - severity;
        }

        if severity > 0.5 {
            if let Some(stake_info) = self.stake_info.get_mut(peer_id) {
                stake_info.amount = (stake_info.amount as f64 * (1.0 - severity * 0.5)) as u64;
            }
        }

        info!("Applied penalty to peer {} with severity {}", peer_id, severity);
        Ok(())
    }

/// Evaluates the overall health of the network
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - A score representing the health of the network, or an IcnError if there was a problem
    pub fn evaluate_network_health(&self) -> IcnResult<f64> {
        let avg_cooperation = self.cooperation_scores.values().sum::<f64>() / self.cooperation_scores.len() as f64;
        let avg_reputation = self.reputation_scores.values().sum::<f64>() / self.reputation_scores.len() as f64;
        
        let total_stake: u64 = self.stake_info.values().map(|info| info.amount).sum();
        let stake_distribution = 1.0 - (self.stake_info.values().map(|info| info.amount as f64 / total_stake as f64).map(|x| x * x).sum::<f64>().sqrt());

        let health_score = (avg_cooperation + avg_reputation + stake_distribution) / 3.0;

        Ok(health_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_poc() -> ProofOfCooperation {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");
        poc.register_peer("peer2");
        poc.register_peer("peer3");
        poc
    }

    #[test]
    fn test_register_and_validate_peer() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1");

        assert!(poc.known_peers.contains("peer1"));
        assert!(!poc.known_peers.contains("unknown_peer"));

        let block = Block::new(0, vec![], "previous_hash".to_string(), "peer1".to_string());
        assert!(poc.validate(&block).is_ok());

        let invalid_block = Block::new(0, vec![], "previous_hash".to_string(), "unknown_peer".to_string());
        assert!(poc.validate(&invalid_block).is_err());
    }

    #[test]
    fn test_update_and_calculate_reputation() {
        let mut poc = setup_test_poc();

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
    fn test_sybil_resistant_check() {
        let mut poc = setup_test_poc();

        poc.update_stake("peer1", 2000, "ICN".to_string(), 60).unwrap();
        poc.update_reputation("peer1", true).unwrap();
        poc.update_governance_participation("peer1", 5, 10, 3).unwrap();

        poc.update_stake("peer2", 500, "ICN".to_string(), 30).unwrap();
        poc.update_reputation("peer2", false).unwrap();
        poc.update_governance_participation("peer2", 1, 2, 1).unwrap();

        assert!(poc.sybil_resistant_check("peer1"), "Good peer should pass Sybil resistance check");
        assert!(!poc.sybil_resistant_check("peer2"), "Bad peer should fail Sybil resistance check");
    }

    #[test]
    fn test_calculate_and_distribute_rewards() {
        let mut poc = setup_test_poc();

        poc.update_stake("peer1", 1000, "ICN".to_string(), 30).unwrap();
        poc.update_cooperation_score("peer1", 0.9).unwrap();
        poc.update_reputation("peer1", true).unwrap();

        poc.update_stake("peer2", 500, "ICN".to_string(), 30).unwrap();
        poc.update_cooperation_score("peer2", 0.5).unwrap();
        poc.update_reputation("peer2", true).unwrap();

        poc.update_stake("peer3", 2000, "ICN".to_string(), 60).unwrap();
        poc.update_cooperation_score("peer3", 0.7).unwrap();
        poc.update_reputation("peer3", true).unwrap();

        let reward1 = poc.calculate_reward("peer1").unwrap();
        let reward2 = poc.calculate_reward("peer2").unwrap();
        let reward3 = poc.calculate_reward("peer3").unwrap();

        assert!(reward1 > reward2, "Peer1 should have higher reward than Peer2");
        assert!(reward3 > reward1, "Peer3 should have higher reward than Peer1");

        let rewards = poc.distribute_rewards().unwrap();
        assert_eq!(rewards.len(), 3, "All three peers should receive rewards");
        assert_eq!(rewards.get("peer1"), Some(&reward1));
        assert_eq!(rewards.get("peer2"), Some(&reward2));
        assert_eq!(rewards.get("peer3"), Some(&reward3));
    }

    #[test]
    fn test_apply_penalty() {
        let mut poc = setup_test_poc();

        poc.update_cooperation_score("peer1", 0.8).unwrap();
        poc.update_reputation("peer1", true).unwrap();
        poc.update_stake("peer1", 1000, "ICN".to_string(), 30).unwrap();

        let initial_cooperation = poc.cooperation_scores.get("peer1").cloned().unwrap();
        let initial_reputation = poc.reputation_scores.get("peer1").cloned().unwrap();
        let initial_stake = poc.stake_info.get("peer1").unwrap().amount;

        poc.apply_penalty("peer1", 0.3).unwrap();

        let after_penalty_cooperation = poc.cooperation_scores.get("peer1").cloned().unwrap();
        let after_penalty_reputation = poc.reputation_scores.get("peer1").cloned().unwrap();
        let after_penalty_stake = poc.stake_info.get("peer1").unwrap().amount;

        assert!(after_penalty_cooperation < initial_cooperation, "Cooperation score should decrease after penalty");
        assert!(after_penalty_reputation < initial_reputation, "Reputation score should decrease after penalty");
        assert_eq!(after_penalty_stake, initial_stake, "Stake should not change for moderate penalty");

        poc.apply_penalty("peer1", 0.8).unwrap();

        let final_cooperation = poc.cooperation_scores.get("peer1").cloned().unwrap();
        let final_reputation = poc.reputation_scores.get("peer1").cloned().unwrap();
        let final_stake = poc.stake_info.get("peer1").unwrap().amount;

        assert!(final_cooperation < after_penalty_cooperation, "Cooperation score should decrease further after severe penalty");
        assert!(final_reputation < after_penalty_reputation, "Reputation score should decrease further after severe penalty");
        assert!(final_stake < after_penalty_stake, "Stake should decrease for severe penalty");
    }

    #[test]
    fn test_network_health_evaluation() {
        let mut poc = setup_test_poc();

        for (i, peer) in ["peer1", "peer2", "peer3"].iter().enumerate() {
            poc.update_cooperation_score(peer, 0.8 + i as f64 * 0.1).unwrap();
            poc.update_reputation(peer, true).unwrap();
            poc.update_stake(peer, 1000 + i as u64 * 500, "ICN".to_string(), 30).unwrap();
        }

        let health_score = poc.evaluate_network_health().unwrap();
        assert!(health_score > 0.8, "Network health should be high for a well-balanced network");

        poc.update_cooperation_score("peer1", 0.3).unwrap();
        poc.update_reputation("peer1", false).unwrap();
        poc.update_stake("peer2", 10000, "ICN".to_string(), 30).unwrap();

        let new_health_score = poc.evaluate_network_health().unwrap();
        assert!(new_health_score < health_score, "Network health should decrease for an imbalanced network");
    }

    #[test]
    fn test_fork_handling() {
        let mut poc = setup_test_poc();

        poc.update_reputation("peer1", true).unwrap();
        poc.update_reputation("peer2", false).unwrap();

        let block1 = Block::new(0, vec![], "genesis".to_string(), "peer1".to_string());
        let block2a = Block::new(1, vec![], block1.hash.clone(), "peer1".to_string());
        let block3a = Block::new(2, vec![], block2a.hash.clone(), "peer1".to_string());
        let chain_a = vec![block1.clone(), block2a, block3a];

        let block2b = Block::new(1, vec![], block1.hash.clone(), "peer2".to_string());
        let block3b = Block::new(2, vec![], block2b.hash.clone(), "peer2".to_string());
        let chain_b = vec![block1, block2b, block3b];

        let selected_chain = poc.handle_fork(&chain_a, &chain_b);
        assert_eq!(selected_chain.len(), 3, "Selected chain should have 3 blocks");
        assert_eq!(selected_chain[2].proposer_id, "peer1", "Chain proposed by peer1 should be selected due to higher reputation");
    }
}