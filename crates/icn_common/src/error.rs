// File: crates/icn_common/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IcnError {
    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Currency error: {0}")]
    Currency(String),

    #[error("Governance error: {0}")]
    Governance(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Node management error: {0}")]
    NodeManagement(String),

    #[error("Sharding error: {0}")]
    Sharding(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("VM error: {0}")]
    Vm(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Zero-Knowledge Proof error: {0}")]
    Zkp(String),

    #[error("Smart Contract error: {0}")]
    SmartContract(String),

    #[error("Cross-shard communication error: {0}")]
    CrossShardCommunication(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type IcnResult<T> = std::result::Result<T, IcnError>;

/// Helper function to convert any error type to IcnError
pub fn to_icn_error<E: std::error::Error>(error: E) -> IcnError {
    IcnError::Unknown(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_error_display() {
        let error = IcnError::Blockchain("Invalid block".to_string());
        assert_eq!(error.to_string(), "Blockchain error: Invalid block");
    }

    #[test]
    fn test_icn_result() {
        fn may_fail() -> IcnResult<i32> {
            Err(IcnError::Currency("Insufficient funds".to_string()))
        }

        let result = may_fail();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Currency error: Insufficient funds");
    }

    #[test]
    fn test_to_icn_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let icn_error = to_icn_error(io_error);
        assert_eq!(icn_error.to_string(), "Unknown error: File not found");
    }
}