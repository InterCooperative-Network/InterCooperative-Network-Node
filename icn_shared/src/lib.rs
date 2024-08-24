// File: icn_shared/src/lib.rs

//! This module defines the core structures and error handling for the InterCooperative Network (ICN) project.
//! It includes custom error types, the `Block` struct representing a blockchain block, and utility functions.

use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Custom error type for the ICN project.
///
/// This enum encapsulates various types of errors that can occur within the ICN project, including
/// configuration, blockchain, consensus, network, smart contract, storage, I/O, and other errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IcnError {
    /// Configuration-related errors.
    Config(String),
    /// Blockchain-related errors.
    Blockchain(String),
    /// Consensus-related errors.
    Consensus(String),
    /// Network-related errors.
    Network(String),
    /// Smart contract-related errors.
    SmartContract(String),
    /// Storage-related errors.
    Storage(String),
    /// I/O-related errors.
    Io(String),
    /// Other miscellaneous errors.
    Other(String),
}

impl fmt::Display for IcnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcnError::Config(msg) => write!(f, "Configuration error: {}", msg),
            IcnError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            IcnError::Consensus(msg) => write!(f, "Consensus error: {}", msg),
            IcnError::Network(msg) => write!(f, "Network error: {}", msg),
            IcnError::SmartContract(msg) => write!(f, "Smart contract error: {}", msg),
            IcnError::Storage(msg) => write!(f, "Storage error: {}", msg),
            IcnError::Io(msg) => write!(f, "I/O error: {}", msg),
            IcnError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for IcnError {}

impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err.to_string())
    }
}

impl From<String> for IcnError {
    fn from(err: String) -> Self {
        IcnError::Other(err)
    }
}

/// Result type alias for the ICN project.
pub type IcnResult<T> = Result<T, IcnError>;

/// Represents a block in the blockchain.
///
/// Each block contains an index, timestamp, a list of transactions, the hash of the previous block,
/// a hash of the current block, the ID of the proposer who created the block, and a nonce for mining.
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
    ///
    /// This constructor generates a new block with the provided index, transactions, previous hash, and proposer ID.
    /// The block's timestamp is set to the current system time, and the hash is calculated based on the block's data.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the block within the blockchain.
    /// * `transactions` - A vector of transactions to include in the block.
    /// * `previous_hash` - The hash of the previous block in the chain.
    /// * `proposer_id` - The ID of the proposer who created the block.
    ///
    /// # Returns
    ///
    /// A new instance of `Block`.
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
    ///
    /// This method computes the SHA-256 hash of the block's content, ensuring the block's integrity.
    ///
    /// # Returns
    ///
    /// A string representing the hexadecimal hash of the block.
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
    ///
    /// This method compares the stored hash with the calculated hash to ensure the block's data has not been tampered with.
    ///
    /// # Returns
    ///
    /// `true` if the block is valid, `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}

/// Defines the possible states of a node in the ICN network.
///
/// The `NodeState` enum represents different operational states of a node in the network, such as initialization,
/// normal operation, or shutdown. These states help in managing the lifecycle and operational flow of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// The node is in the initialization phase.
    Initializing,
    /// The node is fully operational.
    Operational,
    /// The node is in the process of shutting down.
    ShuttingDown,
    /// The node is configuring.
    Configuring, 
}

/// Utility functions for the ICN project.
pub mod utils {
    /// Checks if a given string is a valid hexadecimal number.
    ///
    /// This function ensures that the provided string contains only hexadecimal digits (0-9, a-f, A-F).
    ///
    /// # Arguments
    ///
    /// * `hex_string` - A string slice representing the potential hexadecimal number.
    ///
    /// # Returns
    ///
    /// `true` if the string is a valid hexadecimal number, `false` otherwise.
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