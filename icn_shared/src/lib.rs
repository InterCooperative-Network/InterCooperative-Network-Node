// File: icn_shared/src/lib.rs

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Custom error type for the ICN project.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IcnError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Smart contract error: {0}")]
    SmartContract(String),
    #[error("Virtual Machine error: {0}")]
    VirtualMachine(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for IcnError {
    fn from(err: serde_json::Error) -> Self {
        IcnError::Serialization(err.to_string())
    }
}

/// Result type alias for the ICN project.
pub type IcnResult<T> = Result<T, IcnError>;

/// Represents a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,
    pub previous_hash: String,
    pub hash: String,
    pub proposer_id: String,
    pub nonce: u64,
}

impl Block {
    /// Creates a new `Block` instance.
    pub fn new(index: u64, transactions: Vec<String>, previous_hash: String, proposer_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            proposer_id,
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of the block.
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(serde_json::to_string(&self.transactions).unwrap());
        hasher.update(&self.previous_hash);
        hasher.update(&self.proposer_id);
        hasher.update(self.nonce.to_be_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verifies the block's integrity by checking its hash.
    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}

/// Defines the possible states of a node in the ICN network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Initializing,
    Operational,
    ShuttingDown,
    Configuring,
}

/// Utility functions for the ICN project.
pub mod utils {
    /// Checks if a given string is a valid hexadecimal number.
    pub fn is_valid_hex(hex_string: &str) -> bool {
        hex_string.chars().all(|c| c.is_digit(16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_hex() {
        assert!(utils::is_valid_hex("1a2b3c"));
        assert!(utils::is_valid_hex("ABCDEF"));
        assert!(!utils::is_valid_hex("1a2g3c"));
        assert!(!utils::is_valid_hex("xyz"));
    }

    #[test]
    fn test_block_creation() {
        let block = Block::new(0, vec!["tx1".into()], "prev_hash".to_string(), "proposer1".to_string());
        assert_eq!(block.index, 0);
        assert_eq!(block.transactions, vec!["tx1".to_string()]);
        assert_eq!(block.previous_hash, "prev_hash");
        assert_eq!(block.proposer_id, "proposer1");
        assert_eq!(block.nonce, 0);
        assert!(!block.hash.is_empty());
        assert!(block.is_valid());
    }

    #[test]
    fn test_block_validity() {
        let mut block = Block::new(1, vec!["tx2".into()], "prev_hash".to_string(), "proposer2".to_string());
        assert!(block.is_valid());

        // Tamper with the block
        block.transactions.push("tx3".into());
        assert!(!block.is_valid());

        // Recalculate the hash
        block.hash = block.calculate_hash();
        assert!(block.is_valid());
    }

    #[test]
    fn test_block_hash_changes_with_nonce() {
        let mut block = Block::new(2, vec!["tx4".into()], "prev_hash".to_string(), "proposer3".to_string());
        let original_hash = block.hash.clone();

        block.nonce += 1;
        block.hash = block.calculate_hash();

        assert_ne!(original_hash, block.hash);
        assert!(block.is_valid());
    }

    #[test]
    fn test_icn_error_display() {
        let error = IcnError::Blockchain("Invalid block".to_string());
        assert_eq!(error.to_string(), "Blockchain error: Invalid block");

        let error = IcnError::Network("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");
    }
}