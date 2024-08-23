// File: icn_core/src/coordinator/module_coordinator.rs

//! This module defines the `ModuleCoordinator` responsible for managing
//! and coordinating the different modules of the InterCooperative Network (ICN).
//! The coordinator handles initialization, starting, and stopping of all modules.

use crate::coordinator::CoordinatorError;
use crate::coordinator::CoordinatorResult;

/// Defines errors that can occur within the module coordination process.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    /// An error that occurs during the initialization of a module.
    #[error("Initialization error: {0}")]
    InitializationError(String),
    /// An error that occurs during the operation of a module.
    #[error("Module operation error: {0}")]
    OperationError(String),
}

/// A specialized Result type for the Coordinator module.
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
    pub fn register_module(&mut self, module: Box<dyn Module>) -> CoordinatorResult<()> {
        self.modules.push(module);
        Ok(())
    }

    /// Initializes all registered modules.
    pub fn initialize(&mut self) -> CoordinatorResult<()> {
        for module in &mut self.modules {
            module.initialize()?;
        }
        Ok(())
    }

    /// Starts all registered modules.
    pub fn start(&mut self) -> CoordinatorResult<()> {
        for module in &mut self.modules {
            module.start()?;
        }
        Ok(())
    }

    /// Stops all registered modules.
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
    fn initialize(&mut self) -> CoordinatorResult<()>;
    fn start(&mut self) -> CoordinatorResult<()>;
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
                return Err(CoordinatorError::InitializationError("Module not initialized".to_string()));
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
        let module = Box::new(TestModule { initialized: false, started: false });

        assert!(coordinator.register_module(module).is_ok());
        assert!(coordinator.initialize().is_ok());
        assert!(coordinator.start().is_ok());
        assert!(coordinator.stop().is_ok());
    }
}
