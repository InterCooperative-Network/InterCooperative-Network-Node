// icn_core/src/errors.rs

use std::fmt;

/// `IcnError` defines custom errors for the ICN project.
/// This is used throughout the project for consistent error handling.
#[derive(Debug)]
pub enum IcnError {
    ConfigError(config::ConfigError),
    IoError(std::io::Error),
    TlsError(native_tls::Error),
    Other(String),
}

impl fmt::Display for IcnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IcnError::ConfigError(ref e) => write!(f, "Configuration error: {}", e),
            IcnError::IoError(ref e) => write!(f, "I/O error: {}", e),
            IcnError::TlsError(ref e) => write!(f, "TLS error: {}", e),
            IcnError::Other(ref s) => write!(f, "{}", s),
        }
    }
}

// Implement `From` trait to allow automatic conversion from other error types to `IcnError`.
impl From<config::ConfigError> for IcnError {
    fn from(err: config::ConfigError) -> IcnError {
        IcnError::ConfigError(err)
    }
}

impl From<std::io::Error> for IcnError {
    fn from(err: std::io::Error) -> IcnError {
        IcnError::IoError(err)
    }
}

impl From<native_tls::Error> for IcnError {
    fn from(err: native_tls::Error) -> IcnError {
        IcnError::TlsError(err)
    }
}

/// `IcnResult` is a custom result type that wraps around `Result` using the `IcnError`.
pub type IcnResult<T> = Result<T, IcnError>;
