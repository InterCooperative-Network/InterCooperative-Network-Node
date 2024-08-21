use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH}; // Import the missing time components
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest}; // Import the missing sha2 crate

/// Custom error type for the ICN project.
///
/// This enum represents various error cases that can occur within the InterCooperative Network system.
/// It is used across different crates to provide a consistent error handling mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IcnError {
    Config(String),
    Blockchain(String),
    Consensus(String),
    Network(String),
    SmartContract(String),
    Storage(String),
    Io(String),
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

/// Converts a standard I/O error into an `IcnError`.
impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err.to_string())
    }
}

/// A type alias for `Result` with `IcnError` as the error type.
///
/// This alias is used throughout the project to provide a consistent result type
/// that uses our custom `IcnError`.
pub type IcnResult<T> = Result<T, IcnError>;

/// Represents the current state of a node in the network.
///
/// This enum is used to track the lifecycle of nodes within the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Initializing,
    Operational,
    ShuttingDown,
}

/// Common block structure that can be used across crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,
    pub previous_hash: String,
    pub hash: String,
    pub proposer_id: String,
}

impl Block {
    /// Creates a new block
    pub fn new(index: u64, transactions: Vec<String>, previous_hash: String, proposer_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        // Simulating hash calculation
        let hash = format!("{:x}", Sha256::digest(format!("{:?}", index).as_bytes()));

        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            proposer_id,
        }
    }
}

/// This module contains utility functions used across the project.
pub mod utils {
    /// Validates if a given string is a valid hexadecimal representation.
    pub fn is_valid_hex(hex_string: &str) -> bool {
        hex_string.chars().all(|c| c.is_digit(16))
    }

    // Additional utility functions can be added here as needed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for the `is_valid_hex` utility function.
    #[test]
    fn test_is_valid_hex() {
        assert!(utils::is_valid_hex("1a2b3c"));
        assert!(utils::is_valid_hex("ABCDEF"));
        assert!(!utils::is_valid_hex("1a2g3c"));
        assert!(!utils::is_valid_hex("xyz"));
    }
}
