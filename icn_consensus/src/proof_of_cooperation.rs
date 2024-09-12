use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use icn_shared::{Block, IcnError, IcnResult};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::consensus::Consensus;

/// Constants for Proof of Cooperation consensus mechanism
const DEFAULT_REPUTATION_DECAY: f64 = 0.95;  // Decay factor for inactive members
const WEIGHT_CONSISTENCY: f64 = 0.3;
const WEIGHT_QUALITY: f64 = 0.4;
const WEIGHT_IMPACT: f64 = 0.3;
const MAX_ACTION_HISTORY: usize = 100;
const MIN_STAKE_SYBIL: u64 = 1000;
const MIN_REPUTATION_SYBIL: f64 = 0.7;
const MIN_GOV_PARTICIPATION: u64 = 10;
const DEFAULT_BLOCK_TIME: u64 = 10;  // Minimum block interval
const MAX_VALIDATOR_COUNT: usize = 10;
const COOP_SCORE_WINDOW: usize = 50;  // Cooperation score window

/// Struct representing a peer's stake information
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StakeInfo {
    amount: u64,
    asset_type: String,
    duration: u64,
    last_update: u64,
}

/// Struct representing a peer's computational power
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

impl ProofOfCooperation {
    /// Creates a new instance of ProofOfCooperation consensus mechanism
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
    pub fn register_peer(&self, peer_id: &str) -> IcnResult<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?
            .as_secs();

        {
            let mut known_peers = self.known_peers.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for known_peers".to_string()))?;
            if known_peers.contains(peer_id) {
                return Err(IcnError::Consensus("Peer is already registered".to_string()));
            }
            known_peers.insert(peer_id.to_string());
        }

