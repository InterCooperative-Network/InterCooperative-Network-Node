// File: icn_consensus/src/proof_of_cooperation.rs

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use rand::{Rng, thread_rng};
use log::info;
use serde::{Serialize, Deserialize};

use crate::consensus::{Consensus, NetworkEvent, NetworkCondition};

// Constants
const REPUTATION_DECAY_FACTOR: f64 = 0.95;
const CONSISTENCY_WEIGHT: f64 = 0.3;
const QUALITY_WEIGHT: f64 = 0.4;
const NETWORK_IMPACT_WEIGHT: f64 = 0.3;
const MAX_RECENT_CONTRIBUTIONS: usize = 100;
const MIN_STAKE_FOR_SYBIL_RESISTANCE: u64 = 1000;
const MIN_REPUTATION_FOR_SYBIL_RESISTANCE: f64 = 0.7;
const MIN_GOVERNANCE_PARTICIPATION_FOR_SYBIL_RESISTANCE: u64 = 10;
const BLOCK_TIME_THRESHOLD: u64 = 10; // Minimum time between blocks in seconds
const MAX_VALIDATORS: usize = 10; // Maximum number of validators for a block

/// The main struct implementing the Proof of Cooperation consensus mechanism
#[derive(Clone, Debug)]
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, f64>,
    reputation_scores: HashMap<String, f64>,
    contribution_history: HashMap<String, VecDeque<(u64, f64)>>,
    stake_info: HashMap<String, StakeInfo>,
    computational_power: HashMap<String, ComputationalPower>,
    storage_provision: HashMap<String, StorageProvision>,
    governance_participation: HashMap<String, GovernanceParticipation>,
    last_block_time: u64,
}

/// Struct representing a peer's stake information
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StakeInfo {
    amount: u64,
    asset_type: String,
    duration: u64,
}

/// Struct representing a peer's computational power contribution
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ComputationalPower {
    cpu_power: u64,
    gpu_power: u64,
    specialized_hardware: Vec<String>,
}

/// Struct representing a peer's storage provision
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StorageProvision {
    capacity: u64,
    reliability: f64,
}

