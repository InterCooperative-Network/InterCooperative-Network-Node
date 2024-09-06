// File: icn_shared/src/lib.rs

//! This module defines the core structures and error handling for the InterCooperative Network (ICN) project.
//! It includes custom error types, the `Block` struct representing a blockchain block, and utility functions.

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Custom error type for the ICN project.
///
/// This enum encapsulates various types of errors that can occur within the ICN project, including
/// configuration, blockchain, consensus, network, smart contract, storage, I/O, and other errors.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum IcnError {
    /// Configuration-related errors.
    #[error("Configuration error: {0}")]
    Config(String),
    /// Blockchain-related errors.
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    /// Consensus-related errors.
    #[error("Consensus error: {0}")]
    Consensus(String),
    /// Network-related errors.
    #[error("Network error: {0}")]
    Network(String),
    /// Smart contract-related errors.
    #[error("Smart contract error: {0}")]
    SmartContract(String),
    /// Storage-related errors.
    #[error("Storage error: {0}")]
    Storage(String),
    /// I/O-related errors.
    #[error("I/O error: {0}")]
    Io(String),
    /// Virtual Machine-related errors (add this variant).
    #[error("Virtual Machine error: {0}")]
    VirtualMachine(String),
    /// Other miscellaneous errors.
    #[error("Other error: {0}")]
    Other(String),
}


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
    /// The position of the block within the blockchain.
    pub index: u64,
    /// The timestamp when the block was created.
    pub timestamp: u64,
    /// A vector of transactions included in the block.
    pub transactions: Vec<String>,
    /// The hash of the previous block in the chain.
    pub previous_hash: String,
    /// The hash of the current block.
    pub hash: String,
    /// The ID of the proposer who created the block.
    pub proposer_id: String,
    /// A nonce value used for mining/consensus purposes.
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
