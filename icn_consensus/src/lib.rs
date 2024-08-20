// icn_consensus/src/lib.rs

pub mod proof_of_cooperation;

/// `IcnError` defines custom errors for the consensus module.
/// This is used throughout the crate for consistent error handling.
#[derive(Debug)]
pub enum IcnError {
    ConfigError(config::ConfigError),
    IoError(std::io::Error),
    TlsError(native_tls::Error),
    Other(String),
}

/// `IcnResult` is a custom result type that wraps around `Result` using the `IcnError`.
pub type IcnResult<T> = Result<T, IcnError>;

pub use proof_of_cooperation::ProofOfCooperation;
