// icn_core/src/coordinator/module_coordinator.rs

use std::sync::{Arc, Mutex};
use icn_blockchain::Blockchain;
use icn_consensus::Consensus;
use icn_networking::Networking;
use icn_shared::{NodeState, IcnResult};
use icn_core::config::ConfigLoader;

/// `ModuleCoordinator` coordinates the interactions between various modules of the node.
pub struct ModuleCoordinator {
    blockchain: Arc<Mutex<Blockchain>>,
    consensus: Arc<Mutex<Consensus>>,
    networking: Arc<Mutex<Networking>>,
    node_state: Arc<Mutex<NodeState>>,
}

impl ModuleCoordinator {
    /// Creates a new `ModuleCoordinator` instance.
    pub fn new() -> Self {
        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let networking = Arc::new(Mutex::new(Networking::new()));
        let node_state = Arc::new(Mutex::new(NodeState::Initializing));

        ModuleCoordinator {
            blockchain,
            consensus,
            networking,
            node_state,
        }
    }

    /// Starts the coordinator and its associated modules.
    ///
    /// This function reads necessary configurations (e.g. certificate path, password) from the `ConfigLoader`,
    /// initializes the `Networking` module, and sets the node state to `Operational`.
    ///
    /// # Arguments
    ///
    /// * `config_loader` - A reference to the `ConfigLoader` for accessing configurations.
    ///
    /// # Returns
    /// 
    /// * `IcnResult<()>` -  Ok(()) if successful, or an `IcnError` if an error occurs.
    pub fn start(&self, config_loader: &ConfigLoader) -> IcnResult<()> {
        let cert_file_path = config_loader.get_string("network.cert_file_path")?;
        let cert_password = config_loader.get_string("network.cert_password")?;

        // Initialize the Networking module using the cert_file_path and cert_password
        self.networking
            .lock()
            .unwrap()
            .initialize(&cert_file_path, &cert_password)?;

        // ... other module initializations (blockchain, consensus) ...

        // Update node state to Operational
        *self.node_state.lock().unwrap() = NodeState::Operational;

        Ok(())
    }

    // ... other methods for module interactions ...
}