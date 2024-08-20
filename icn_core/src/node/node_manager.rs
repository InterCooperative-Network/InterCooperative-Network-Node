use crate::config::ConfigLoader;
use crate::coordinator::ModuleCoordinator;

pub struct NodeManager {
    config: ConfigLoader,
    coordinator: ModuleCoordinator,
}

impl NodeManager {
    pub fn new() -> Self {
        let config = ConfigLoader::new();
        let coordinator = ModuleCoordinator::new();
        NodeManager { config, coordinator }
    }

    pub async fn start(&mut self) {
        println!("Starting ICN Node...");

        // Load and print configuration values
        if let Ok(node_name) = self.config.get_str("node.name") {
            println!("Node Name: {}", node_name);
        }

        self.coordinator.initialize().await;
        self.coordinator.start().await;
    }

    pub async fn stop(&mut self) {
        println!("Stopping ICN Node...");
        self.coordinator.stop().await;
    }
}
