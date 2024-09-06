use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use log::{info, warn, error};
use serde::{Serialize, Deserialize};

use crate::consensus::Consensus;

/// Constants for the Proof of Cooperation algorithm
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
const COOPERATION_SCORE_WINDOW: usize = 50; // Number of recent actions to consider for cooperation score

/// The main struct implementing the Proof of Cooperation consensus mechanism
#[derive(Clone)]
pub struct ProofOfCooperation {
    known_peers: Arc<RwLock<HashSet<String>>>,
    cooperation_scores: Arc<RwLock<HashMap<String, VecDeque<f64>>>>,
    reputation_scores: Arc<RwLock<HashMap<String, f64>>>,
    contribution_history: Arc<RwLock<HashMap<String, VecDeque<(u64, f64)>>>>,
    stake_info: Arc<RwLock<HashMap<String, StakeInfo>>>,
    computational_power: Arc<RwLock<HashMap<String, ComputationalPower>>>,
    storage_provision: Arc<RwLock<HashMap<String, StorageProvision>>>,
    governance_participation: Arc<RwLock<HashMap<String, GovernanceParticipation>>>,
    last_block_time: Arc<RwLock<u64>>,
}

/// Struct representing a peer's stake information
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StakeInfo {
    amount: u64,
    asset_type: String,
    duration: u64,
    last_update: u64,
}

/// Struct representing a peer's computational power contribution
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ComputationalPower {
    cpu_power: u64,
    gpu_power: u64,
    specialized_hardware: Vec<String>,
    last_update: u64,
}

/// Struct representing a peer's storage provision
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StorageProvision {
    capacity: u64,
    reliability: f64,
    uptime: f64,
    last_update: u64,
}

/// Struct representing a peer's participation in governance
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GovernanceParticipation {
    proposals_submitted: u64,
    votes_cast: u64,
    discussions_participated: u64,
    last_update: u64,
}

