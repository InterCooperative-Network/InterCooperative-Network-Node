use std::sync::{Arc, Mutex};
use icn_blockchain::Chain;
use icn_consensus::ProofOfCooperation;
use icn_networking::Networking;
use icn_shared::{NodeState, IcnResult, IcnError, Block};
use crate::config::Config;
use log::info;

/// The `ModuleCoordinator` is responsible for coordinating various modules within the 
/// InterCooperative Network (ICN) node. It manages the blockchain, consensus, networking, 
/// and node state, providing a centralized interface for starting and stopping the node.
pub struct ModuleCoordinator {
    blockchain: Arc<Mutex<Chain<ProofOfCooperation>>>,
    consensus: Arc<ProofOfCooperation>,
    networking: Arc<Mutex<Networking>>,
    node_state: Arc<Mutex<NodeState>>,
}

impl ModuleCoordinator {
    /// Creates a new instance of `ModuleCoordinator`.
    pub fn new(consensus: Arc<ProofOfCooperation>) -> Self {
        info!("Initializing ModuleCoordinator");
        ModuleCoordinator {
            blockchain: Arc::new(Mutex::new(Chain::new(Arc::clone(&consensus)))),
            consensus: Arc::clone(&consensus),
            networking: Arc::new(Mutex::new(Networking::new())),
            node_state: Arc::new(Mutex::new(NodeState::Initializing)),
        }
    }

    /// Starts the node by loading the TLS identity and starting the networking server.
    pub async fn start(&self, config: &Config) -> IcnResult<()> {
        info!("Starting node");

        let identity = Networking::load_tls_identity(
            &config.server.cert_file_path,
            &config.server.key_file_path,
            &config.server.cert_password,
        ).map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))?;

        let mut networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;

        let mut port = config.server.port;
        let max_retry = 10;
        let mut retry_count = 0;

        while retry_count < max_retry {
            let address = format!("{}:{}", config.server.host, port);
            match networking.start_server(&address, identity.clone()).await {
                Ok(_) => {
                    info!("Server started on {}", address);
                    break;
                }
                Err(e) => {
                    if retry_count == max_retry - 1 {
                        return Err(IcnError::Network(format!("Failed to bind to any port after {} attempts: {}", max_retry, e)));
                    }
                    info!("Port {} is in use, trying next port", port);
                    port += 1;
                    retry_count += 1;
                }
            }
        }

        {
            let mut state = self.node_state.lock()
                .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
            *state = NodeState::Operational;
        }

        info!("Node started successfully");
        Ok(())
    }

    /// Stops the node by shutting down the networking server and updating the node state.
    pub async fn stop(&self) -> IcnResult<()> {
        info!("Stopping node");

        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;

        networking.stop().await?;

        {
            let mut state = self.node_state.lock()
                .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
            *state = NodeState::ShuttingDown;
        }

        info!("Node stopped successfully");
        Ok(())
    }

    /// Adds a new block to the blockchain with the provided transactions.
    pub fn add_block(&self, transactions: Vec<String>) -> IcnResult<()> {
        info!("Adding new block to the blockchain");

        let mut blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let previous_hash = blockchain.latest_block()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));

        let proposer_id = self.consensus.select_proposer()
            .map_err(|e| IcnError::Consensus(format!("Failed to select proposer: {}", e)))?;

        blockchain.add_block(transactions, previous_hash, proposer_id)?;

        info!("New block added successfully");
        Ok(())
    }

    /// Validates the latest block in the blockchain.
    pub fn validate_latest_block(&self) -> IcnResult<bool> {
        info!("Validating the latest block");

        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let latest_block = blockchain.latest_block()
            .ok_or_else(|| IcnError::Blockchain("Blockchain is empty".to_string()))?;

        self.consensus.validate(&latest_block)
    }

    /// Retrieves the current state of the node.
    pub fn get_node_state(&self) -> IcnResult<NodeState> {
        let state = self.node_state.lock()
            .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
        Ok(*state)
    }

    /// Retrieves the latest block from the blockchain.
    pub fn get_latest_block(&self) -> IcnResult<Option<Block>> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;
        Ok(blockchain.latest_block().cloned())
    }

    /// Retrieves the blockchain length.
    pub fn get_blockchain_length(&self) -> IcnResult<usize> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;
        Ok(blockchain.blocks.len())
    }

    /// Broadcasts a message to all connected peers.
    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;
        networking.broadcast_message(message).await
    }

    /// Attempts to connect to a new peer.
    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;
        networking.connect_to_peer(address).await
    }

    /// Updates the node's configuration.
    pub async fn update_config(&self, new_config: &Config) -> IcnResult<()> {
        info!("Updating node configuration");

        // Reload the TLS identity if needed
        if !new_config.server.cert_file_path.is_empty() && !new_config.server.key_file_path.is_empty() {
            let identity = Networking::load_tls_identity(
                &new_config.server.cert_file_path,
                &new_config.server.key_file_path,
                &new_config.server.cert_password,
            ).map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))?;

            let mut networking = self.networking.lock()
                .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;

            networking.update_tls_identity(identity).await?;
        }

        // Update other configurations as needed
        {
            let mut state = self.node_state.lock()
                .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
            *state = NodeState::Configuring; // Ensure the Configuring state exists in the NodeState enum
        }

        // Apply new config...

        {
            let mut state = self.node_state.lock()
                .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
            *state = NodeState::Operational;
        }

        info!("Node configuration updated successfully");
        Ok(())
    }

    /// Reboots the node by stopping and then restarting it.
    pub async fn reboot_node(&self, config: &Config) -> IcnResult<()> {
        self.stop().await?;
        self.start(config).await
    }
}