        self.initialize_peer_data(peer_id, current_time)?;
        info!("Registered peer: {}", peer_id);
        Ok(())
    }

    /// Helper function to initialize data for a new peer
    fn initialize_peer_data(&self, peer_id: &str, current_time: u64) -> IcnResult<()> {
        self.cooperation_scores.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?
            .insert(peer_id.to_string(), VecDeque::from(vec![1.0; COOP_SCORE_WINDOW]));

        self.reputation_scores.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?
            .insert(peer_id.to_string(), 1.0);

        self.contribution_history.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?
            .insert(peer_id.to_string(), VecDeque::new());

        self.stake_info.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?
            .insert(peer_id.to_string(), StakeInfo {
                amount: 0,
                asset_type: "ICN".to_string(),
                duration: 0,
                last_update: current_time,
            });

        self.computational_power.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for computational_power".to_string()))?
            .insert(peer_id.to_string(), ComputationalPower {
                cpu_power: 0,
                gpu_power: 0,
                specialized_hardware: Vec::new(),
                last_update: current_time,
            });

        self.storage_provision.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for storage_provision".to_string()))?
            .insert(peer_id.to_string(), StorageProvision {
                capacity: 0,
                reliability: 1.0,
                uptime: 1.0,
                last_update: current_time,
            });

        self.governance_participation.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for governance_participation".to_string()))?
            .insert(peer_id.to_string(), GovernanceParticipation {
                proposals_submitted: 0,
                votes_cast: 0,
                discussions_participated: 0,
                last_update: current_time,
            });

        Ok(())
    }

    /// Selects validators for block validation
    fn select_validators(&self) -> IcnResult<Vec<String>> {
        let known_peers = self.known_peers.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for known_peers".to_string()))?;
        let stake_info = self.stake_info.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?;
        let reputation_scores = self.reputation_scores.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?;

        let validators: Vec<String> = known_peers
            .iter()
            .filter(|&peer_id| {
                let stake = stake_info.get(peer_id).map(|info| info.amount).unwrap_or(0);
                let reputation = reputation_scores.get(peer_id).cloned().unwrap_or(0.0);
                stake > MIN_STAKE_SYBIL && reputation > MIN_REPUTATION_SYBIL
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

        Ok(sorted_validators.into_iter().take(MAX_VALIDATOR_COUNT).collect())
    }

    /// Calculates the score of a validator based on stake and reputation
    fn calculate_validator_score(&self, peer_id: &str) -> f64 {
        let stake = self.stake_info.read().unwrap()
            .get(peer_id).map(|info| info.amount).unwrap_or(0) as f64;

        let reputation = self.reputation_scores.read().unwrap()
            .get(peer_id).cloned().unwrap_or(0.0);

        stake * reputation
    }

    /// Conducts a stake-weighted vote for block validation
    fn stake_weighted_vote(&self, validator_id: &str, block: &Block) -> IcnResult<bool> {
        let stake = self.stake_info.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?
            .get(validator_id).ok_or_else(|| IcnError::Consensus(format!("No stake info for validator: {}", validator_id)))?.amount as f64;

        let reputation = self.reputation_scores.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for reputation_scores".to_string()))?
            .get(validator_id).cloned().unwrap_or(0.0);

        let voting_power = (stake * reputation).sqrt();
        let random_value = self.hash_to_float(&block.hash);
        Ok(voting_power * random_value > 0.5)
    }

    /// Converts a hash string to a float value between 0 and 1
    fn hash_to_float(&self, hash: &str) -> f64 {
        let numeric_hash = u64::from_str_radix(&hash[0..16], 16).unwrap_or(0);
        numeric_hash as f64 / u64::MAX as f64
    }

    /// Validates a block by selecting validators and conducting a stake-weighted vote
    fn validate(&self, block: &Block) -> IcnResult<bool> {
        let known_peers = self.known_peers.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for known_peers".to_string()))?;
        if !known_peers.contains(&block.proposer_id) {
            return Err(IcnError::Consensus(format!("Unknown proposer: {}", block.proposer_id)));
        }

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?.as_secs();
        let last_block_time = *self.last_block_time.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for last_block_time".to_string()))?;
        if current_time < last_block_time + DEFAULT_BLOCK_TIME {
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

        self.update_reputation(&block.proposer_id, is_valid)?;
        Ok(is_valid)
    }

    /// Records a contribution made by a peer
    fn record_contribution(&self, peer_id: &str, score: f64) -> IcnResult<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?.as_secs();

        let mut history = self.contribution_history.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?;
        let peer_history = history.entry(peer_id.to_string()).or_insert_with(VecDeque::new);
        peer_history.push_back((timestamp, score));

        while peer_history.len() > MAX_ACTION_HISTORY {
            peer_history.pop_front();
        }

        Ok(())
    }

    /// Updates the reputation of a peer based on their actions
    pub fn update_reputation(&self, peer_id: &str, positive_action: bool) -> IcnResult<()> {
        let coop_score = {
            let cooperation_scores = self.cooperation_scores.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for cooperation_scores".to_string()))?;
            cooperation_scores.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?.iter().sum::<f64>() / COOP_SCORE_WINDOW as f64
        };

        let consistency = self.calculate_consistency(peer_id)?;
        let network_impact = self.calculate_network_impact(peer_id)?;

        let mut rep_scores = self.reputation_scores.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?;
        let rep_score = rep_scores.entry(peer_id.to_string()).or_insert(1.0);
        let quality_factor = if positive_action { 1.1 } else { 0.9 };

        let new_rep_score = (
            WEIGHT_CONSISTENCY * consistency +
            WEIGHT_QUALITY * coop_score * quality_factor +
            WEIGHT_IMPACT * network_impact
        ) * DEFAULT_REPUTATION_DECAY + (1.0 - DEFAULT_REPUTATION_DECAY) * *rep_score;

        *rep_score = new_rep_score.max(0.0).min(1.0);
        Ok(())
    }

    /// Calculates the consistency of a peer's contributions
    fn calculate_consistency(&self, peer_id: &str) -> IcnResult<f64> {
        let history = self.contribution_history.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for contribution_history".to_string()))?;
        let peer_history = history.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;

        if peer_history.is_empty() {
            return Ok(1.0);
        }

        let recent_contributions: Vec<f64> = peer_history.iter().map(|&(_, score)| score).collect();
        let mean: f64 = recent_contributions.iter().sum::<f64>() / recent_contributions.len() as f64;
        let variance: f64 = recent_contributions.iter().map(|&score| (score - mean).powi(2)).sum::<f64>() / recent_contributions.len() as f64;
        let std_deviation = variance.sqrt();

        Ok(1.0 / (1.0 + std_deviation))
    }

    /// Calculates the network impact of a peer
    fn calculate_network_impact(&self, peer_id: &str) -> IcnResult<f64> {
        let stake_info = self.stake_info.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for stake_info".to_string()))?.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("No stake info for peer: {}", peer_id)))?.clone();
        let comp_power = self.computational_power.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for computational_power".to_string()))?.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("No computational power info for peer: {}", peer_id)))?.clone();
        let storage = self.storage_provision.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for storage_provision".to_string()))?.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("No storage provision info for peer: {}", peer_id)))?.clone();
        let governance = self.governance_participation.read().map_err(|_| IcnError::Consensus("Failed to acquire read lock for governance_participation".to_string()))?.get(peer_id).ok_or_else(|| IcnError::Consensus(format!("No governance participation info for peer: {}", peer_id)))?.clone();

        let stake_impact = (stake_info.amount as f64).log10() / 10.0;
        let comp_impact = (comp_power.cpu_power as f64 + comp_power.gpu_power as f64).log10() / 10.0;
        let storage_impact = (storage.capacity as f64).log10() / 20.0 * storage.reliability * storage.uptime;
        let governance_impact = (governance.proposals_submitted + governance.votes_cast + governance.discussions_participated) as f64 / 100.0;

        let total_impact = stake_impact + comp_impact + storage_impact + governance_impact;
        Ok(total_impact.min(1.0))
    }
}

impl Consensus for ProofOfCooperation {
    fn validate(&self, block: &Block) -> IcnResult<bool> {
        self.validate(block)
    }

    fn select_proposer(&self) -> IcnResult<String> {
        let eligible_peers = self.get_eligible_peers();
        if eligible_peers.is_empty() {
            return Err(IcnError::Consensus("No eligible proposers available".to_string()));
        }

        let total_score: f64 = eligible_peers.iter().map(|peer_id| self.calculate_validator_score(peer_id)).sum();
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
        self.get_eligible_peers()  // Calls the public method defined earlier
    }

    fn update_state(&self, latest_block: &Block) -> IcnResult<()> {
        let mut last_block_time = self.last_block_time.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for last_block_time".to_string()))?;
        *last_block_time = latest_block.timestamp;
        Ok(())
    }

    fn initialize(&self, latest_block: &Block) -> IcnResult<()> {
        let mut last_block_time = self.last_block_time.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for last_block_time".to_string()))?;
        *last_block_time = latest_block.timestamp;
        Ok(())
    }

    fn handle_network_event(&self, event: crate::NetworkEvent) -> IcnResult<()> {
        match event {
            crate::NetworkEvent::PeerConnected(peer_id) => {
                self.register_peer(&peer_id)?;
                info!("New peer connected: {}", peer_id);
            },
            crate::NetworkEvent::PeerDisconnected(peer_id) => {
                self.remove_peer(&peer_id)?;
                info!("Peer disconnected: {}", peer_id);
            },
            crate::NetworkEvent::NetworkPartitionDetected => {
                warn!("Network partition detected");
            },
            crate::NetworkEvent::NetworkReunified => {
                info!("Network reunified");
            },
            crate::NetworkEvent::NetworkConditionChanged(condition) => {
                self.adjust_parameters_for_network_condition(condition)?;
            },
        }
        Ok(())
    }
}

impl ProofOfCooperation {
    fn remove_peer(&self, peer_id: &str) -> IcnResult<()> {
        self.known_peers.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for known_peers".to_string()))?.remove(peer_id);
        self.cooperation_scores.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for cooperation_scores".to_string()))?.remove(peer_id);
        self.reputation_scores.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for reputation_scores".to_string()))?.remove(peer_id);
        self.contribution_history.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for contribution_history".to_string()))?.remove(peer_id);
        self.stake_info.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for stake_info".to_string()))?.remove(peer_id);
        self.computational_power.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for computational_power".to_string()))?.remove(peer_id);
        self.storage_provision.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for storage_provision".to_string()))?.remove(peer_id);
        self.governance_participation.write().map_err(|_| IcnError::Consensus("Failed to acquire write lock for governance_participation".to_string()))?.remove(peer_id);
        Ok(())
    }

    fn adjust_parameters_for_network_condition(&self, condition: crate::NetworkCondition) -> IcnResult<()> {
        match condition {
            crate::NetworkCondition::Normal => info!("Network condition: Normal"),
            crate::NetworkCondition::HighLatency => warn!("Adjusting for high latency"),
            crate::NetworkCondition::Congested => warn!("Adjusting for congestion"),
            crate::NetworkCondition::Unstable => error!("Adjusting for unstable network"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_shared::Block;

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
        assert!(poc.update_reputation("peer1", true).is_ok());

        let reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap_or(0.0);
        assert!(reputation > 0.5);

        assert!(poc.update_reputation("peer1", false).is_ok());
        let new_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap_or(0.0);
        assert!(new_reputation < reputation);
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

        assert!(poc.sybil_resistant_check("peer1"));
        assert!(!poc.sybil_resistant_check("peer2"));
    }

    #[test]
    fn test_calculate_and_distribute_rewards() {
        let poc = setup_test_poc();
        assert!(poc.update_stake("peer1", 1000, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_reputation("peer1", true).is_ok());

        assert!(poc.update_stake("peer2", 500, "ICN".to_string(), 30).is_ok());
        assert!(poc.update_reputation("peer2", true).is_ok());

        assert!(poc.update_stake("peer3", 2000, "ICN".to_string(), 60).is_ok());
        assert!(poc.update_reputation("peer3", true).is_ok());

        let reward1 = poc.calculate_reward("peer1").unwrap();
        let reward2 = poc.calculate_reward("peer2").unwrap();
        let reward3 = poc.calculate_reward("peer3").unwrap();

        assert!(reward1 > reward2);
        assert!(reward3 > reward1);

        let rewards = poc.distribute_rewards().unwrap();
        assert_eq!(rewards.len(), 3);
        assert_eq!(rewards.get("peer1"), Some(&reward1));
        assert_eq!(rewards.get("peer2"), Some(&reward2));
        assert_eq!(rewards.get("peer3"), Some(&reward3));
    }

    #[test]
    fn test_apply_penalty() {
        let poc = setup_test_poc();
        assert!(poc.update_reputation("peer1", true).is_ok());
        assert!(poc.update_stake("peer1", 1000, "ICN".to_string(), 30).is_ok());

        let initial_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let initial_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(poc.apply_penalty("peer1", 0.3).is_ok());
        let after_penalty_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let after_penalty_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(after_penalty_reputation < initial_reputation);
        assert_eq!(after_penalty_stake, initial_stake);

        assert!(poc.apply_penalty("peer1", 0.8).is_ok());
        let final_reputation = poc.reputation_scores.read().unwrap().get("peer1").cloned().unwrap();
        let final_stake = poc.stake_info.read().unwrap().get("peer1").unwrap().amount;

        assert!(final_reputation < after_penalty_reputation);
        assert!(final_stake < after_penalty_stake);
    }

    #[test]
    fn test_network_health_evaluation() {
        let poc = setup_test_poc();
        for (i, peer) in ["peer1", "peer2", "peer3"].iter().enumerate() {
            assert!(poc.update_stake(peer, 1000 + i as u64 * 500, "ICN".to_string(), 30).is_ok());
            assert!(poc.update_reputation(peer, true).is_ok());
        }

        let health_score = poc.evaluate_network_health().unwrap();
        assert!(health_score > 0.8);

        assert!(poc.update_reputation("peer1", false).is_ok());
        let new_health_score = poc.evaluate_network_health().unwrap();
        assert!(new_health_score < health_score);
    }
}
