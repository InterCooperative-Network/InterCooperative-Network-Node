// File: icn_core/src/coordinator/module_coordinator.rs

use std::sync::{Arc, Mutex};
use icn_blockchain::Chain;
use icn_consensus::{ProofOfCooperation, Consensus};
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
            consensus: Arc::clone(&consensus),
            networking: Arc::new(Mutex::new(Networking::new())),
            node_state: Arc::new(Mutex::new(NodeState::Initializing)),
        }
    }

    /// Starts the node by loading the TLS identity and starting the networking server.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object containing network and TLS settings.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the node starts successfully, or an `IcnError` otherwise.
    pub async fn start(&self, config: &Config) -> IcnResult<()> {
        info!("Starting node");

        let identity = Networking::load_tls_identity(
            &config.server.cert_file_path,
            &config.server.key_file_path,
            &config.server.cert_password
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

        self.consensus.validate(latest_block)
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

    /// Attempts to connect to a new peer.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the peer to connect to.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the connection was successful, or an `IcnError` otherwise.
    pub async fn connect_to_peer(&self, address: &str) -> IcnResult<()> {
        let networking = self.networking.lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?;
        networking.connect_to_peer(address).await
    }

    /// Updates the node's configuration.
    ///
    /// # Arguments
    ///
    /// * `new_config` - The new configuration to apply.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the configuration was successfully updated, or an `IcnError` otherwise.
    pub async fn update_config(&self, new_config: &Config) -> IcnResult<()> {
        info!("Updating node configuration");
        
        // Stop the current networking service
        self.stop().await?;

        // Start the networking service with the new configuration
        self.start(new_config).await?;

        info!("Node configuration updated successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config_loader::{ServerConfig, DatabaseConfig};

    #[tokio::test]
    async fn test_module_coordinator_lifecycle() {
        let consensus = Arc::new(ProofOfCooperation::new());
        let coordinator = ModuleCoordinator::new(consensus);

        // Create a mock configuration
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                debug: true,
                cert_file_path: "/path/to/cert.pem".to_string(),
                key_file_path: "/path/to/key.pem".to_string(),
                cert_password: "".to_string(),
            },
            database: DatabaseConfig {
                urls: vec!["postgresql://user:pass@localhost/db1".to_string()],
            },
        };

        // Test starting the node
        assert!(coordinator.start(&config).await.is_ok());

        // Test adding a block
        let transactions = vec!["tx1".to_string(), "tx2".to_string()];
        assert!(coordinator.add_block(transactions).is_ok());

        // Test getting the blockchain length
        assert_eq!(coordinator.get_blockchain_length().unwrap(), 1);

        // Test stopping the node
        assert!(coordinator.stop().await.is_ok());
    }
}