// icn_core/src/main.rs

use icn_core::config::ConfigLoader; // Import from the config module
use icn_core::node::NodeManager;
use icn_core::coordinator::ModuleCoordinator;
use std::env;
use icn_shared::IcnResult;

#[tokio::main] // Using tokio's main to enable async runtime
async fn main() -> IcnResult<()> {
    // Obtain config_path from command-line arguments or environment variables
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "default_config_path"
    };

    // Initialize the ConfigLoader using the correct module path
    let config_loader = ConfigLoader::new(config_path)?;

    // Initialize the ModuleCoordinator
    let mut coordinator = ModuleCoordinator::new();

    // Create a NodeManager with both the ConfigLoader and ModuleCoordinator
    let mut node = NodeManager::new(config_loader, coordinator)?;

    // Start the node manager
    node.start().await?;

    Ok(())
}
