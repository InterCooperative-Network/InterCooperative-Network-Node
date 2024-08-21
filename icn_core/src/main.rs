// icn_core/src/main.rs

use icn_core::config::ConfigLoader;
use icn_core::coordinator::ModuleCoordinator;
use icn_consensus::ProofOfCooperation;
use std::env;
use log::{error, info};
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

    // Log the start of the application
    info!("Starting ICN Core...");

    // Default configuration file name
    let default_config_file = "config.toml";

    // Obtain the config_path from command-line arguments or use the default
    let config_path = env::args().nth(1).unwrap_or_else(|| default_config_file.to_string());

    // Initialize the ConfigLoader with the specified or default configuration file
    let config_loader = ConfigLoader::new(&config_path).map_err(|e| {
        error!("Failed to load configuration: {}", e);
        IcnError::Config(format!("Failed to load configuration: {}", e))
    })?;

    // Log successful configuration loading
    info!("Configuration loaded successfully from: {}", config_path);

    // Create the consensus mechanism
    let consensus = ProofOfCooperation::new();

    // Initialize the ModuleCoordinator, responsible for managing interactions between modules
    let mut coordinator = ModuleCoordinator::new(consensus);

    // Start the coordinator and handle any errors that occur during startup
    coordinator.start(config_loader.get_config()).await.map_err(|e| {
        error!("Coordinator failed to start: {}", e);
        IcnError::Other(format!("Coordinator failed to start: {}", e))
    })?;

    // Log successful startup
    info!("ICN Core started successfully");

    // Keep the main thread alive
    tokio::signal::ctrl_c().await.map_err(|e| {
        error!("Failed to listen for shutdown signal: {}", e);
        IcnError::Other(format!("Failed to listen for shutdown signal: {}", e))
    })?;

    // Log shutdown
    info!("Shutting down ICN Core...");

    // Perform any necessary cleanup here

    Ok(())
}