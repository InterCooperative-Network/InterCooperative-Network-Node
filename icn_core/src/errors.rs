use std::fmt;
use std::error::Error;

/// Custom error type for the ICN project
#[derive(Debug)]
pub enum IcnError {
    /// Configuration error
    Config(String),

    /// IO error
    Io(std::io::Error),

    /// TOML parsing error
    Toml(toml::de::Error),

    /// Blockchain error
    Blockchain(String),

    /// Consensus error
    Consensus(String),

    /// Networking error
    Networking(String),

    /// Smart Contract error
    SmartContract(String),

    /// Virtual Machine error
    VirtualMachine(String),

    /// Storage error
    Storage(String),

    /// Other errors
    Other(String),
}

impl fmt::Display for IcnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcnError::Config(msg) => write!(f, "Configuration error: {}", msg),
            IcnError::Io(err) => write!(f, "I/O error: {}", err),
            IcnError::Toml(err) => write!(f, "TOML parsing error: {}", err),
            IcnError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            IcnError::Consensus(msg) => write!(f, "Consensus error: {}", msg),
            IcnError::Networking(msg) => write!(f, "Networking error: {}", msg),
            IcnError::SmartContract(msg) => write!(f, "Smart Contract error: {}", msg),
            IcnError::VirtualMachine(msg) => write!(f, "Virtual Machine error: {}", msg),
            IcnError::Storage(msg) => write!(f, "Storage error: {}", msg),
            IcnError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for IcnError {}

impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err)
    }
}

impl From<toml::de::Error> for IcnError {
    fn from(err: toml::de::Error) -> Self {
        IcnError::Toml(err)
    }
}
