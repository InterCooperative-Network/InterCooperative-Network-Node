// /opt/InterCooperative-Network-Node/icn_core/src/main.rs

use icn_core::node::NodeManager;
use log::{error, info};
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Handle command-line arguments for config file path
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config/default_config.toml" // Update this path to your actual config file location
    };

    // Create a new NodeManager instance, handling any potential errors
    let mut node = match NodeManager::new(config_path) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to initialize NodeManager: {:?}", e);
            exit(1); // Exit with an error code
        }
    };

    // Start the node, handling any errors during startup
    match node.start().await {
        Ok(_) => {
            info!("Node started successfully");
        }
        Err(e) => {
            error!("Failed to start node: {:?}", e);
            exit(1); // Exit with an error code
        }
    }

    // Signal handling for graceful shutdown
    if let Err(e) = tokio::signal::ctrl_c().await {
        error!("Failed to listen for shutdown signal: {:?}", e);
    }

    // Stop the node when receiving a shutdown signal
    if let Err(e) = node.stop().await {
        error!("Failed to stop node: {:?}", e);
    } else {
        info!("Node stopped gracefully");
    }
}
