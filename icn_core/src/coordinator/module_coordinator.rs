// /opt/InterCooperative-Network-Node/icn_core/src/coordinator/module_coordinator.rs

use icn_consensus::Consensus;
use icn_blockchain::Chain;
use icn_identity::Identity;
use icn_governance::Proposal; // Keep if you plan to use it later
use icn_networking::Networking;
use icn_shared::IcnError;
use log::info;

/// Coordinates the different modules of an ICN node, including consensus, blockchain, identity, governance, and networking.
pub struct ModuleCoordinator {
    consensus: Consensus,
    blockchain: Chain,
    identity: Identity,
    // Commented out or remove if not used
    // governance: Proposal,
    networking: Networking,
}

impl ModuleCoordinator {
    pub fn new() -> Self {
        ModuleCoordinator {
            consensus: Consensus::new(),
            blockchain: Chain::new(),
            identity: Identity::new("node_id", "node_name"),
            // Comment out or remove if not used
            // governance: Proposal::new(1, "Initial Proposal"),
            networking: Networking::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), IcnError> {
        info!("Initializing modules...");
        self.consensus = Consensus::new();
        self.blockchain = Chain::new();
        self.identity.initialize().map_err(|e| IcnError::Other(e.to_string()))?;
        self.networking.initialize().map_err(|e| IcnError::Other(e.to_string()))?;
        info!("All modules initialized successfully");
        Ok(())
    }

    pub async fn start(&mut self) -> Result<(), IcnError> {
        info!("Starting modules...");
        self.networking.start_server("127.0.0.1:8080")?;
        info!("All modules started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), IcnError> {
        info!("Stopping modules...");
        self.networking.stop().map_err(|e| IcnError::Other(e.to_string()))?;
        info!("All modules stopped successfully");
        Ok(())
    }
}
