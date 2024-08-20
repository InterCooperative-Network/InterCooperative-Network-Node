// icn_shared/src/lib.rs

use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum IcnError {
    Io(std::io::Error),
    Tls(native_tls::Error),
    Config(String),
    Consensus(String),
    Network(String),
    SmartContract(String),
    Database(String),
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    Other(String),
}

impl fmt::Display for IcnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IcnError::Io(e) => write!(f, "I/O error: {}", e),
            IcnError::Tls(e) => write!(f, "TLS error: {}", e),
            IcnError::Config(e) => write!(f, "Configuration error: {}", e),
            IcnError::Consensus(e) => write!(f, "Consensus error: {}", e),
            IcnError::Network(e) => write!(f, "Network error: {}", e),
            IcnError::SmartContract(e) => write!(f, "Smart contract error: {}", e),
            IcnError::Database(e) => write!(f, "Database error: {}", e),
            IcnError::Validation(e) => write!(f, "Validation error: {}", e),
            IcnError::NotFound(e) => write!(f, "Not found: {}", e),
            IcnError::Unauthorized(e) => write!(f, "Unauthorized: {}", e),
            IcnError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl Error for IcnError {}

impl From<std::io::Error> for IcnError {
    fn from(error: std::io::Error) -> Self {
        IcnError::Io(error)
    }
}

impl From<native_tls::Error> for IcnError {
    fn from(error: native_tls::Error) -> Self {
        IcnError::Tls(error)
    }
}

pub type IcnResult<T> = Result<T, IcnError>;