// File: icn_core/src/errors.rs

use std::fmt;
use std::error::Error;

/// Custom error type for the ICN project, used across core functionalities.
#[derive(Debug)]
pub enum IcnError {
    /// Configuration error, typically occurs during loading or parsing configuration files.
    Config(String),

    /// IO error, wraps standard I/O errors.
    Io(std::io::Error),

    /// TOML parsing error, occurs during the parsing of TOML configuration files.
    Toml(toml::de::Error),

    /// Blockchain error, used for errors related to blockchain operations.
    Blockchain(String),

    /// Consensus error, used for errors related to consensus algorithms and operations.
    Consensus(String),

    /// Networking error, used for errors during networking operations.
    Networking(String),

    /// Smart Contract error, used for errors related to smart contract execution or validation.
    SmartContract(String),

    /// Virtual Machine error, used for errors within the virtual machine environment.
    VirtualMachine(String),

    /// Storage error, used for errors related to data storage operations.
    Storage(String),

    /// Other errors, used for any miscellaneous errors that don't fit into the above categories.
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

// Conversion from standard IO errors to IcnError.
impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> Self {
        IcnError::Io(err)
    }
}

// Conversion from TOML parsing errors to IcnError.
impl From<toml::de::Error> for IcnError {
    fn from(err: toml::de::Error) -> Self {
        IcnError::Toml(err)
    }
}