/// Struct representing a peer's participation in governance
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GovernanceParticipation {
    proposals_submitted: u64,
    votes_cast: u64,
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
    /// This function initializes all necessary data structures for a new peer,
    /// setting default values for various metrics.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the new peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok(()) if the peer is successfully registered, or an error otherwise.
    pub fn register_peer(&mut self, peer_id: &str) -> IcnResult<()> {
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
        Ok(())
    }

    /// Selects validators for block validation
    ///
    /// This function chooses a set of validators based on their stake, reputation,
    /// and other factors to ensure a fair and secure validation process.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<String>>` - A result containing a vector of selected validator IDs or an error
    fn select_validators(&self) -> IcnResult<Vec<String>> {
        let validators: Vec<String> = self.known_peers.iter()
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

        let mut sorted_validators = validators;
        sorted_validators.sort_by(|a, b| {
            let a_score = self.calculate_validator_score(a);
            let b_score = self.calculate_validator_score(b);
            b_score.partial_cmp(&a_score).unwrap()
        });

        Ok(sorted_validators.into_iter().take(MAX_VALIDATORS).collect())
    }

    /// Calculates the score of a validator based on stake and reputation
    ///
    /// This function computes a score for a validator, which is used in the
    /// validator selection process.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    ///
    /// # Returns
    ///
    /// * `f64` - The calculated validator score
    fn calculate_validator_score(&self, peer_id: &str) -> f64 {
        let stake = self.stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0) as f64;
        let reputation = self.reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
        stake * reputation
    }

    /// Conducts a stake-weighted vote for block validation
    ///
    /// This function simulates a vote by a validator, taking into account their
    /// stake and reputation to determine the voting power.
    ///
    /// # Arguments
    ///
    /// * `validator_id` - A string slice that holds the identifier of the validator
    /// * `block` - A reference to the Block being voted on
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - A result containing the vote (true for valid, false for invalid) or an error
    fn stake_weighted_vote(&self, validator_id: &str, block: &Block) -> IcnResult<bool> {
        let stake = self.stake_info.get(validator_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for validator: {}", validator_id)))?
            .amount as f64;
        let reputation = self.reputation_scores.get(validator_id)
            .cloned()
            .unwrap_or(0.0);

        // Combine stake and reputation for voting power
        let voting_power = (stake * reputation).sqrt();
        let random_threshold = thread_rng().gen::<f64>();

        // Simplified voting logic
        Ok(voting_power * random_threshold > 0.5)
    }

    /// Records a contribution made by a peer
    ///
    /// This function logs a contribution to the peer's history, which is used
    /// in calculating their cooperation and reputation scores.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `score` - A f64 representing the score of the contribution
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
    /// This function evaluates how consistent a peer's contributions have been
    /// over time, which is a factor in their reputation score.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - A result containing the calculated consistency score or an error
    fn calculate_consistency(&self, peer_id: &str) -> IcnResult<f64> {
        let history = self.contribution_history
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        if history.is_empty() {
            return Ok(1.0);
        }

        let recent_contributions: Vec<f64> = history.iter().map(|&(_, score)| score).collect();
        let mean: f64 = recent_contributions.iter().sum::<f64>() / recent_contributions.len() as f64;
        let variance: f64 = recent_contributions.iter().map(|&score| (score - mean).powi(2)).sum::<f64>() / recent_contributions.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

    /// Updates the reputation of a peer based on their actions
    ///
    /// This function adjusts a peer's reputation score based on their recent actions
    /// and overall contribution to the network.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `positive_action` - A boolean indicating whether the action was positive or negative
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
    /// This function evaluates a peer's overall impact on the network based on
    /// their stake, computational power, storage provision, and governance participation.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - A result containing the calculated network impact score or an error
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
    /// This function adjusts a peer's cooperation score based on their recent actions
    /// and contributions to the network.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `new_score` - A f64 representing the new cooperation score
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
    /// This function returns a list of peers that meet the minimum requirements
    /// for stake, reputation, and governance participation to be considered for validation.
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

    /// Updates the stake information of a peer
    ///
    /// This function updates the stake amount, asset type, and duration for a given peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `amount` - A u64 representing the new stake amount
    /// * `asset_type` - A String representing the type of asset being staked
    /// * `duration` - A u64 representing the duration of the stake in seconds
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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

        info!("Updated stake for peer {}: amount={}, asset_type={}, duration={}", 
              peer_id, stake_info.amount, stake_info.asset_type, stake_info.duration);
        Ok(())
    }

    /// Updates the computational power information of a peer
    ///
    /// This function updates the CPU power, GPU power, and specialized hardware
    /// information for a given peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `cpu_power` - A u64 representing the CPU power
    /// * `gpu_power` - A u64 representing the GPU power
    /// * `specialized_hardware` - A Vec<String> representing specialized hardware capabilities
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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

        info!("Updated computational power for peer {}: cpu_power={}, gpu_power={}, specialized_hardware={:?}", 
              peer_id, cpu_power, gpu_power, specialized_hardware);
        Ok(())
    }

    /// Updates the storage provision information of a peer
    ///
    /// This function updates the storage capacity and reliability for a given peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `capacity` - A u64 representing the storage capacity in bytes
    /// * `reliability` - A f64 representing the reliability score (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
    /// This function updates the number of proposals submitted, votes cast,
    /// and discussions participated in for a given peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `proposals_submitted` - A u64 representing the number of proposals submitted
    /// * `votes_cast` - A u64 representing the number of votes cast
    /// * `discussions_participated` - A u64 representing the number of discussions participated in
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
              peer_id, governance_info.proposals_submitted, governance_info.votes_cast, governance_info.discussions_participated);
        Ok(())
    }

    /// Checks if a peer passes the Sybil resistance criteria
    ///
    /// This function evaluates whether a peer meets the minimum requirements
    /// for stake, reputation, and governance participation to be considered
    /// resistant to Sybil attacks.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    ///
    /// # Returns
    ///
    /// * `bool` - true if the peer passes the Sybil resistance check, false otherwise
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
    /// This function computes a reward for a peer based on their cooperation score,
    /// reputation score, and network impact.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    ///
    /// # Returns
    ///
    /// * `IcnResult<u64>` - A result containing the calculated reward or an error
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
    /// This function calculates and distributes rewards to all peers that meet
    /// the eligibility criteria.
    ///
    /// # Returns
    ///
    /// * `IcnResult<HashMap<String, u64>>` - A result containing a map of peer IDs to their rewards, or an error
    pub fn distribute_rewards(&mut self) -> IcnResult<HashMap<String, u64>> {
        let mut rewards = HashMap::new();

        for peer_id in self.get_eligible_peers() {
            let reward = self.calculate_reward(&peer_id)?;
            rewards.insert(peer_id, reward);
        }

        Ok(rewards)
    }

    /// Applies a penalty to a peer based on the severity of their misconduct
    ///
    /// This function reduces a peer's cooperation score, reputation score, and
    /// potentially their stake based on the severity of their misconduct.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - A string slice that holds the identifier of the peer
    /// * `severity` - A f64 representing the severity of the misconduct (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
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
    /// This function calculates a health score for the entire network based on
    /// average cooperation and reputation scores, as well as stake distribution.
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - A result containing the calculated health score or an error
    pub fn evaluate_network_health(&self) -> IcnResult<f64> {
        let avg_cooperation = self.cooperation_scores.values().sum::<f64>() / self.cooperation_scores.len() as f64;
        let avg_reputation = self.reputation_scores.values().sum::<f64>() / self.reputation_scores.len() as f64;

        let total_stake: u64 = self.stake_info.values().map(|info| info.amount).sum();
        let stake_distribution = 1.0 - (self.stake_info.values().map(|info| info.amount as f64 / total_stake as f64).map(|x| x * x).sum::<f64>().sqrt());

        let health_score = (avg_cooperation + avg_reputation + stake_distribution) / 3.0;

        Ok(health_score)
    }
}

