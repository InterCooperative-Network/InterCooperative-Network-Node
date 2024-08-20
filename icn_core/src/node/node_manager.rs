// src/node/node_manager.rs

use crate::config::ConfigLoader;
use crate::coordinator::ModuleCoordinator;
use icn_shared::IcnError;
use log::info;

/// NodeManager is responsible for initializing and managing the lifecycle of the node.
pub struct NodeManager {
    config: ConfigLoader,
    coordinator: ModuleCoordinator,
}

impl NodeManager {
    /// Creates a new instance of NodeManager with the provided configuration path.
    ///
    /// # Arguments
    ///
    /// * `config_path` - A string slice that holds the path to the configuration file.
    pub fn new(config_path: &str) -> Result<Self, IcnError> {
        let config = ConfigLoader::new(config_path)?;

        // Add a debug output to show the entire loaded configuration
        println!("Loaded full configuration: {:#?}", config.get_config());

        Ok(NodeManager {
            config,
            coordinator: ModuleCoordinator::new(),
        })
    }

    /// Starts the node by initializing all required modules.
    ///
    /// This method performs the following steps:
    /// 1. Loads the node name from the configuration.
    /// 2. Initializes the coordinator with all necessary modules.
    /// 3. Starts the networking server.
    ///
    /// # Errors
    ///
    /// Returns an `IcnError` if any step in the initialization process fails.
    pub async fn start(&mut self) -> Result<(), IcnError> {
        info!("Starting NodeManager...");

        // Accessing the node name directly from the config
        if let Some(node_table) = self.config.get_config().get("node") {
            if let Some(node_name) = node_table.get("name") {
                info!("Node Name: {}", node_name.as_str().unwrap());
            } else {
                eprintln!("Failed to get 'node.name' from configuration: Name not found");
                return Err(IcnError::Config("Config error: node.name not found".to_string()));
            }
        } else {
            eprintln!("Failed to get 'node.name' from configuration: Node table not found");
            return Err(IcnError::Config("Config error: node table not found".to_string()));
        }

        // Initialize and start all modules
        self.coordinator.initialize().await?;
        self.coordinator.start().await?;

        info!("Node started successfully");
        Ok(())
    }

    /// Stops the node by gracefully shutting down all modules.
    ///
    /// This method performs the following steps:
    /// 1. Stops the coordinator which will in turn stop all modules.
    ///
    /// # Errors
    ///
    /// Returns an `IcnError` if any step in the shutdown process fails.
    pub async fn stop(&mut self) -> Result<(), IcnError> {
        info!("Stopping NodeManager...");
        self.coordinator.stop().await?;
        info!("Node stopped successfully");
        Ok(())
    }
}
