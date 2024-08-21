use icn_core::config::ConfigLoader;
use icn_core::coordinator::ModuleCoordinator;
use icn_consensus::ProofOfCooperation;
use std::env;
use log::error;

/// The main entry point for the ICN Core module.
///
/// This function initializes the node by performing the following steps:
/// 1. Loads the configuration file.
/// 2. Initializes the `ModuleCoordinator`, which manages the various components of the node.
/// 3. Starts the coordinator, which is responsible for the node's lifecycle management.
///
/// # Command-Line Arguments
///
/// The program accepts an optional command-line argument:
/// - `config_path` (optional): The path to the configuration file. If not provided, the default
/// configuration file `config.toml` will be used.
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
/// If the configuration file cannot be loaded or if the coordinator fails to start, an error
/// will be logged and the program will exit gracefully.
///
/// # Dependencies
///
/// This function relies on several core components:
/// - `ConfigLoader`: Loads and parses the configuration file.
/// - `ModuleCoordinator`: Initializes and manages the coordination between different modules.
/// - `ProofOfCooperation`: The consensus mechanism used by the network.
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

    // Create the consensus mechanism
    let consensus = ProofOfCooperation::new();

    // Initialize the ModuleCoordinator, responsible for managing interactions between modules
    let mut coordinator = ModuleCoordinator::new(consensus);

    // Start the coordinator and handle any errors that occur during startup
    if let Err(e) = coordinator.start(config_loader.get_config()).await {
        error!("Coordinator failed to start: {}", e);
    }
}