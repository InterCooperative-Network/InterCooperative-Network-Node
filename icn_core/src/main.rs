// File: icn_core/src/main.rs

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{error, info, debug};
use clap::Parser;
use icn_core::config::ConfigLoader;
use icn_core::coordinator::ModuleCoordinator;
use icn_consensus::ProofOfCooperation;
use icn_shared::IcnError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Set the log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), IcnError> {
    let cli = Cli::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&cli.log_level)).init();

    info!("Starting ICN Core...");

    // Load the configuration
    let _config_loader = ConfigLoader::new(&cli.config).map_err(|e| {
        error!("Failed to load configuration: {}", e);
        IcnError::Config(format!("Failed to load configuration: {}", e))
    })?;

    info!("Configuration loaded successfully from: {}", cli.config);

    let _consensus = Arc::new(ProofOfCooperation::new());
    let mut coordinator = ModuleCoordinator::new();

    // Set up graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("Received interrupt signal. Initiating graceful shutdown...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    coordinator.start().map_err(|e| {
        error!("Coordinator failed to start: {}", e);
        IcnError::Other(format!("Coordinator failed to start: {}", e))
    })?;

    info!("ICN Core started successfully");

    // Main loop
    while running.load(Ordering::SeqCst) {
        // Perform periodic tasks here
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        debug!("Node is running...");
    }

    info!("Shutting down ICN Core...");

    // Perform cleanup
    coordinator.stop().map_err(|e| {
        error!("Coordinator failed to stop: {}", e);
        IcnError::Other(format!("Coordinator failed to stop: {}", e))
    })?;

    info!("ICN Core shutdown complete.");

    Ok(())
}
