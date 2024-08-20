// icn_core/src/main.rs

use icn_core::config::ConfigLoader;
use icn_core::node::NodeManager;
use icn_core::coordinator::ModuleCoordinator;
use std::env;
use log::error;

#[tokio::main]
async fn main() {
    // Default configuration file name
    let default_config_file = "config.toml";

    // Obtain config_path from command-line arguments or use the default
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        default_config_file
    };

    // Initialize the ConfigLoader and handle potential errors
    let config_loader = match ConfigLoader::new(config_path) {
        Ok(loader) => loader,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return; // Exit the program if configuration loading fails
        }
    };

    // Initialize the ModuleCoordinator
    let coordinator = ModuleCoordinator::new();

    // Create a NodeManager
    let mut node = NodeManager::new(config_loader, coordinator).expect("Failed to create NodeManager");

    // Start the node manager and handle potential errors
    if let Err(e) = node.start().await {
        error!("Node manager failed to start: {}", e);
    }
}