impl ProofOfCooperation {
    /// Creates a new instance of the ProofOfCooperation consensus mechanism
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: Arc::new(RwLock::new(HashSet::new())),
            cooperation_scores: Arc::new(RwLock::new(HashMap::new())),
            reputation_scores: Arc::new(RwLock::new(HashMap::new())),
            contribution_history: Arc::new(RwLock::new(HashMap::new())),
            stake_info: Arc::new(RwLock::new(HashMap::new())),
            computational_power: Arc::new(RwLock::new(HashMap::new())),
            storage_provision: Arc::new(RwLock::new(HashMap::new())),
            governance_participation: Arc::new(RwLock::new(HashMap::new())),
            last_block_time: Arc::new(RwLock::new(0)),
        }
    }

    /// Registers a new peer in the network
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer to be registered
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the peer was successfully registered, Err otherwise
    pub fn register_peer(&self, peer_id: &str) -> IcnResult<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?
            .as_secs();

        // Add the peer to the known peers set
        self.known_peers.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for known_peers".to_string()))?
            .insert(peer_id.to_string());
        
        // Initialize cooperation and reputation scores
        self.cooperation_scores.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?
            .insert(peer_id.to_string(), VecDeque::from(vec![1.0; COOPERATION_SCORE_WINDOW]));
        self.reputation_scores.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?
            .insert(peer_id.to_string(), 1.0);
        
        // Initialize contribution history
        self.contribution_history.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?
            .insert(peer_id.to_string(), VecDeque::new());
        
        // Initialize stake info
        self.stake_info.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?
            .insert(peer_id.to_string(), StakeInfo {
                amount: 0,
                asset_type: "ICN".to_string(),
                duration: 0,
                last_update: current_time,
            });
        
        // Initialize computational power
        self.computational_power.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for computational_power".to_string()))?
            .insert(peer_id.to_string(), ComputationalPower {
                cpu_power: 0,
                gpu_power: 0,
                specialized_hardware: Vec::new(),
                last_update: current_time,
            });
        
        // Initialize storage provision
        self.storage_provision.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for storage_provision".to_string()))?
            .insert(peer_id.to_string(), StorageProvision {
                capacity: 0,
                reliability: 1.0,
                uptime: 1.0,
                last_update: current_time,
            });
        
        // Initialize governance participation
        self.governance_participation.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for governance_participation".to_string()))?
            .insert(peer_id.to_string(), GovernanceParticipation {
                proposals_submitted: 0,
                votes_cast: 0,
                discussions_participated: 0,
                last_update: current_time,
            });

        info!("Registered peer: {}", peer_id);
        Ok(())
    }

    /// Selects validators for block validation
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<String>>` - A vector of selected validator IDs, or an error if selection fails
    fn select_validators(&self) -> IcnResult<Vec<String>> {
        let known_peers = self.known_peers.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for known_peers".to_string()))?;
        let stake_info = self.stake_info.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?;
        let reputation_scores = self.reputation_scores.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?;

        let validators: Vec<String> = known_peers.iter()
            .filter(|&peer_id| {
                let stake = stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
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
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to calculate the score for
    ///
    /// # Returns
    ///
    /// * `f64` - The calculated validator score
    fn calculate_validator_score(&self, peer_id: &str) -> f64 {
        let stake = self.stake_info.read()
            .unwrap_or_else(|_| panic!("Failed to acquire read lock for stake_info"))
            .get(peer_id)
            .map(|info| info.amount)
            .unwrap_or(0) as f64;

        let reputation = self.reputation_scores.read()
            .unwrap_or_else(|_| panic!("Failed to acquire read lock for reputation_scores"))
            .get(peer_id)
            .cloned()
            .unwrap_or(0.0);

        stake * reputation
    }

    /// Conducts a stake-weighted vote for block validation
    ///
    /// # Arguments
    ///
    /// * `validator_id` - The ID of the validator casting the vote
    /// * `block` - The block being voted on
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - The result of the vote (true for approval, false for rejection)
    fn stake_weighted_vote(&self, validator_id: &str, block: &Block) -> IcnResult<bool> {
        let stake = self.stake_info.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?
            .get(validator_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for validator: {}", validator_id)))?
            .amount as f64;

        let reputation = self.reputation_scores.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?
            .get(validator_id)
            .cloned()
            .unwrap_or(0.0);

        // Combine stake and reputation for voting power
        let voting_power = (stake * reputation).sqrt();
        
        // Use the block hash to generate a deterministic random value
        let random_value = self.hash_to_float(&block.hash);

        // Simplified voting logic
        Ok(voting_power * random_value > 0.5)
    }

    /// Converts a hash string to a float value between 0 and 1
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash string to convert
    ///
    /// # Returns
    ///
    /// * `f64` - A float value between 0 and 1
    fn hash_to_float(&self, hash: &str) -> f64 {
        let numeric_hash = u64::from_str_radix(&hash[0..16], 16).unwrap_or(0);
        numeric_hash as f64 / u64::MAX as f64
    }

    /// Records a contribution made by a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer making the contribution
    /// * `score` - The score of the contribution
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the contribution was successfully recorded, Err otherwise
    fn record_contribution(&self, peer_id: &str, score: f64) -> IcnResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();

        let mut history = self.contribution_history.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?;
        let peer_history = history.entry(peer_id.to_string()).or_insert_with(VecDeque::new);

        peer_history.push_back((timestamp, score));

        while peer_history.len() > MAX_RECENT_CONTRIBUTIONS {
            peer_history.pop_front();
        }

        Ok(())
    }

    /// Calculates the consistency of a peer's contributions
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to calculate consistency for
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - The calculated consistency score, or an error if calculation fails
    fn calculate_consistency(&self, peer_id: &str) -> IcnResult<f64> {
        let history = self.contribution_history.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for contribution_history".to_string()))?;
        let peer_history = history.get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        if peer_history.is_empty() {
            return Ok(1.0);
        }

        let recent_contributions: Vec<f64> = peer_history.iter().map(|&(_, score)| score).collect();
        let mean: f64 = recent_contributions.iter().sum::<f64>() / recent_contributions.len() as f64;
        let variance: f64 = recent_contributions.iter()
            .map(|&score| (score - mean).powi(2))
            .sum::<f64>() / recent_contributions.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

    /// Updates the reputation of a peer based on their actions
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose reputation is being updated
    /// * `positive_action` - Whether the action was positive (true) or negative (false)
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the reputation was successfully updated, Err otherwise
    pub fn update_reputation(&self, peer_id: &str, positive_action: bool) -> IcnResult<()> {
        let coop_score = {
            let cooperation_scores = self.cooperation_scores.read()
                .map_err(|_| IcnError::Consensus("Failed to acquire read lock for cooperation_scores".to_string()))?;
            cooperation_scores.get(peer_id)
                .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?
                .iter()
                .sum::<f64>() / COOPERATION_SCORE_WINDOW as f64
        };

        let consistency = self.calculate_consistency(peer_id)?;
        let network_impact = self.calculate_network_impact(peer_id)?;

        let mut rep_scores = self.reputation_scores.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?;
        let rep_score = rep_scores.entry(peer_id.to_string()).or_insert(1.0);

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
    /// * `peer_id` - The ID of the peer to calculate network impact for
    ///
    /// # Returns
    ///
    /// * `IcnResult<f64>` - The calculated network impact score, or an error if calculation fails
    fn calculate_network_impact(&self, peer_id: &str) -> IcnResult<f64> {
        let stake_info = self.stake_info.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No stake info for peer: {}", peer_id)))?
            .clone();

        let comp_power = self.computational_power.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for computational_power".to_string()))?
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No computational power info for peer: {}", peer_id)))?
            .clone();

        let storage = self.storage_provision.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for storage_provision".to_string()))?
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No storage provision info for peer: {}", peer_id)))?
            .clone();

        let governance = self.governance_participation.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock for governance_participation".to_string()))?
            .get(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("No governance participation info for peer: {}", peer_id)))?
            .clone();

        let stake_impact = (stake_info.amount as f64).log10() / 10.0;
        let comp_impact = (comp_power.cpu_power as f64 + comp_power.gpu_power as f64).log10() / 10.0;
        let storage_impact = (storage.capacity as f64).log10() / 20.0 * storage.reliability * storage.uptime;
        let governance_impact = (governance.proposals_submitted + governance.votes_cast + governance.discussions_participated) as f64 / 100.0;

        let total_impact = stake_impact + comp_impact + storage_impact + governance_impact;

        Ok(total_impact.min(1.0))
    }

    /// Updates the cooperation score of a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose cooperation score is being updated
    /// * `new_score` - The new cooperation score to be added
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the cooperation score was successfully updated, Err otherwise
    pub fn update_cooperation_score(&self, peer_id: &str, new_score: f64) -> IcnResult<()> {
        let mut scores = self.cooperation_scores.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?;
        let score_queue = scores.entry(peer_id.to_string()).or_insert_with(|| VecDeque::from(vec![1.0; COOPERATION_SCORE_WINDOW]));

        score_queue.push_back(new_score.max(0.0).min(1.0));
        if score_queue.len() > COOPERATION_SCORE_WINDOW {
            score_queue.pop_front();
        }

        self.record_contribution(peer_id, new_score)?;

        Ok(())
    }

    /// Retrieves a list of eligible peers for validation
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector of peer IDs that are eligible for validation
    pub fn get_eligible_peers(&self) -> Vec<String> {
        let known_peers = self.known_peers.read().unwrap();
        let stake_info = self.stake_info.read().unwrap();
        let reputation_scores = self.reputation_scores.read().unwrap();

        known_peers.iter()
            .filter(|&peer_id| {
                let stake = stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
                stake >= MIN_STAKE_FOR_SYBIL_RESISTANCE && reputation >= MIN_REPUTATION_FOR_SYBIL_RESISTANCE
            })
            .cloned()
            .collect()
    }

    /// Updates the stake information of a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose stake is being updated
    /// * `amount` - The new stake amount
    /// * `asset_type` - The type of asset being staked
    /// * `duration` - The duration of the stake
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the stake was successfully updated, Err otherwise
    pub fn update_stake(&self, peer_id: &str, amount: u64, asset_type: String, duration: u64) -> IcnResult<()> {
        let mut stake_info = self.stake_info.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?;
        
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();

        let peer_stake = stake_info.entry(peer_id.to_string()).or_insert(StakeInfo {
            amount: 0,
            asset_type: asset_type.clone(),
            duration: 0,
            last_update: current_time,
        });

        peer_stake.amount = amount;
        peer_stake.asset_type = asset_type;
        peer_stake.duration = duration;
        peer_stake.last_update = current_time;

        info!("Updated stake for peer {}: amount={}, asset_type={}, duration={}", 
              peer_id, peer_stake.amount, peer_stake.asset_type, peer_stake.duration);
        Ok(())
    }

    /// Updates the computational power information of a peer
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer whose computational power is being updated
    /// * `cpu_power` - The new CPU power value
    /// * `gpu_power` - The new GPU power value
    /// * `specialized_hardware` - A vector of specialized hardware identifiers
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the computational power was successfully updated, Err otherwise
    pub fn update_computational_power(&self, peer_id: &str, cpu_power: u64, gpu_power: u64, specialized_hardware: Vec<String>) -> IcnResult<()> {
        let mut comp_power = self.computational_power.write()
            .map_err(|_| IcnError::Consensus("Failed to acquire write lock for computational_power".to_string()))?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
            .as_secs();

        let peer_power = comp_power.entry(peer_id.to_string()).or_insert(ComputationalPower {
            cpu_power: 0,
            gpu_power: 0,
            specialized_hardware: Vec::new(),
            last_update: current_time,
        });

        peer_power.cpu_power = cpu_power;
        peer_power.gpu_power = gpu_power;
        peer_power.specialized_hardware = specialized_hardware.clone();
        peer_power.last_update = current_time;

        info!("Updated computational power for peer {}: cpu_power={}, gpu_power={}, specialized_hardware={:?}", 
              peer_id, cpu_power, gpu_power, specialized_hardware);
              Ok(())
            }
        
            /// Updates the storage provision information of a peer
            ///
            /// # Arguments
            ///
            /// * `peer_id` - The ID of the peer whose storage provision is being updated
            /// * `capacity` - The new storage capacity
            /// * `reliability` - The new reliability score
            /// * `uptime` - The new uptime percentage
            ///
            /// # Returns
            ///
            /// * `IcnResult<()>` - Ok if the storage provision was successfully updated, Err otherwise
            pub fn update_storage_provision(&self, peer_id: &str, capacity: u64, reliability: f64, uptime: f64) -> IcnResult<()> {
                let mut storage_info = self.storage_provision.write()
                    .map_err(|_| IcnError::Consensus("Failed to acquire write lock for storage_provision".to_string()))?;
        
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
                    .as_secs();
        
                let peer_storage = storage_info.entry(peer_id.to_string()).or_insert(StorageProvision {
                    capacity: 0,
                    reliability: 1.0,
                    uptime: 1.0,
                    last_update: current_time,
                });
        
                peer_storage.capacity = capacity;
                peer_storage.reliability = reliability.max(0.0).min(1.0);
                peer_storage.uptime = uptime.max(0.0).min(1.0);
                peer_storage.last_update = current_time;
        
                info!("Updated storage provision for peer {}: capacity={}, reliability={}, uptime={}", 
                      peer_id, capacity, reliability, uptime);
                Ok(())
            }
        
            /// Updates the governance participation information of a peer
            ///
            /// # Arguments
            ///
            /// * `peer_id` - The ID of the peer whose governance participation is being updated
            /// * `proposals_submitted` - The number of proposals submitted
            /// * `votes_cast` - The number of votes cast
            /// * `discussions_participated` - The number of discussions participated in
            ///
            /// # Returns
            ///
            /// * `IcnResult<()>` - Ok if the governance participation was successfully updated, Err otherwise
            pub fn update_governance_participation(&self, peer_id: &str, proposals_submitted: u64, votes_cast: u64, discussions_participated: u64) -> IcnResult<()> {
                let mut governance_info = self.governance_participation.write()
                    .map_err(|_| IcnError::Consensus("Failed to acquire write lock for governance_participation".to_string()))?;
        
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| IcnError::Other(format!("System time error: {}", e)))?
                    .as_secs();
        
                let peer_governance = governance_info.entry(peer_id.to_string()).or_insert(GovernanceParticipation {
                    proposals_submitted: 0,
                    votes_cast: 0,
                    discussions_participated: 0,
                    last_update: current_time,
                });
        
                peer_governance.proposals_submitted = proposals_submitted;
                peer_governance.votes_cast = votes_cast;
                peer_governance.discussions_participated = discussions_participated;
                peer_governance.last_update = current_time;
        
                info!("Updated governance participation for peer {}: proposals_submitted={}, votes_cast={}, discussions_participated={}", 
                      peer_id, peer_governance.proposals_submitted, peer_governance.votes_cast, peer_governance.discussions_participated);
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
            /// * `bool` - True if the peer passes the Sybil resistance check, false otherwise
            pub fn sybil_resistant_check(&self, peer_id: &str) -> bool {
                let stake = self.stake_info.read().unwrap().get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = self.reputation_scores.read().unwrap().get(peer_id).cloned().unwrap_or(0.0);
                let governance_participation = self.governance_participation.read().unwrap().get(peer_id)
                    .map(|info| info.proposals_submitted + info.votes_cast + info.discussions_participated)
                    .unwrap_or(0);
        
                stake >= MIN_STAKE_FOR_SYBIL_RESISTANCE &&
                reputation >= MIN_REPUTATION_FOR_SYBIL_RESISTANCE &&
                governance_participation >= MIN_GOVERNANCE_PARTICIPATION_FOR_SYBIL_RESISTANCE
            }
        
            /// Calculates the reward for a peer based on their contributions and reputation
            ///
            /// # Arguments
            ///
            /// * `peer_id` - The ID of the peer to calculate the reward for
            ///
            /// # Returns
            ///
            /// * `IcnResult<u64>` - The calculated reward amount, or an error if calculation fails
            pub fn calculate_reward(&self, peer_id: &str) -> IcnResult<u64> {
                let cooperation_score = {
                    let scores = self.cooperation_scores.read()
                        .map_err(|_| IcnError::Consensus("Failed to acquire read lock for cooperation_scores".to_string()))?;
                    scores.get(peer_id)
                        .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?
                        .iter()
                        .sum::<f64>() / COOPERATION_SCORE_WINDOW as f64
                };
        
                let reputation_score = *self.reputation_scores.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?
                    .get(peer_id)
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
            /// * `IcnResult<HashMap<String, u64>>` - A map of peer IDs to their calculated rewards, or an error if distribution fails
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
            /// * `severity` - The severity of the misconduct (0.0 to 1.0)
            ///
            /// # Returns
            ///
            /// * `IcnResult<()>` - Ok if the penalty was successfully applied, Err otherwise
            pub fn apply_penalty(&self, peer_id: &str, severity: f64) -> IcnResult<()> {
                let severity = severity.max(0.0).min(1.0);
        
                // Update cooperation score
                {
                    let mut scores = self.cooperation_scores.write()
                        .map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?;
                    if let Some(score_queue) = scores.get_mut(peer_id) {
                        let penalty_score = 1.0 - severity;
                        score_queue.push_back(penalty_score);
                        if score_queue.len() > COOPERATION_SCORE_WINDOW {
                            score_queue.pop_front();
                        }
                    }
                }
        
                // Update reputation score
                {
                    let mut scores = self.reputation_scores.write()
                        .map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?;
                    if let Some(score) = scores.get_mut(peer_id) {
                        *score *= 1.0 - severity;
                    }
                }
        
                // Update stake if severity is high
                if severity > 0.5 {
                    let mut stake_info = self.stake_info.write()
                        .map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?;
                    if let Some(info) = stake_info.get_mut(peer_id) {
                        info.amount = (info.amount as f64 * (1.0 - severity * 0.5)) as u64;
                    }
                }
        
                info!("Applied penalty to peer {} with severity {}", peer_id, severity);
                Ok(())
            }
        
            /// Evaluates the overall health of the network
            ///
            /// # Returns
            ///
            /// * `IcnResult<f64>` - A score representing the network health (0.0 to 1.0), or an error if evaluation fails
            pub fn evaluate_network_health(&self) -> IcnResult<f64> {
                let cooperation_scores = self.cooperation_scores.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for cooperation_scores".to_string()))?;
                let reputation_scores = self.reputation_scores.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?;
                let stake_info = self.stake_info.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?;
        
                let avg_cooperation = cooperation_scores.values()
                    .map(|scores| scores.iter().sum::<f64>() / scores.len() as f64)
                    .sum::<f64>() / cooperation_scores.len() as f64;
                let avg_reputation = reputation_scores.values().sum::<f64>() / reputation_scores.len() as f64;
        
                let total_stake: u64 = stake_info.values().map(|info| info.amount).sum();
                let stake_distribution = 1.0 - (stake_info.values()
                    .map(|info| info.amount as f64 / total_stake as f64)
                    .map(|x| x * x)
                    .sum::<f64>()
                    .sqrt());
        
                let health_score = (avg_cooperation + avg_reputation + stake_distribution) / 3.0;
        
                Ok(health_score)
            }
        }
        
        impl Consensus for ProofOfCooperation {
            fn validate(&self, block: &Block) -> IcnResult<bool> {
                let known_peers = self.known_peers.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for known_peers".to_string()))?;
                if !known_peers.contains(&block.proposer_id) {
                    return Err(IcnError::Consensus(format!("Unknown proposer: {}", block.proposer_id)));
                }
        
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?
                    .as_secs();
        
                let last_block_time = *self.last_block_time.read()
                    .map_err(|_| IcnError::Consensus("Failed to acquire read lock for last_block_time".to_string()))?;
                if current_time < last_block_time + BLOCK_TIME_THRESHOLD {
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
        
            fn select_proposer(&self) -> IcnResult<String> {
                let eligible_peers = self.get_eligible_peers();
                if eligible_peers.is_empty() {
                    return Err(IcnError::Consensus("No eligible proposers available".to_string()));
                }
        
                let total_score: f64 = eligible_peers.iter()
                    .map(|peer_id| self.calculate_validator_score(peer_id))
                    .sum();
        
                let mut rng = rand::thread_rng();
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
        
            fn get_eligible_peers(&self) -> Vec<String> {
                self.get_eligible_peers() // This calls the public method we defined earlier
            }
        
            fn update_state(&self, latest_block: &Block) -> IcnResult<()> {
                let mut last_block_time = self.last_block_time.write()
                    .map_err(|_| IcnError::Consensus("Failed to acquire write lock for last_block_time".to_string()))?;
                *last_block_time = latest_block.timestamp;
                Ok(())
            }
        
            fn initialize(&self, latest_block: &Block) -> IcnResult<()> {
                let mut last_block_time = self.last_block_time.write()
                    .map_err(|_| IcnError::Consensus("Failed to acquire write lock for last_block_time".to_string()))?;
                *last_block_time = latest_block.timestamp;
                
                // Additional initialization logic can be added here
                // For example, we could initialize the known peers list, set initial stakes, etc.
                
                Ok(())
            }
        
            fn handle_network_event(&self, event: NetworkEvent) -> IcnResult<()> {
                match event {
                    NetworkEvent::PeerConnected(peer_id) => {
                        self.register_peer(&peer_id)?;
                        info!("New peer connected: {}", peer_id);
                    },
                    NetworkEvent::PeerDisconnected(peer_id) => {
                        // Remove peer from all data structures
                        self.remove_peer(&peer_id)?;
                        info!("Peer disconnected: {}", peer_id);
                    },
                    NetworkEvent::NetworkPartitionDetected => {
                        warn!("Network partition detected");
                        // Implement logic to handle network partition
                        // For example, we might want to temporarily adjust our consensus parameters
                        // or enter a more conservative validation mode
                    },
                    NetworkEvent::NetworkReunified => {
                        info!("Network reunified");
// Implement logic to handle network reunification
                // Here we might want to resynchronize our state with the network
                // and return to normal consensus operation
                self.resynchronize_network_state()?;
            },
            NetworkEvent::NetworkConditionChanged(condition) => {
                match condition {
                    NetworkCondition::Normal => info!("Network condition: Normal"),
                    NetworkCondition::HighLatency => warn!("Network condition: High Latency"),
                    NetworkCondition::Congested => warn!("Network condition: Congested"),
                    NetworkCondition::Unstable => error!("Network condition: Unstable"),
                }
                // Adjust consensus parameters based on network condition
                // For example, we might want to adjust our block time or validation thresholds
                self.adjust_parameters_for_network_condition(condition)?;
            },
        }
        Ok(())
    }
}

impl ProofOfCooperation {
    /// Removes a peer from all data structures
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The ID of the peer to remove
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if the peer was successfully removed, Err otherwise
    fn remove_peer(&self, peer_id: &str) -> IcnResult<()> {
        {
            let mut known_peers = self.known_peers.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for known_peers".to_string()))?;
            known_peers.remove(peer_id);
        }
        {
            let mut cooperation_scores = self.cooperation_scores.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?;
            cooperation_scores.remove(peer_id);
        }
        {
            let mut reputation_scores = self.reputation_scores.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?;
            reputation_scores.remove(peer_id);
        }
        {
            let mut contribution_history = self.contribution_history.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?;
            contribution_history.remove(peer_id);
        }
        {
            let mut stake_info = self.stake_info.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?;
            stake_info.remove(peer_id);
        }
        {
            let mut computational_power = self.computational_power.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for computational_power".to_string()))?;
            computational_power.remove(peer_id);
        }
        {
            let mut storage_provision = self.storage_provision.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for storage_provision".to_string()))?;
            storage_provision.remove(peer_id);
        }
        {
            let mut governance_participation = self.governance_participation.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock for governance_participation".to_string()))?;
            governance_participation.remove(peer_id);
        }
        Ok(())
    }

    /// Adjusts consensus parameters based on network conditions
    ///
    /// # Arguments
    ///
    /// * `condition` - The current network condition
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if parameters were successfully adjusted, Err otherwise
    fn adjust_parameters_for_network_condition(&self, condition: NetworkCondition) -> IcnResult<()> {
        // This is a placeholder implementation. In a real-world scenario, you would adjust
        // various parameters based on the network condition.
        match condition {
            NetworkCondition::Normal => {
                // Reset to default parameters
                info!("Resetting consensus parameters to default values");
            },
            NetworkCondition::HighLatency => {
                // Increase block time threshold
                // Decrease number of required validators
                warn!("Adjusting consensus parameters for high latency");
            },
            NetworkCondition::Congested => {
                // Increase block size limit
                // Decrease transaction throughput
                warn!("Adjusting consensus parameters for network congestion");
            },
            NetworkCondition::Unstable => {
                // Increase confirmation threshold
                // Decrease block production rate
                error!("Adjusting consensus parameters for unstable network");
            },
        }
        Ok(())
    }

    /// Resynchronizes the network state after a network partition or reconnection
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Ok if resynchronization was successful, Err otherwise
    fn resynchronize_network_state(&self) -> IcnResult<()> {
        // Placeholder implementation
        // In a real-world scenario, this would involve complex logic to:
        // 1. Identify the longest/most valid chain
        // 2. Request and verify missing blocks
        // 3. Reapply transactions that were not included in the main chain
        // 4. Update the local state to match the network consensus
        info!("Resynchronizing network state");
        // ... implementation details ...
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_poc() -> ProofOfCooperation {
        let poc = ProofOfCooperation::new();
        poc.register_peer("peer1").unwrap();
        poc.register_peer("peer2").unwrap();
        poc.register_peer("peer3").unwrap();
        poc
    }

    #[test]
    fn test_register_and_validate_peer() {
        let poc = ProofOfCooperation::new();
        assert!(poc.register_peer("peer1").is_ok());

        let known_peers = poc.known_peers.read().unwrap();
        assert!(known_peers.contains("peer1"));
        assert!(!known_peers.contains("unknown_peer"));

        let block = Block::new(0, vec![], "previous_hash".to_string(), "peer1".to_string());
        assert!(poc.validate(&block).is_ok());

        let invalid_block = Block::new(0, vec![], "previous_hash".to_string(), "unknown_peer".to_string());
        assert!(poc.validate(&invalid_block).is_err());
    }

    #[test]
    fn test_update_and_calculate_reputation() {
        let poc = setup_test_poc();

        assert!(poc.update_cooperation_score("peer1", 0.8).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());

        let reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap_or(0.0);
        assert!(reputation > 0.5, "Reputation should increase after positive action");

        assert!(poc.update_cooperation_score("peer1", 0.2).is_ok());
        assert!(poc.update_reputation("peer1", false).is_ok());

        let new_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap_or(0.0);
        assert!(new_reputation < reputation, "Reputation should decrease after negative action");
    }

    #[test]
    fn test_sybil_resistant_check() {
        let poc = setup_test_poc();

        assert!(poc.update_stake("peer1", 2000, "ICN".to_string(), 60).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());
        assert!(poc.update_governance_participation("peer1", 5, 10, 3).is_ok());

        assert!(poc.update_stake("peer2", 500, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_reputation("peer2", false).is_ok());
        assert!(poc.update_governance_participation("peer2", 1, 2, 1).is_ok());

        assert!(poc.sybil_resistant_check("peer1"), "Good peer should pass Sybil resistance check");
        assert!(!poc.sybil_resistant_check("peer2"), "Bad peer should fail Sybil resistance check");
    }

    #[test]
    fn test_calculate_and_distribute_rewards() {
        let poc = setup_test_poc();

        assert!(poc.update_stake("peer1", 1000, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_cooperation_score("peer1", 0.9).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());

        assert!(poc.update_stake("peer2", 500, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_cooperation_score("peer2", 0.5).is_ok());
        assert!(poc.update_reputation("peer2", true).is_ok());

        assert!(poc.update_stake("peer3", 2000, "ICN".to_string(), 60).is_ok());
        assert!(poc.update_cooperation_score("peer3", 0.7).is_ok());
        assert!(poc.update_reputation("peer3", true).is_ok());

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
        let poc = setup_test_poc();

        assert!(poc.update_cooperation_score("peer1", 0.8).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());
        assert!(poc.update_stake("peer1", 1000, "ICN".to_string(), 30).is_ok());

        let initial_cooperation = poc.cooperation_scores.read().unwrap().get("peer1").unwrap().iter().sum::<f64>() / COOPERATION_SCORE_WINDOW as f64;
        let initial_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let initial_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(poc.apply_penalty("peer1", 0.3).is_ok());

        let after_penalty_cooperation = poc.cooperation_scores.read().unwrap().get("peer1").unwrap().iter().sum::<f64>() / COOPERATION_SCORE_WINDOW as f64;
        let after_penalty_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let after_penalty_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(after_penalty_cooperation < initial_cooperation, "Cooperation score should decrease after penalty");
        assert!(after_penalty_reputation < initial_reputation, "Reputation score should decrease after penalty");
        assert_eq!(after_penalty_stake, initial_stake, "Stake should not change for moderate penalty");

        assert!(poc.apply_penalty("peer1", 0.8).is_ok());

        let final_cooperation = poc.cooperation_scores.read().unwrap().get("peer1").unwrap().iter().sum::<f64>() / COOPERATION_SCORE_WINDOW as f64;
        let final_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let final_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(final_cooperation < after_penalty_cooperation, "Cooperation score should decrease further after severe penalty");
        assert!(final_reputation < after_penalty_reputation, "Reputation score should decrease further after severe penalty");
        assert!(final_stake < after_penalty_stake, "Stake should decrease for severe penalty");
    }

    #[test]
    fn test_network_health_evaluation() {
        let poc = setup_test_poc();

        for (i, peer) in ["peer1", "peer2", "peer3"].iter().enumerate() {
            assert!(poc.update_cooperation_score(peer, 0.8 + i as f64 * 0.1).is_ok());
            assert!(poc.update_reputation(peer, true).is_ok());
            assert!(poc.update_stake(peer, 1000 + i as u64 * 500, "ICN".to_string(), 30).is_ok());
        }

        let health_score = poc.evaluate_network_health().unwrap();
        assert!(health_score > 0.8, "Network health should be high for a well-balanced network");

        assert!(poc.update_cooperation_score("peer1", 0.3).is_ok());
        assert!(poc.update_reputation("peer1", false).is_ok());
        assert!(poc.update_stake("peer2", 10000, "ICN".to_string(), 30).is_ok());

        let new_health_score = poc.evaluate_network_health().unwrap();
        assert!(new_health_score < health_score, "Network health should decrease for an imbalanced network");
    }

    #[test]
    fn test_select_proposer() {
        let poc = setup_test_poc();

        // Set up peers with different stakes and reputations
        assert!(poc.update_stake("peer1", 2000, "ICN".to_string(), 60).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());
        assert!(poc.update_cooperation_score("peer1", 0.9).is_ok());

        assert!(poc.update_stake("peer2", 1000, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_reputation("peer2", true).is_ok());
        assert!(poc.update_cooperation_score("peer2", 0.7).is_ok());

        assert!(poc.update_stake("peer3", 3000, "ICN".to_string(), 90).is_ok());
        assert!(poc.update_reputation("peer3", true).is_ok());
        assert!(poc.update_cooperation_score("peer3", 0.8).is_ok());

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
        let poc = setup_test_poc();

        // Test PeerConnected event
        assert!(poc.handle_network_event(NetworkEvent::PeerConnected("peer4".to_string())).is_ok());
        assert!(poc.known_peers.read().unwrap().contains("peer4"));

        // Test PeerDisconnected event
        assert!(poc.handle_network_event(NetworkEvent::PeerDisconnected("peer2".to_string())).is_ok());
        assert!(!poc.known_peers.read().unwrap().contains("peer2"));

        // Test NetworkConditionChanged event
        assert!(poc.handle_network_event(NetworkEvent::NetworkConditionChanged(NetworkCondition::HighLatency)).is_ok());

        // Test NetworkPartitionDetected and NetworkReunified events
        assert!(poc.handle_network_event(NetworkEvent::NetworkPartitionDetected).is_ok());
        assert!(poc.handle_network_event(NetworkEvent::NetworkReunified).is_ok());
    }

    #[test]
    fn test_adjust_parameters_for_network_condition() {
        let poc = setup_test_poc();

        assert!(poc.adjust_parameters_for_network_condition(NetworkCondition::Normal).is_ok());
        assert!(poc.adjust_parameters_for_network_condition(NetworkCondition::HighLatency).is_ok());
        assert!(poc.adjust_parameters_for_network_condition(NetworkCondition::Congested).is_ok());
        assert!(poc.adjust_parameters_for_network_condition(NetworkCondition::Unstable).is_ok());
    }

    #[test]
    fn test_resynchronize_network_state() {
        let poc = setup_test_poc();

        assert!(poc.resynchronize_network_state().is_ok());
    }
}