/// Implement the `Consensus` trait for `ProofOfCooperation`
impl Consensus for ProofOfCooperation {
    /// Validates a block according to the Proof of Cooperation consensus rules
    ///
    /// This function checks if a block is valid by verifying the proposer,
    /// ensuring proper time between blocks, and conducting a stake-weighted vote.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the Block to be validated
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - A result containing true if the block is valid, false otherwise, or an error
    fn validate(&mut self, block: &Block) -> IcnResult<bool> {
        if !self.known_peers.contains(&block.proposer_id) {
            return Err(IcnError::Consensus(format!("Unknown proposer: {}", block.proposer_id)));
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?
            .as_secs();

        if current_time < self.last_block_time + BLOCK_TIME_THRESHOLD {
            return Err(IcnError::Consensus("Block proposed too soon".to_string()));
        }

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

        // Update reputation of the proposer based on the block's validity
        self.update_reputation(&block.proposer_id, is_valid)?;

        Ok(is_valid)
    }

    /// Selects a proposer for the next block
    ///
    /// This function chooses a peer to propose the next block based on their
    /// stake, reputation, and other factors to ensure a fair and secure selection process.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - A result containing the ID of the selected proposer or an error
    fn select_proposer(&mut self) -> IcnResult<String> {
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

    /// Gets the list of eligible peers for consensus participation
    ///
    /// This function returns a list of peer IDs that meet the minimum requirements
    /// for participating in the consensus process.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector of eligible peer IDs
    fn get_eligible_peers(&self) -> Vec<String> {
        self.get_eligible_peers() // This calls the public method we defined earlier
    }

    /// Updates the consensus state based on the current blockchain state
    ///
    /// This function is called after a new block is added to the chain to update
    /// the internal state of the consensus mechanism.
    ///
    /// # Arguments
    ///
    /// * `latest_block` - A reference to the latest block in the blockchain
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - A result indicating success or an error
    fn update_state(&mut self, latest_block: &Block) -> IcnResult<()> {
        self.last_block_time = latest_block.timestamp;
        Ok(())
    }

    /// Initializes the consensus mechanism with the current blockchain state
    ///
    /// This method is called when the node starts up or when it needs to resynchronize
    /// with the network. It allows the consensus mechanism to initialize its internal
    /// state based on the current blockchain.
    ///
    /// # Arguments
    ///
    /// * `latest_block` - A reference to the latest block in the blockchain
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if initialization is successful, or an `IcnError` if it fails.
    fn initialize(&mut self, latest_block: &Block) -> IcnResult<()> {
        self.last_block_time = latest_block.timestamp;
        // Additional initialization logic can be added here
        Ok(())
    }

    /// Handles network events that may affect the consensus state
    ///
    /// This method allows the consensus mechanism to react to various network events,
    /// such as peer connections/disconnections, network partitions, etc.
    ///
    /// # Arguments
    ///
    /// * `event` - A NetworkEvent representing different types of network events
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the event is handled successfully, or an `IcnError` if handling fails.
    fn handle_network_event(&mut self, event: NetworkEvent) -> IcnResult<()> {
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                self.register_peer(&peer_id)?;
                info!("New peer connected: {}", peer_id);
            },
            NetworkEvent::PeerDisconnected(peer_id) => {
                // Remove peer from all data structures
                self.known_peers.remove(&peer_id);
                self.cooperation_scores.remove(&peer_id);
                self.reputation_scores.remove(&peer_id);
                self.contribution_history.remove(&peer_id);
                self.stake_info.remove(&peer_id);
                self.computational_power.remove(&peer_id);
                self.storage_provision.remove(&peer_id);
                self.governance_participation.remove(&peer_id);
                info!("Peer disconnected: {}", peer_id);
            },
            NetworkEvent::NetworkPartitionDetected => {
                // Implement logic to handle network partition
                info!("Network partition detected");
            },
            NetworkEvent::NetworkReunified => {
                // Implement logic to handle network reunification
                info!("Network reunified");
            },
            NetworkEvent::NetworkConditionChanged(condition) => {
                match condition {
                    NetworkCondition::Normal => info!("Network condition: Normal"),
                    NetworkCondition::HighLatency => info!("Network condition: High Latency"),
                    NetworkCondition::Congested => info!("Network condition: Congested"),
                    NetworkCondition::Unstable => info!("Network condition: Unstable"),
                }
                // Adjust consensus parameters based on network condition
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_poc() -> ProofOfCooperation {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1").unwrap();
        poc.register_peer("peer2").unwrap();
        poc.register_peer("peer3").unwrap();
        poc
    }

    #[test]
    fn test_register_and_validate_peer() {
        let mut poc = ProofOfCooperation::new();
        poc.register_peer("peer1").unwrap();

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
    fn test_select_proposer() {
        let mut poc = setup_test_poc();

        // Set up peers with different stakes and reputations
        poc.update_stake("peer1", 2000, "ICN".to_string(), 60).unwrap();
        poc.update_reputation("peer1", true).unwrap();
        poc.update_cooperation_score("peer1", 0.9).unwrap();

        poc.update_stake("peer2", 1000, "ICN".to_string(), 30).unwrap();
        poc.update_reputation("peer2", true).unwrap();
        poc.update_cooperation_score("peer2", 0.7).unwrap();

        poc.update_stake("peer3", 3000, "ICN".to_string(), 90).unwrap();
        poc.update_reputation("peer3", true).unwrap();
        poc.update_cooperation_score("peer3", 0.8).unwrap();

        // Run multiple selections to check for randomness and weighted probability
        let mut selections = HashMap::new();
        for _ in 0..1000 {
            let proposer = poc.select_proposer().unwrap();
            *selections.entry(proposer).or_insert(0) += 1;
        }

        // Check that all peers were selected at least once
        assert!(selections.contains_key("peer1"));
        assert!(selections.contains_key("peer2"));
        assert!(selections.contains_key("peer3"));

        // Check that the peer with the highest stake and reputation (peer3) was selected most often
        assert!(selections.get("peer3").unwrap() > selections.get("peer1").unwrap());
        assert!(selections.get("peer3").unwrap() > selections.get("peer2").unwrap());
    }

    #[test]
    fn test_handle_network_event() {
        let mut poc = setup_test_poc();

        // Test PeerConnected event
        poc.handle_network_event(NetworkEvent::PeerConnected("peer4".to_string())).unwrap();
        assert!(poc.known_peers.contains("peer4"));

        // Test PeerDisconnected event
        poc.handle_network_event(NetworkEvent::PeerDisconnected("peer2".to_string())).unwrap();
        assert!(!poc.known_peers.contains("peer2"));

        // Test NetworkConditionChanged event
        poc.handle_network_event(NetworkEvent::NetworkConditionChanged(NetworkCondition::HighLatency)).unwrap();
        // Add assertions here to check if the consensus mechanism reacted appropriately to the network condition change

        // Test NetworkPartitionDetected and NetworkReunified events
        poc.handle_network_event(NetworkEvent::NetworkPartitionDetected).unwrap();
        poc.handle_network_event(NetworkEvent::NetworkReunified).unwrap();
        // Add assertions here to check if the consensus mechanism handled these events correctly
    }
}