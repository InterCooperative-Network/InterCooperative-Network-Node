// File: icn_core/src/main.rs

use std::env;
use log::{error, info};
use std::sync::Arc;
use icn_core::config::ConfigLoader; // Correct import
use icn_core::coordinator::ModuleCoordinator;
use icn_consensus::ProofOfCooperation;
use icn_shared::IcnError;

/// The main entry point for the ICN Core module.
///
/// This function initializes the node by performing the following steps:
/// 1. Sets up logging.
/// 2. Loads the configuration file.
/// 3. Initializes the consensus mechanism (Proof of Cooperation).
/// 4. Initializes the `ModuleCoordinator`, which manages the various components of the node.
/// 5. Starts the coordinator, which is responsible for the node's lifecycle management.
///
/// # Command-Line Arguments
///
/// The program accepts an optional command-line argument:
/// - `config_path` (optional): The path to the configuration file. If not provided, the default
///   configuration file `config.toml` will be used.
///
/// # Error Handling
///
/// If any step fails, an error will be logged, and the program will exit gracefully.
#[tokio::main]
async fn main() -> Result<(), IcnError> {
    // Initialize logging
    env_logger::init();

    info!("Starting ICN Core...");

    let default_config_file = "config.toml";
    let config_path = env::args().nth(1).unwrap_or_else(|| default_config_file.to_string());

    // Load the configuration
    let config_loader = ConfigLoader::new(&config_path).map_err(|e| {
        error!("Failed to load configuration: {}", e);
        IcnError::Config(format!("Failed to load configuration: {}", e))
    })?;

    info!("Configuration loaded successfully from: {}", config_path);

    let consensus = Arc::new(ProofOfCooperation::new());
    let coordinator = ModuleCoordinator::new(Arc::clone(&consensus));

    coordinator.start(config_loader.get_config()).await.map_err(|e| {
        error!("Coordinator failed to start: {}", e);
        IcnError::Other(format!("Coordinator failed to start: {}", e))
    })?;

    info!("ICN Core started successfully");

    tokio::signal::ctrl_c().await.map_err(|e| {
        error!("Failed to listen for shutdown signal: {}", e);
        IcnError::Other(format!("Failed to listen for shutdown signal: {}", e))
    })?;

    info!("Shutting down ICN Core...");

    // Perform cleanup here

    Ok(())
}
