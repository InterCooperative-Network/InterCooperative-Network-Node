// icn_shared/src/error.rs

// File: shared/src/errors.rs

//! This module defines the errors that can occur across various components of the ICN project.
//! These errors are shared across different crates to maintain consistency in error handling.

use thiserror::Error;

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

    /// A generic error for any other type of failure.a
    #[error("Generic error: {0}")]
    Generic(String),
}

/// A specialized Result type for the shared module.
pub type IcnResult<T> = Result<T, IcnError>;
