// Filename: icn_consensus/src/proof_of_cooperation.rs

//! Proof of Cooperation (PoC) Consensus Mechanism Implementation
//!
//! This module implements the Proof of Cooperation consensus mechanism for the InterCooperative Network (ICN).
//! PoC is designed to prioritize active participation, quality contributions, and equitable power distribution
//! across the network. It combines elements of reputation, stake, and cooperation to create a fair and
//! efficient consensus process.
//!
//! Key Features:
//! - Dynamic cooperation and reputation scoring
//! - Stake-weighted validation and voting
//! - Sybil resistance through multi-factor identity verification
//! - Byzantine fault tolerance
//! - Incentive alignment with network goals
//! - Transparent and auditable decision-making processes
//! - Smart contract integration for automated governance and reward distribution
//!
//! The `ProofOfCooperation` struct is the central component, managing peer information, scoring, and consensus operations.
//! It implements the `Consensus` trait, providing core functionality for block validation and proposer selection.

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use icn_smart_contracts::{SmartContract, SmartContractEngine};
use rand::{Rng, thread_rng};
use log::{info, warn, error};

use crate::consensus::Consensus;

/// Constants used in the Proof of Cooperation consensus mechanism
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
    smart_contract_engine: SmartContractEngine,
}

/// Struct representing a peer's stake information
#[derive(Clone, Debug)]
struct StakeInfo {
    amount: u64,
    asset_type: String,
    duration: u64,
}

/// Struct representing a peer's computational power contribution
#[derive(Clone, Debug)]
struct ComputationalPower {
    cpu_power: u64,
    gpu_power: u64,
    specialized_hardware: Vec<String>,
}

/// Struct representing a peer's storage provision
#[derive(Clone, Debug)]
struct StorageProvision {
    capacity: u64,
    reliability: f64,
}

