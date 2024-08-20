// icn_core/src/node/node_manager.rs

use crate::config::ConfigLoader;
use crate::coordinator::ModuleCoordinator;
use crate::errors::{IcnError, IcnResult};

/// The NodeManager struct is responsible for managing the ICN node's lifecycle,
/// including starting, stopping, and handling configuration.
pub struct NodeManager {
    config: ConfigLoader,
    coordinator: ModuleCoordinator,
}

impl NodeManager {
    /// Creates a new NodeManager instance with a ConfigLoader and ModuleCoordinator.
    pub fn new() -> Self {
        let config = ConfigLoader::new();
        let coordinator = ModuleCoordinator::new();
        NodeManager { config, coordinator }
    }

    /// Starts the ICN node by loading configuration and initializing all modules.
    /// Returns an `IcnResult` indicating success or an error.
    pub async fn start(&mut self) -> IcnResult<()> {
        println!("Starting ICN Node...");

        // Attempt to retrieve the node name from configuration
        match self.config.get_str("node.name") {
            Ok(node_name) => println!("Node Name: {}", node_name),
            Err(e) => return Err(IcnError::ConfigError(e)), // Return error if configuration fails
        }

        // Initialize and start all modules using the coordinator
        self.coordinator.initialize().await?;
        self.coordinator.start().await?;
        Ok(())
    }

    /// Stops the ICN node and gracefully shuts down all modules.
    /// Returns an `IcnResult` indicating success or an error.
    pub async fn stop(&mut self) -> IcnResult<()> {
        println!("Stopping ICN Node...");
        self.coordinator.stop().await?;
        Ok(())
    }
}
