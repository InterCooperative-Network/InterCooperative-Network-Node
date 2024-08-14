// File: crates/icn_common/src/lib.rs

pub mod error;
pub mod bit_utils;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::collections::HashMap;

pub use crate::error::{IcnError, IcnResult};

/// Configuration for the InterCooperative Network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shard_count: u64,
    pub consensus_threshold: f64,
    pub consensus_quorum: f64,
    pub network_port: u16,
}

/// Represents a transaction in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: i64,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    /// Creates a new transaction
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, timestamp: i64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp,
            signature: None,
        }
    }

    /// Signs the transaction with the given keypair
    pub fn sign(&mut self, keypair: &Keypair) -> IcnResult<()> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message).to_bytes().to_vec();
        self.signature = Some(signature);
        Ok(())
    }

    /// Verifies the transaction signature
    pub fn verify(&self) -> IcnResult<bool> {
        let signature = self.signature.as_ref().ok_or(IcnError::Identity("Missing signature".into()))?;
        let public_key = PublicKey::from_bytes(&hex::decode(&self.from)?)
            .map_err(|e| IcnError::Identity(format!("Invalid public key: {}", e)))?;
        let message = self.to_bytes();
        let signature = Signature::from_bytes(signature)
            .map_err(|e| IcnError::Identity(format!("Invalid signature: {}", e)))?;
        Ok(public_key.verify(&message, &signature).is_ok())
    }

    /// Converts the transaction to bytes for signing
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.from.as_bytes());
        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }

    /// Calculates the transaction fee
    pub fn get_fee(&self) -> f64 {
        // Implement a more sophisticated fee calculation here
        0.001 * self.amount
    }
}

/// Represents a proposal in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
}

/// Represents a vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: i64,
    pub zkp: Option<Vec<u8>>,
}

/// Status of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

/// Type of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

/// Category of a proposal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Economic,
    Technical,
    Social,
}

/// Types of currency in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Custom(String),
}

/// Statistics about the network
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub node_count: usize,
    pub total_transactions: usize,
    pub active_proposals: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_signing_and_verification() {
        let mut rng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut rng);
        let public_key = keypair.public;

        let mut tx = Transaction::new(
            hex::encode(public_key.as_bytes()),
            "recipient".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            Utc::now().timestamp(),
        );

        tx.sign(&keypair).expect("Failed to sign transaction");
        assert!(tx.verify().expect("Failed to verify transaction"));

        // Test invalid signature
        tx.amount = 200.0;
        assert!(!tx.verify().expect("Verification should fail for tampered transaction"));
    }

    #[test]
    fn test_currency_type_equality() {
        assert_eq!(CurrencyType::BasicNeeds, CurrencyType::BasicNeeds);
        assert_ne!(CurrencyType::BasicNeeds, CurrencyType::Education);
        assert_eq!(CurrencyType::Custom("Test".to_string()), CurrencyType::Custom("Test".to_string()));
        assert_ne!(CurrencyType::Custom("Test1".to_string()), CurrencyType::Custom("Test2".to_string()));
    }

    #[test]
    fn test_proposal_lifecycle() {
        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "test_proposer".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + chrono::Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            execution_timestamp: None,
        };

        assert_eq!(proposal.status, ProposalStatus::Active);
        
        // Simulate proposal passing
        let passed_proposal = Proposal {
            status: ProposalStatus::Passed,
            ..proposal
        };
        assert_eq!(passed_proposal.status, ProposalStatus::Passed);

        // Simulate proposal execution
        let executed_proposal = Proposal {
            status: ProposalStatus::Executed,
            execution_timestamp: Some(Utc::now()),
            ..passed_proposal
        };
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
        assert!(executed_proposal.execution_timestamp.is_some());
    }

    #[test]
    fn test_network_stats() {
        let stats = NetworkStats {
            node_count: 5,
            total_transactions: 100,
            active_proposals: 3,
        };
        assert_eq!(stats.node_count, 5);
        assert_eq!(stats.total_transactions, 100);
        assert_eq!(stats.active_proposals, 3);
    }
}

// Additional utility functions

/// Generates a new keypair for use in the network
pub fn generate_keypair() -> Keypair {
    let mut csprng = OsRng{};
    Keypair::generate(&mut csprng)
}

/// Converts a public key to its string representation
pub fn public_key_to_string(public_key: &PublicKey) -> String {
    hex::encode(public_key.as_bytes())
}

/// Converts a string representation back to a public key
pub fn string_to_public_key(s: &str) -> IcnResult<PublicKey> {
    let bytes = hex::decode(s).map_err(|e| IcnError::Identity(format!("Invalid public key string: {}", e)))?;
    PublicKey::from_bytes(&bytes).map_err(|e| IcnError::Identity(format!("Invalid public key bytes: {}", e)))
}

/// Validates a proposal's parameters
pub fn validate_proposal(proposal: &Proposal) -> IcnResult<()> {
    if proposal.title.is_empty() {
        return Err(IcnError::Governance("Proposal title cannot be empty".into()));
    }
    if proposal.description.is_empty() {
        return Err(IcnError::Governance("Proposal description cannot be empty".into()));
    }
    if proposal.required_quorum <= 0.0 || proposal.required_quorum > 1.0 {
        return Err(IcnError::Governance("Invalid required quorum".into()));
    }
    if proposal.voting_ends_at <= proposal.created_at {
        return Err(IcnError::Governance("Voting end time must be after creation time".into()));
    }
    Ok(())
}