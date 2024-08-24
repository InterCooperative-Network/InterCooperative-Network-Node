// File: icn_core/src/coordinator/mod.rs

//! This is the module entry point for the coordinator module.
//! It re-exports the `ModuleCoordinator`, `CoordinatorError`, and `CoordinatorResult`.

pub mod module_coordinator;

pub use self::module_coordinator::{ModuleCoordinator, CoordinatorError, CoordinatorResult};
