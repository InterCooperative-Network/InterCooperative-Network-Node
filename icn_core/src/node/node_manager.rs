use crate::config::ConfigLoader;  // Use `crate` instead of `icn_core`
use icn_shared::IcnResult;
use crate::coordinator::ModuleCoordinator;

pub struct NodeManager {
    config: ConfigLoader,
    coordinator: ModuleCoordinator,
}

impl NodeManager {
    pub fn new(config_loader: ConfigLoader, coordinator: ModuleCoordinator) -> Self {
        NodeManager {
            config: config_loader,
            coordinator,
        }
    }

    pub fn load_configuration(&self) -> IcnResult<()> {
        println!("Loaded full configuration: {:#?}", self.config.get_config());

        if let Some(_node_table) = self.config.get_config().get("node") {
            // Handle node table...
        }

        Ok(())
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        self.coordinator.start().await?; // Async function, use await
        Ok(())
    }

    pub async fn stop(&mut self) -> IcnResult<()> {
        self.coordinator.stop()?; // No await needed as this is not async
        Ok(())
    }
}
