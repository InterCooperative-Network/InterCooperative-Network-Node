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

// Implement conversion from std::io::Error for I/O-related issues.
impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Network(err.to_string())
    }
}

// Implement conversion from a generic String for other errors.
impl From<String> for IcnError {
    fn from(err: String) -> Self {
        IcnError::Generic(err)
    }
}

// Implement conversion for any other types of errors you might want to handle later.
// Additional conversions can be implemented here as needed.

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization_error() {
        let invalid_json = "{ invalid_json: true }";
        let result: Result<serde_json::Value, IcnError> = serde_json::from_str(invalid_json).map_err(IcnError::from);
        assert!(matches!(result, Err(IcnError::Serialization(_))));
    }

    #[test]
    fn test_storage_error() {
        let error = IcnError::Storage("Failed to access storage".to_string());
        assert_eq!(format!("{}", error), "Storage error: Failed to access storage");
    }

    #[test]
    fn test_consensus_error() {
        let error = IcnError::Consensus("Failed consensus check".to_string());
        assert_eq!(format!("{}", error), "Consensus error: Failed consensus check");
    }

    #[test]
    fn test_network_error() {
        let error = IcnError::Network("Connection failed".to_string());
        assert_eq!(format!("{}", error), "Network error: Connection failed");
    }

    #[test]
    fn test_transaction_error() {
        let error = IcnError::Transaction("Invalid transaction".to_string());
        assert_eq!(format!("{}", error), "Transaction error: Invalid transaction");
    }

    #[test]
    fn test_blockchain_error() {
        let error = IcnError::Blockchain("Invalid block".to_string());
        assert_eq!(format!("{}", error), "Blockchain error: Invalid block");
    }

    #[test]
    fn test_virtual_machine_error() {
        let error = IcnError::VirtualMachine("VM execution failed".to_string());
        assert_eq!(format!("{}", error), "Virtual Machine error: VM execution failed");
    }

    #[test]
    fn test_generic_error() {
        let error = IcnError::Generic("Some generic error".to_string());
        assert_eq!(format!("{}", error), "Generic error: Some generic error");
    }
}
