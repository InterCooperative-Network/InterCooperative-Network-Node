// file: icn_consensus/src/lib.rs

use icn_shared::{Block, IcnError, IcnResult};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

pub trait Consensus: Clone + Send + Sync {
    fn validate(&self, block: &Block) -> IcnResult<bool>;
    fn select_proposer(&self) -> IcnResult<String>;
    fn get_eligible_peers(&self) -> Vec<String>;
}

#[derive(Clone, Debug)]
pub struct ProofOfCooperation {
    known_peers: HashSet<String>,
    cooperation_scores: HashMap<String, f64>,
    reputation_scores: HashMap<String, f64>,
    contribution_history: HashMap<String, Vec<(u64, f64)>>, 
    last_block_time: u64,
}

impl ProofOfCooperation {
    pub fn new() -> Self {
        ProofOfCooperation {
            known_peers: HashSet::new(),
            cooperation_scores: HashMap::new(),
            reputation_scores: HashMap::new(),
            contribution_history: HashMap::new(),
            last_block_time: 0,
        }
    }

    pub fn register_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
        self.cooperation_scores.insert(peer_id.to_string(), 1.0);
        self.reputation_scores.insert(peer_id.to_string(), 1.0);  
        self.contribution_history.insert(peer_id.to_string(), Vec::new());
        info!("Registered peer: {}", peer_id);
    }

    pub fn is_registered(&self, peer_id: &str) -> bool {
        self.known_peers.contains(peer_id)
    }

    pub fn update_cooperation_score(&mut self, peer_id: &str, performance: f64) -> IcnResult<()> {
        let score = self.cooperation_scores
            .get_mut(peer_id)
            .ok_or_else(|| IcnError::Consensus(format!("Unknown peer: {}", peer_id)))?;
        
        *score = (*score * performance).max(0.1).min(2.0);  
        self.update_reputation(peer_id)?;  
        self.record_contribution(peer_id, *score)?;
        Ok(())
    }

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

    fn get_eligible_peers(&self) -> Vec<String> {
        self.known_peers.iter().cloned().collect()
    }
}

