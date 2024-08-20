// icn_core/src/errors.rs

use std::error::Error;
use std::fmt;
use thiserror::Error;

/// Custom error type for the ICN project
#[derive(Error, Debug)]
pub enum IcnError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Blockchain error
    #[error("Blockchain error: {0}")]
    Blockchain(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    Consensus(String),

    /// Networking error
    #[error("Networking error: {0}")]
    Networking(String),

    /// Smart Contract error
    #[error("Smart Contract error: {0}")]
    SmartContract(String),

    /// Virtual Machine error
    #[error("Virtual Machine error: {0}")]
    VirtualMachine(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

// Implementing the `Error` trait allows our custom error to be used in the `?` operator
impl Error for IcnError {}

// Implementing the `Display` trait allows our custom error to be formatted and printed
impl fmt::Display for IcnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // Use the `Error` trait's implementation for formatting
    }
}