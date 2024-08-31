// File: icn_core/src/coordinator/module_coordinator.rs

use log::{info, error, debug};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// This module defines the `ModuleCoordinator` responsible for managing
/// and coordinating the different modules of the InterCooperative Network (ICN).
/// The coordinator handles initialization, starting, and stopping of all modules.

/// Define a custom error type for the coordinator module.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("Initialization Error: {0}")]
    InitializationError(String),
    #[error("Start Error: {0}")]
    StartError(String),
    #[error("Stop Error: {0}")]
    StopError(String),
}

/// Custom result type for the coordinator module.
pub type CoordinatorResult<T> = Result<T, CoordinatorError>;

/// The `ModuleCoordinator` struct is responsible for managing and coordinating
/// the various modules that make up the ICN node. It ensures that all modules
/// are initialized, started, and stopped in the correct order.
pub struct ModuleCoordinator {
    modules: Vec<Arc<Mutex<Box<dyn Module>>>>,
    shutdown_sender: mpsc::Sender<()>,
    shutdown_receiver: mpsc::Receiver<()>,
}

impl ModuleCoordinator {
    /// Creates a new instance of `ModuleCoordinator`.
    pub fn new() -> Self {
        let (shutdown_sender, shutdown_receiver) = mpsc::channel(1);
        ModuleCoordinator {
            modules: Vec::new(),
            shutdown_sender,
            shutdown_receiver,
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
        debug!("Registering new module");
        self.modules.push(Arc::new(Mutex::new(module)));
        Ok(())
    }

    /// Initializes all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully initialized, or an error otherwise.
    pub fn initialize(&mut self) -> CoordinatorResult<()> {
        info!("Initializing all modules...");
        for (index, module) in self.modules.iter().enumerate() {
            debug!("Initializing module {}", index);
            module.lock()
                .map_err(|e| CoordinatorError::InitializationError(format!("Failed to acquire lock for module {}: {}", index, e)))?
                .initialize()
                .map_err(|e| CoordinatorError::InitializationError(format!("Failed to initialize module {}: {}", index, e)))?;
        }
        info!("All modules initialized successfully");
        Ok(())
    }

    /// Starts all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully started, or an error otherwise.
    pub fn start(&mut self) -> CoordinatorResult<()> {
        info!("Starting all modules...");
        for (index, module) in self.modules.iter().enumerate() {
            debug!("Starting module {}", index);
            module.lock()
                .map_err(|e| CoordinatorError::StartError(format!("Failed to acquire lock for module {}: {}", index, e)))?
                .start()
                .map_err(|e| CoordinatorError::StartError(format!("Failed to start module {}: {}", index, e)))?;
        }
        info!("All modules started successfully");
        Ok(())
    }

    /// Stops all registered modules.
    ///
    /// # Returns
    ///
    /// * `CoordinatorResult<()>` - Returns `Ok(())` if all modules are successfully stopped, or an error otherwise.
    pub fn stop(&mut self) -> CoordinatorResult<()> {
        info!("Stopping all modules...");
        for (index, module) in self.modules.iter().enumerate().rev() {
            debug!("Stopping module {}", index);
            module.lock()
                .map_err(|e| CoordinatorError::StopError(format!("Failed to acquire lock for module {}: {}", index, e)))?
                .stop()
                .map_err(|e| CoordinatorError::StopError(format!("Failed to stop module {}: {}", index, e)))?;
        }
        info!("All modules stopped successfully");
        Ok(())
    }

    /// Returns a clone of the shutdown sender.
    pub fn get_shutdown_sender(&self) -> mpsc::Sender<()> {
        self.shutdown_sender.clone()
    }

    /// Waits for a shutdown signal.
    pub async fn wait_for_shutdown(&mut self) {
        if self.shutdown_receiver.recv().await.is_some() {
            info!("Received shutdown signal");
        }
    }
}

/// The `Module` trait defines the interface for modules that can be managed by the `ModuleCoordinator`.
/// Each module must implement methods for initialization, starting, and stopping.
pub trait Module: Send + Sync {
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

    #[tokio::test]
    async fn test_module_coordinator() {
        let mut coordinator = ModuleCoordinator::new();
        let module = Box::new(TestModule {
            initialized: false,
            started: false,
        });

        assert!(coordinator.register_module(module).is_ok());
        assert!(coordinator.initialize().is_ok());
        assert!(coordinator.start().is_ok());
        assert!(coordinator.stop().is_ok());

        // Test shutdown signal
        let shutdown_sender = coordinator.get_shutdown_sender();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            shutdown_sender.send(()).await.unwrap();
        });
        coordinator.wait_for_shutdown().await;
    }
}
