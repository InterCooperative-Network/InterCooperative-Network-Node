use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// The `Consensus` trait defines the interface for consensus mechanisms
/// within the InterCooperative Network blockchain system.
pub trait Consensus: Clone + Send + Sync {
    fn validate(&self, block: &Block) -> IcnResult<bool>;
    fn select_proposer(&self) -> IcnResult<String>;
}

/// The `ProofOfCooperation` struct implements the consensus mechanism for the ICN project.
#[derive(Clone, Debug)]  // Adding Debug trait for better error handling and logging
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, u64>,
    last_block_time: u64,
}

impl ProofOfCooperation {
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            last_block_time: 0,
        }
    }

    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 100);
        info!("Registered peer: {}", peer_id);
    }

    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }
}

impl Consensus for ProofOfCooperation {
    fn validate(&self, block: &Block) -> IcnResult<bool> {
        if !self.is_registered(&block.proposer_id) {
            return Err(IcnError::Consensus(format!("Unknown proposer: {}", block.proposer_id)));
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| IcnError::Consensus(format!("System time error: {}", e)))?
            .as_secs();

        if current_time < self.last_block_time + 10 {
            return Err(IcnError::Consensus("Block proposed too soon".to_string()));
        }

        Ok(true)
    }

    fn select_proposer(&self) -> IcnResult<String> {
        let mut rng = rand::thread_rng();
        let total_score: u64 = self.cooperation_scores.values().sum();
        let random_value: u64 = rng.gen_range(0..total_score);

        let mut cumulative_score = 0;
        for (peer_id, score) in &self.cooperation_scores {
            cumulative_score += score;
            if cumulative_score >= random_value {
                return Ok(peer_id.clone());
            }
        }

        Err(IcnError::Consensus("Failed to select a proposer".to_string()))
    }
}
