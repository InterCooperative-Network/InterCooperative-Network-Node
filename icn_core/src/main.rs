use icn_core::config::ConfigLoader;
use icn_core::node::NodeManager;
use icn_core::coordinator::ModuleCoordinator;
use std::env;
use log::error;

/// The main entry point for the ICN Core module.
///
/// This function initializes the node by performing the following steps:
/// 1. Loads the configuration file.
/// 2. Initializes the `ModuleCoordinator`, which manages the various components of the node.
/// 3. Creates and starts the `NodeManager`, which is responsible for the node's lifecycle management.
///
/// # Command-Line Arguments
///
/// The program accepts an optional command-line argument:
/// - `config_path` (optional): The path to the configuration file. If not provided, the default
///   configuration file `config.toml` will be used.
///
/// # Example
///
/// ```bash
/// # Run the program with the default configuration file
/// cargo run
///
/// # Run the program with a custom configuration file
/// cargo run -- config/custom_config.toml
/// ```
///
/// # Error Handling
///
/// If the configuration file cannot be loaded or if the node manager fails to start, an error
/// will be logged and the program will exit gracefully.
///
/// # Dependencies
///
/// This function relies on several core components:
/// - `ConfigLoader`: Loads and parses the configuration file.
/// - `ModuleCoordinator`: Initializes and manages the coordination between different modules.
/// - `NodeManager`: Manages the node's lifecycle, including starting and stopping the node.
#[tokio::main]
async fn main() {
    // Default configuration file name
    let default_config_file = "config.toml";

    // Obtain the config_path from command-line arguments or use the default
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        default_config_file
    };

    // Initialize the ConfigLoader with the specified or default configuration file
    let config_loader = match ConfigLoader::new(config_path) {
        Ok(loader) => loader,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return; // Exit the program if configuration loading fails
        }
    };

    // Initialize the ModuleCoordinator, responsible for managing interactions between modules
    let coordinator = ModuleCoordinator::new();

    // Create a NodeManager to handle the node's lifecycle
    let mut node = NodeManager::new(config_loader, coordinator)
        .expect("Failed to create NodeManager");

    // Start the node manager and handle any errors that occur during startup
    if let Err(e) = node.start().await {
        error!("Node manager failed to start: {}", e);
    }
}
