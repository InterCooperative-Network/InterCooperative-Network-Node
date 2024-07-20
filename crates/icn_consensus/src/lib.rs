use icn_types::{IcnResult, IcnError, Block, Transaction, Node};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: String,
    pub reputation: f64,
    pub is_validator: bool,
}

pub struct PoCConsensus {
    pub members: HashMap<String, Member>,
    pub threshold: f64,
    pub quorum: f64,
}

impl PoCConsensus {
    pub fn new(threshold: f64, quorum: f64) -> Self {
        PoCConsensus {
            members: HashMap::new(),
            threshold,
            quorum,
        }
    }

    pub fn add_member(&mut self, id: String, is_validator: bool) -> IcnResult<()> {
        if self.members.contains_key(&id) {
            return Err(IcnError::Consensus("Member already exists".to_string()));
        }
        self.members.insert(id.clone(), Member {
            id,
            reputation: 1.0,
            is_validator,
        });
        Ok(())
    }

    pub fn remove_member(&mut self, id: &str) -> IcnResult<()> {
        if self.members.remove(id).is_none() {
            return Err(IcnError::Consensus("Member not found".to_string()));
        }
        Ok(())
    }

    pub fn update_reputation(&mut self, id: &str, change: f64) -> IcnResult<()> {
        let member = self.members.get_mut(id).ok_or_else(|| IcnError::Consensus("Member not found".to_string()))?;
        member.reputation += change;
        member.reputation = member.reputation.max(0.0); // Ensure reputation doesn't go negative
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        // In a real implementation, this would involve more complex validation logic
        // For now, we'll just check if the block has any transactions
        if block.transactions.is_empty() {
            return Err(IcnError::Consensus("Block has no transactions".to_string()));
        }
        Ok(true)
    }

    pub fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool> {
        let total_reputation: f64 = self.members.values().filter(|m| m.is_validator).map(|m| m.reputation).sum();

        let mut positive_reputation = 0.0;
        let mut participating_reputation = 0.0;

        for (member_id, vote) in votes {
            if let Some(member) = self.members.get(*member_id) {
                if member.is_validator {
                    participating_reputation += member.reputation;
                    if *vote {
                        positive_reputation += member.reputation;
                    }
                }
            } else {
                return Err(IcnError::Consensus("Invalid member in votes".to_string()));
            }
        }

        if participating_reputation / total_reputation < self.quorum {
            return Err(IcnError::Consensus("Quorum not reached".to_string()));
        }

        Ok(positive_reputation / participating_reputation >= self.threshold)
    }

    pub fn get_validators(&self) -> Vec<&Member> {
        self.members.values().filter(|m| m.is_validator).collect()
    }
}

pub trait ConsensusAlgorithm {
    fn validate_block(&self, block: &Block) -> IcnResult<bool>;
    fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool>;
}

impl ConsensusAlgorithm for PoCConsensus {
    fn validate_block(&self, block: &Block) -> IcnResult<bool> {
        self.validate_block(block)
    }

    fn reach_consensus(&self, block: &Block, votes: &[(&str, bool)]) -> IcnResult<bool> {
        self.reach_consensus(block, votes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_types::CurrencyType;

    #[test]
    fn test_add_and_remove_member() {
        let mut consensus = PoCConsensus::new(0.66, 0.51);
        assert!(consensus.add_member("Alice".to_string(), true).is_ok());
        assert!(consensus.add_member("Bob".to_string(), false).is_ok());
        assert_eq!(consensus.members.len(), 2);
        assert!(consensus.remove_member("Alice").is_ok());
        assert_eq!(consensus.members.len(), 1);
        assert!(consensus.remove_member("Charlie").is_err());
    }

    #[test]
    fn test_update_reputation() {
        let mut consensus = PoCConsensus::new(0.66, 0.51);
        consensus.add_member("Alice".to_string(), true).unwrap();
        assert!(consensus.update_reputation("Alice", 0.5).is_ok());
        assert_eq!(consensus.members.get("Alice").unwrap().reputation, 1.5);
        assert!(consensus.update_reputation("Bob", 1.0).is_err());
    }

    #[test]
    fn test_reach_consensus() {
        let mut consensus = PoCConsensus::new(0.66, 0.51);
        consensus.add_member("Alice".to_string(), true).unwrap();
        consensus.add_member("Bob".to_string(), true).unwrap();
        consensus.add_member("Charlie".to_string(), true).unwrap();

        let block = Block {
            index: 1,
            timestamp: 0,
            transactions: vec![Transaction {
                from: "Alice".to_string(),
                to: "Bob".to_string(),
                amount: 100.0,
                currency_type: CurrencyType::BasicNeeds,
                timestamp: 0,
                signature: None,
            }],
            previous_hash: "previous_hash".to_string(),
            hash: "hash".to_string(),
        };

        let votes = vec![
            ("Alice", true),
            ("Bob", true),
            ("Charlie", false),
        ];

        assert!(consensus.reach_consensus(&block, &votes).unwrap());

        let insufficient_votes = vec![
            ("Alice", true),
            ("Bob", true),
        ];

        assert!(consensus.reach_consensus(&block, &insufficient_votes).is_err());
    }
}