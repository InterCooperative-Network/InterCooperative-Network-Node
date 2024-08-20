// File: icn_shared/src/lib.rs

/// The `icn_shared` crate provides common types and utilities used across the InterCooperative Network project.
///
/// This crate includes error types, result aliases, and shared data structures that are used by other crates
/// in the project to ensure consistency and reduce code duplication.

use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};

/// Custom error type for the ICN project.
///
/// This enum represents various error cases that can occur within the InterCooperative Network system.
/// It is used across different crates to provide a consistent error handling mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IcnError {
    /// Configuration-related errors.
    Config(String),
    /// Blockchain-related errors.
    Blockchain(String),
    /// Consensus mechanism errors.
    Consensus(String),
    /// Network-related errors.
    Network(String),
    /// Smart contract errors.
    SmartContract(String),
    /// Storage-related errors.
    Storage(String),
    /// I/O errors.
    Io(String),
    /// Any other errors that don't fit into the above categories.
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

/// A type alias for Result with IcnError as the error type.
///
/// This alias is used throughout the project to provide a consistent result type
/// that uses our custom `IcnError`.
pub type IcnResult<T> = Result<T, IcnError>;

/// Represents the current state of a node in the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// The node is in the process of starting up and initializing its components.
    Initializing,
    /// The node is fully operational and participating in the network.
    Operational,
    /// The node is in the process of shutting down.
    ShuttingDown,
}

/// This module contains utility functions used across the project.
pub mod utils {
    use super::*;

    /// Validates if a given string is a valid hexadecimal representation.
    ///
    /// # Arguments
    ///
    /// * `hex_string` - A string slice that should contain a hexadecimal value.
    ///
    /// # Returns
    ///
    /// * `true` if the string is a valid hexadecimal representation, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use icn_shared::utils::is_valid_hex;
    ///
    /// assert!(is_valid_hex("1a2b3c"));
    /// assert!(!is_valid_hex("1a2g3c")); // 'g' is not a valid hex character
    /// ```
    pub fn is_valid_hex(hex_string: &str) -> bool {
        hex_string.chars().all(|c| c.is_digit(16))
    }

    // Add more utility functions as needed
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
}