// File: icn_shared/src/lib.rs

use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};

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

impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err.to_string())
    }
}

pub type IcnResult<T> = Result<T, IcnError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Initializing,
    Operational,
    ShuttingDown,
}

// Add any other shared types or constants here