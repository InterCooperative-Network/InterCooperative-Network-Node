// File: icn_core/src/coordinator/module_coordinator.rs

//! This module defines the `ModuleCoordinator` responsible for managing
//! and coordinating the different modules of the InterCooperative Network (ICN).
//! The coordinator handles initialization, starting, and stopping of all modules.

use crate::errors::{IcnError, IcnResult};

/// Define a custom error type for the coordinator module.
#[derive(Debug)]
pub enum CoordinatorError {
    InitializationError(String),
    StartError(String),
    StopError(String),
}

impl std::fmt::Display for CoordinatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinatorError::InitializationError(msg) => write!(f, "Initialization Error: {}", msg),
            CoordinatorError::StartError(msg) => write!(f, "Start Error: {}", msg),
            CoordinatorError::StopError(msg) => write!(f, "Stop Error: {}", msg),
        }
    }
}

impl std::error::Error for CoordinatorError {}

/// Custom result type for the coordinator module.
pub type CoordinatorResult<T> = Result<T, CoordinatorError>;

/// The `ModuleCoordinator` struct is responsible for managing and coordinating
/// the various modules that make up the ICN node. It ensures that all modules
/// are initialized, started, and stopped in the correct order.
pub struct ModuleCoordinator {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleCoordinator {
    /// Creates a new instance of `ModuleCoordinator`.
    pub fn new() -> Self {
        ModuleCoordinator {
            modules: Vec::new(),
        }
    }

    /// Registers a new module with the coordinator.
    ///
    /// # Arguments
    ///
    /// * `module` - The module to be registered.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if the module is successfully registered, or an error otherwise.
    pub fn register_module(&mut self, module: Box<dyn Module>) -> CoordinatorResult<()> {
        self.modules.push(module);
        Ok(())
    }

    /// Initializes all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully initialized, or an error otherwise.
    pub fn initialize(&mut self) -> CoordinatorResult<()> {
        for module in &mut self.modules {
            module.initialize()?;
        }
        Ok(())
    }

    /// Starts all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully started, or an error otherwise.
    pub fn start(&mut self) -> CoordinatorResult<()> {
        for module in &mut self.modules {
            module.start()?;
        }
        Ok(())
    }

    /// Stops all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully stopped, or an error otherwise.
    pub fn stop(&mut self) -> CoordinatorResult<()> {
        for module in &mut self.modules {
            module.stop()?;
        }
        Ok(())
    }
}

/// The `Module` trait defines the interface for modules that can be managed by the `ModuleCoordinator`.
/// Each module must implement methods for initialization, starting, and stopping.
pub trait Module {
    /// Initializes the module.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if the module is successfully initialized, or an error otherwise.
    fn initialize(&mut self) -> CoordinatorResult<()>;

    /// Starts the module.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if the module is successfully started, or an error otherwise.
    fn start(&mut self) -> CoordinatorResult<()>;

    /// Stops the module.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if the module is successfully stopped, or an error otherwise.
    fn stop(&mut self) -> CoordinatorResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestModule {
        initialized: bool,
        started: bool,
    }

    impl Module for TestModule {
        fn initialize(&mut self) -> CoordinatorResult<()> {
            self.initialized = true;
            Ok(())
        }

        fn start(&mut self) -> CoordinatorResult<()> {
            if !self.initialized {
                return Err(CoordinatorError::InitializationError(
                    "Module not initialized".to_string(),
                ));
            }
            self.started = true;
            Ok(())
        }

        fn stop(&mut self) -> CoordinatorResult<()> {
            self.started = false;
            Ok(())
        }
    }

    #[test]
    fn test_module_coordinator() {
        let mut coordinator = ModuleCoordinator::new();
        let module = Box::new(TestModule {
            initialized: false,
            started: false,
        });

        assert!(coordinator.register_module(module).is_ok());
        assert!(coordinator.initialize().is_ok());
        assert!(coordinator.start().is_ok());
        assert!(coordinator.stop().is_ok());
    }
}
