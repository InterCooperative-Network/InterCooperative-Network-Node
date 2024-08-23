// File: icn_core/src/lib.rs

pub mod config;
pub mod coordinator;
pub mod errors;

pub use config::ConfigLoader;
pub use coordinator::module_coordinator::ModuleCoordinator;
pub use errors::IcnError;
