// /opt/InterCooperative-Network-Node/icn_core/src/lib.rs

pub mod config;
pub mod coordinator;
pub mod errors;

pub use config::ConfigLoader;
pub use coordinator::ModuleCoordinator;
pub use errors::IcnError;
