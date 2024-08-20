use icn_consensus::ProofOfCooperation;
use icn_blockchain::block::Block;
use icn_blockchain::transaction::Transaction;
use icn_identity::Identity;
use icn_governance::Proposal;
use icn_networking::Networking;
use log::{info, error};

/// The ModuleCoordinator struct is responsible for coordinating the different
/// modules of the ICN node, such as consensus, identity, governance, and networking.
pub struct ModuleCoordinator {
    consensus: ProofOfCooperation,
    identity: Identity,
    governance: Proposal,
    networking: Networking,
}

impl ModuleCoordinator {
    /// Creates a new instance of ModuleCoordinator and initializes the logger.
    pub fn new() -> Self {
        env_logger::init();  // Initialize the logger
        ModuleCoordinator {
            consensus: ProofOfCooperation::new(),
            identity: Identity::new("id123", "ICN Node Identity"),
            governance: Proposal::new(1, "First Proposal"),
            networking: Networking::new(),
        }
    }

    /// Initializes all modules and validates the sample block and its transactions.
    pub async fn initialize(&mut self) {
        info!("Initializing modules...");

        let sample_block = Block::new(1, vec![Transaction::new("Alice", "Bob", 50)], "0".to_string());
        if sample_block.validate_transactions() {
            info!("Transactions in the block are valid.");
        } else {
            error!("Transactions in the block are invalid.");
        }

        if self.consensus.validate(&sample_block) {
            info!("Consensus validated the block successfully.");
        } else {
            error!("Consensus failed to validate the block.");
        }

        info!("Identity ID: {}", self.identity.id);
        info!("Identity Name: {}", self.identity.name);
        info!("Proposal ID: {}", self.governance.id);
        info!("Proposal Description: {}", self.governance.description);

        self.networking.start_server("127.0.0.1:7878").unwrap();
        self.networking.connect_to_peer("127.0.0.1:7878").unwrap();
        self.networking.broadcast_message("Hello, peers!").unwrap();
    }

    /// Starts all modules within the coordinator.
    pub async fn start(&mut self) {
        info!("Starting modules...");
    }

    /// Stops all modules within the coordinator.
    pub async fn stop(&mut self) {
        info!("Stopping modules...");
    }
}
