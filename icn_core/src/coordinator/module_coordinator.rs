use std::sync::{Arc, Mutex};
use icn_blockchain::Chain;
use icn_consensus::{ProofOfCooperation, Consensus};
use icn_networking::Networking;
use icn_shared::{NodeState, IcnResult, IcnError, Block};
use crate::config::Config;  // Ensure correct import from crate
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
    ///
    /// This function initializes the blockchain, consensus, networking, and node state, 
    /// encapsulating them within `Arc<Mutex<>>` for thread-safe shared access.
    ///
    /// # Arguments
    ///
    /// * `consensus` - The consensus mechanism to be used by the blockchain.
    ///
    /// # Returns
    ///
    /// * `ModuleCoordinator` - A new instance of `ModuleCoordinator`.
    pub fn new(consensus: Arc<ProofOfCooperation>) -> Self {
        info!("Initializing ModuleCoordinator");
        ModuleCoordinator {
            blockchain: Arc::new(Mutex::new(Chain::new(Arc::clone(&consensus)))),
            consensus: Arc::clone(&consensus),  // Clone the Arc to avoid moving the value
            networking: Arc::new(Mutex::new(Networking::new())),
            node_state: Arc::new(Mutex::new(NodeState::Initializing)),
        }
    }

    /// Starts the node by loading the TLS identity and starting the networking server.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object containing network settings.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the node starts successfully, or an `IcnError` otherwise.
    pub async fn start(&self, config: &Config) -> IcnResult<()> {
        info!("Starting node");

        let cert_file_path = config.server.host.clone();
        let key_file_path = config.database.urls[0].clone(); // Example of accessing a database URL
        let cert_password = "default_password"; // Replace with actual logic

        let identity = Networking::load_tls_identity(&cert_file_path, &key_file_path, &cert_password)
            .map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))?;

        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;

        networking.start_server("0.0.0.0:8080", identity).await?;

        {
            let mut state = self.node_state.lock()
                .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
            *state = NodeState::Operational;
        }

        info!("Node started successfully");
        Ok(())
    }

    /// Stops the node by shutting down the networking server and updating the node state.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the node stops successfully, or an `IcnError` otherwise.
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
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of `String` objects representing the transactions to include in the new block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added, or an `IcnError` otherwise.
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
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, or an `IcnError` if validation fails.
    pub fn validate_latest_block(&self) -> IcnResult<bool> {
        info!("Validating the latest block");

        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let latest_block = blockchain.latest_block()
            .ok_or_else(|| IcnError::Blockchain("Blockchain is empty".to_string()))?;

        let is_valid = self.consensus.validate(&latest_block)
            .map_err(|e| IcnError::Consensus(format!("Failed to validate block: {}", e)))?;

        info!("Latest block validation result: {}", is_valid);
        Ok(is_valid)
    }

    /// Retrieves the current state of the node.
    ///
    /// # Returns
    ///
    /// * `IcnResult<NodeState>` - Returns the current `NodeState` if successful, or an `IcnError` otherwise.
    pub fn get_node_state(&self) -> IcnResult<NodeState> {
        let state = self.node_state.lock()
            .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))?;
        Ok(*state)
    }

    /// Retrieves the latest block from the blockchain.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Option<Block>>` - Returns the latest `Block` if it exists, `None` if the blockchain is empty, or an `IcnError` otherwise.
    pub fn get_latest_block(&self) -> IcnResult<Option<Block>> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;
        Ok(blockchain.latest_block().cloned())
    }

    /// Retrieves the blockchain length.
    ///
    /// # Returns
    ///
    /// * `IcnResult<usize>` - Returns the current length of the blockchain, or an `IcnError` if the operation fails.
    pub fn get_blockchain_length(&self) -> IcnResult<usize> {
        let blockchain = self.blockchain.lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;
        Ok(blockchain.blocks.len())
    }

    /// Broadcasts a message to all connected peers.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to broadcast.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the message was successfully broadcast, or an `IcnError` otherwise.
    pub async fn broadcast_message(&self, message: &str) -> IcnResult<()> {
        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;
        networking.broadcast_message(message).await
    }
}
