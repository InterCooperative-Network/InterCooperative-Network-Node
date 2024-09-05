// File: icn_shared/src/error.rs

//! This module defines the errors that can occur across various components of the ICN project.
//! These errors are shared across different crates to maintain consistency in error handling.

use thiserror::Error;
use serde_json;

/// The `IcnError` enum represents errors that can occur across different components of the ICN project.
#[derive(Debug, Error)]
pub enum IcnError {
    /// An error that occurs during serialization or deserialization.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// An error that occurs during storage operations.
    #[error("Storage error: {0}")]
    Storage(String),

    /// An error related to consensus mechanisms.
    #[error("Consensus error: {0}")]
    Consensus(String),

    /// An error related to network operations.
    #[error("Network error: {0}")]
    Network(String),

    /// An error related to transaction operations.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// An error related to blockchain operations.
    #[error("Blockchain error: {0}")]
    Blockchain(String),

    /// An error related to virtual machine operations.
    #[error("Virtual Machine error: {0}")]
    VirtualMachine(String),

    /// A generic error for any other type of failure.
    #[error("Generic error: {0}")]
    Generic(String),
}

/// A specialized Result type for the shared module.
pub type IcnResult<T> = Result<T, IcnError>;

// Implement conversion from serde_json errors to IcnError for serialization issues.
impl From<serde_json::Error> for IcnError {
    fn from(err: serde_json::Error) -> Self {
        IcnError::Serialization(err.to_string())
    }
}

// You can add more conversion implementations here if needed for other error types.