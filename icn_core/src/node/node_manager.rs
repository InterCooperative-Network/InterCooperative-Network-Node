// icn_core/src/node/node_manager.rs

use crate::config::ConfigLoader; // Import ConfigLoader from the config module
use crate::coordinator::ModuleCoordinator;
use icn_shared::IcnResult;

pub struct NodeManager {
    config_loader: ConfigLoader,
    coordinator: ModuleCoordinator,
    // Other fields...
}

impl NodeManager {
    pub fn new(config_loader: ConfigLoader, coordinator: ModuleCoordinator) -> IcnResult<Self> {
        // Initialize the NodeManager with the provided config_loader and coordinator
        Ok(NodeManager {
            config_loader,
            coordinator,
            // Other initializations...
        })
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        // Logic to start the node manager
        self.config_loader.load()?; // Use the config_loader
        self.coordinator.start().await?;
        Ok(())
    }

    // Other methods...
}