/// Struct representing a peer's participation in governance
#[derive(Clone, Debug)]
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
            smart_contract_engine: SmartContractEngine::new(),
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

        // Execute smart contract for additional voting logic
        let voting_contract = self.smart_contract_engine.get_contract("voting_logic")
            .ok_or_else(|| IcnError::Consensus("Voting logic smart contract not found".to_string()))?;
        let args = vec![
            voting_power.to_string(),
            random_threshold.to_string(),
            serde_json::to_string(block).map_err(|e| IcnError::Consensus(format!("Failed to serialize block: {}", e)))?
        ];
        let result = self.smart_contract_engine.call_contract(voting_contract.id, "validate_vote", args)?;
        
        Ok(result == "true")
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
    fn update_reputation(&mut self, peer_id: &str, positive_action: bool) -> IcnResult<()> {
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

        // Execute smart contract for additional reputation logic
        let reputation_contract = self.smart_contract_engine.get_contract("reputation_adjustment")
            .ok_or_else(|| IcnError::Consensus("Reputation adjustment smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            rep_score.to_string(),
            positive_action.to_string()
        ];
        let adjusted_score = self.smart_contract_engine.call_contract(reputation_contract.id, "adjust_reputation", args)?;
        *rep_score = adjusted_score.parse::<f64>().map_err(|e| IcnError::Consensus(format!("Failed to parse adjusted reputation score: {}", e)))?;

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

        // Execute smart contract for additional network impact calculation
        let impact_contract = self.smart_contract_engine.get_contract("network_impact")
            .ok_or_else(|| IcnError::Consensus("Network impact smart contract not found".to_string()))?;
        let args = vec![
            total_impact.to_string(),
            serde_json::to_string(stake_info).map_err(|e| IcnError::Consensus(format!("Failed to serialize stake info: {}", e)))?,
            serde_json::to_string(comp_power).map_err(|e| IcnError::Consensus(format!("Failed to serialize computational power: {}", e)))?,
            serde_json::to_string(storage).map_err(|e| IcnError::Consensus(format!("Failed to serialize storage provision: {}", e)))?,
            serde_json::to_string(governance).map_err(|e| IcnError::Consensus(format!("Failed to serialize governance participation: {}", e)))?
        ];
        let adjusted_impact = self.smart_contract_engine.call_contract(impact_contract.id, "calculate_impact", args)?;
        let final_impact = adjusted_impact.parse::<f64>().map_err(|e| IcnError::Consensus(format!("Failed to parse adjusted network impact: {}", e)))?;

        Ok(final_impact.min(1.0))
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

        // Execute smart contract for additional cooperation score adjustment
        let cooperation_contract = self.smart_contract_engine.get_contract("cooperation_adjustment")
            .ok_or_else(|| IcnError::Consensus("Cooperation adjustment smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            score.to_string(),
            new_score.to_string()
        ];
        let adjusted_score = self.smart_contract_engine.call_contract(cooperation_contract.id, "adjust_cooperation", args)?;
        *score = adjusted_score.parse::<f64>().map_err(|e| IcnError::Consensus(format!("Failed to parse adjusted cooperation score: {}", e)))?;

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

        // Execute smart contract for stake validation and potential bonuses
        let stake_contract = self.smart_contract_engine.get_contract("stake_management")
            .ok_or_else(|| IcnError::Consensus("Stake management smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            amount.to_string(),
            stake_info.asset_type.clone(),
            duration.to_string()
        ];
        let result = self.smart_contract_engine.call_contract(stake_contract.id, "validate_and_adjust_stake", args)?;
        let adjusted_stake: StakeInfo = serde_json::from_str(&result)
            .map_err(|e| IcnError::Consensus(format!("Failed to parse adjusted stake info: {}", e)))?;

        *stake_info = adjusted_stake;

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

        // Execute smart contract for governance participation validation and potential bonuses
        let governance_contract = self.smart_contract_engine.get_contract("governance_participation")
            .ok_or_else(|| IcnError::Consensus("Governance participation smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            proposals_submitted.to_string(),
            votes_cast.to_string(),
            discussions_participated.to_string()
        ];
        let result = self.smart_contract_engine.call_contract(governance_contract.id, "validate_and_adjust_participation", args)?;
        let adjusted_participation: GovernanceParticipation = serde_json::from_str(&result)
            .map_err(|e| IcnError::Consensus(format!("Failed to parse adjusted governance participation: {}", e)))?;

        *governance_info = adjusted_participation;

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

        // Execute smart contract for reward calculation
        let reward_contract = self.smart_contract_engine.get_contract("reward_calculation")
            .ok_or_else(|| IcnError::Consensus("Reward calculation smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            cooperation_score.to_string(),
            reputation_score.to_string(),
            network_impact.to_string(),
            adjusted_reward.to_string()
        ];
        let final_reward = self.smart_contract_engine.call_contract(reward_contract.id, "calculate_final_reward", args)?;

        Ok(final_reward.parse::<u64>().map_err(|e| IcnError::Consensus(format!("Failed to parse final reward: {}", e)))?)
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

        // Execute smart contract for reward distribution
        let distribution_contract = self.smart_contract_engine.get_contract("reward_distribution")
            .ok_or_else(|| IcnError::Consensus("Reward distribution smart contract not found".to_string()))?;
        let args = vec![serde_json::to_string(&rewards).map_err(|e| IcnError::Consensus(format!("Failed to serialize rewards: {}", e)))?];
        let distributed_rewards = self.smart_contract_engine.call_contract(distribution_contract.id, "distribute_rewards", args)?;
        
        let final_rewards: HashMap<String, u64> = serde_json::from_str(&distributed_rewards)
            .map_err(|e| IcnError::Consensus(format!("Failed to parse distributed rewards: {}", e)))?;

        Ok(final_rewards)
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

        // Execute smart contract for penalty application
        let penalty_contract = self.smart_contract_engine.get_contract("penalty_application")
            .ok_or_else(|| IcnError::Consensus("Penalty application smart contract not found".to_string()))?;
        let args = vec![
            peer_id.to_string(),
            severity.to_string(),
            self.cooperation_scores.get(peer_id).unwrap_or(&0.0).to_string(),
            self.reputation_scores.get(peer_id).unwrap_or(&0.0).to_string(),
            self.stake_info.get(peer_id).map(|s| s.amount).unwrap_or(0).to_string()
        ];
        let penalty_result = self.smart_contract_engine.call_contract(penalty_contract.id, "apply_penalty", args)?;
        
        let penalty_data: HashMap<String, f64> = serde_json::from_str(&penalty_result)
            .map_err(|e| IcnError::Consensus(format!("Failed to parse penalty result: {}", e)))?;

        if let Some(coop_score) = penalty_data.get("cooperation_score") {
            self.cooperation_scores.insert(peer_id.to_string(), *coop_score);
        }
        if let Some(rep_score) = penalty_data.get("reputation_score") {
            self.reputation_scores.insert(peer_id.to_string(), *rep_score);
        }
        if let Some(stake_amount) = penalty_data.get("stake_amount") {
            if let Some(stake_info) = self.stake_info.get_mut(peer_id) {
                stake_info.amount = *stake_amount as u64;
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

        // Execute smart contract for network health evaluation
        let health_contract = self.smart_contract_engine.get_contract("network_health")
            .ok_or_else(|| IcnError::Consensus("Network health smart contract not found".to_string()))?;
        let args = vec![
            avg_cooperation.to_string(),
            avg_reputation.to_string(),
            stake_distribution.to_string(),
            health_score.to_string()
        ];
        let final_health_score = self.smart_contract_engine.call_contract(health_contract.id, "evaluate_health", args)?;

        Ok(final_health_score.parse::<f64>().map_err(|e| IcnError::Consensus(format!("Failed to parse final health score: {}", e)))?)
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
    fn test_update_computational_power() {
        let mut poc = setup_test_poc();

        poc.update_computational_power("peer1", 100, 200, vec!["ASIC".to_string()]).unwrap();

        let comp_power = poc.computational_power.get("peer1").unwrap();
        assert_eq!(comp_power.cpu_power, 100);
        assert_eq!(comp_power.gpu_power, 200);
        assert_eq!(comp_power.specialized_hardware, vec!["ASIC".to_string()]);
    }

    #[test]
    fn test_update_storage_provision() {
        let mut poc = setup_test_poc();

        poc.update_storage_provision("peer1", 1000, 0.95).unwrap();

        let storage = poc.storage_provision.get("peer1").unwrap();
        assert_eq!(storage.capacity, 1000);
        assert_eq!(storage.reliability, 0.95);
    }

    #[test]
    fn test_update_governance_participation() {
        let mut poc = setup_test_poc();

        poc.update_governance_participation("peer1", 5, 10, 3).unwrap();

        let governance = poc.governance_participation.get("peer1").unwrap();
        assert_eq!(governance.proposals_submitted, 5);
        assert_eq!(governance.votes_cast, 10);
        assert_eq!(governance.discussions_participated, 3);
    }

    #[test]
    fn test_calculate_network_impact() {
        let mut poc = setup_test_poc();

        poc.update_stake("peer1", 2000, "ICN".to_string(), 60).unwrap();
        poc.update_computational_power("peer1", 100, 200, vec!["ASIC".to_string()]).unwrap();
        poc.update_storage_provision("peer1", 1000, 0.95).unwrap();
        poc.update_governance_participation("peer1", 5, 10, 3).unwrap();

        let impact = poc.calculate_network_impact("peer1").unwrap();
        assert!(impact > 0.0 && impact <= 1.0, "Network impact should be between 0 and 1");
    }

    #[test]
    fn test_smart_contract_integration() {
        let mut poc = ProofOfCooperation::new();
        
        // Test smart contract deployment
        let contract_code = r#"
            function calculate_final_reward(peer_id, cooperation_score, reputation_score, network_impact, adjusted_reward) {
                return Math.floor(adjusted_reward * 1.1);  // 10% bonus
            }
        "#;
        let contract_id = poc.smart_contract_engine.deploy_contract(contract_code).unwrap();
        
        // Test smart contract execution
        poc.register_peer("test_peer");
        poc.update_cooperation_score("test_peer", 0.8).unwrap();
        poc.update_reputation("test_peer", true).unwrap();
        
        let reward = poc.calculate_reward("test_peer").unwrap();
        assert!(reward > 0, "Reward should be positive");
    }
}