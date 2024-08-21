use icn_shared::{IcnError, IcnResult, Block}; // Import `Block` from `icn_shared`
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

/// The `Consensus` trait defines the interface for consensus mechanisms
/// within the InterCooperative Network blockchain system.
pub trait Consensus {
    /// Validates a block according to the consensus rules.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to the block that needs to be validated.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, or an `IcnError` if validation fails.
    fn validate(&self, block: &Block) -> IcnResult<bool>;

    /// Selects a proposer for the next block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - Returns the ID of the selected proposer, or an `IcnError` if selection fails.
    fn select_proposer(&self) -> IcnResult<String>;
}

/// The `ProofOfCooperation` struct implements the consensus mechanism for the ICN project.
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, u64>,
    last_block_time: u64,
}

impl ProofOfCooperation {
    /// Creates a new instance of the `ProofOfCooperation` consensus mechanism.
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            last_block_time: 0,
        }
    }

    /// Registers a new peer in the consensus system.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer to be registered.
    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 100);
        info!("Registered peer: {}", peer_id);
    }

    /// Checks whether a peer is already registered in the system.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - The unique identifier of the peer to be checked.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the peer is registered, `false` otherwise.
    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }
}

impl Consensus for ProofOfCooperation {
    /// Validates a block according to the consensus rules.
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

        // Additional validation logic can be implemented here

        Ok(true)
    }

    /// Selects a proposer for the next block based on cooperation scores.
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