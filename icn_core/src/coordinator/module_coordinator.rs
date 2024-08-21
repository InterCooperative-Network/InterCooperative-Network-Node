use std::sync::{Arc, Mutex};
use icn_blockchain::Chain;
use icn_consensus::Consensus;
use icn_networking::Networking;
use icn_shared::{NodeState, IcnResult, IcnError};
use config::Config;

/// The `ModuleCoordinator` is responsible for coordinating various modules within the 
/// InterCooperative Network (ICN) node. It manages the blockchain, consensus, networking, 
/// and node state, providing a centralized interface for starting and stopping the node.
pub struct ModuleCoordinator<C: Consensus + Clone> {
    blockchain: Arc<Mutex<Chain<C>>>,
    consensus: Arc<Mutex<C>>,
    networking: Arc<Mutex<Networking>>,
    node_state: Arc<Mutex<NodeState>>,
}

impl<C: Consensus + Clone> ModuleCoordinator<C> {
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
    pub fn new(consensus: C) -> Self {
        ModuleCoordinator {
            blockchain: Arc::new(Mutex::new(Chain::new(consensus.clone()))),
            consensus: Arc::new(Mutex::new(consensus)),
            networking: Arc::new(Mutex::new(Networking::new())),
            node_state: Arc::new(Mutex::new(NodeState::Initializing)),
        }
    }

    /// Starts the node by loading the TLS identity and starting the networking server.
    ///
    /// This function reads the configuration to retrieve the paths for the certificate and key files, 
    /// as well as the password for the certificate. It then loads the TLS identity and starts the server.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration object containing network settings.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the node starts successfully, or an `IcnError` otherwise.
    pub async fn start(&self, config: &Config) -> IcnResult<()> {
        let cert_file_path: String = config.get::<String>("network.cert_file_path")
            .map_err(|e| IcnError::Config(format!("Invalid cert file path: {}", e)))?;
        let key_file_path: String = config.get::<String>("network.key_file_path")
            .map_err(|e| IcnError::Config(format!("Invalid key file path: {}", e)))?;
        let cert_password: String = config.get::<String>("network.cert_password")
            .map_err(|e| IcnError::Config(format!("Invalid cert password: {}", e)))?;

        let identity = Networking::load_tls_identity(&cert_file_path, &key_file_path, &cert_password)
            .map_err(|e| IcnError::Network(format!("Failed to load TLS identity: {}", e)))?;
        
        self.networking
            .lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?
            .start_server("0.0.0.0:8080", identity)
            .await?;

        *self.node_state
            .lock()
            .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))? = NodeState::Operational;

        Ok(())
    }

    /// Stops the node by shutting down the networking server and updating the node state.
    ///
    /// This function safely stops the server and sets the node state to `ShuttingDown`.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the node stops successfully, or an `IcnError` otherwise.
    pub async fn stop(&self) -> IcnResult<()> {
        self.networking
            .lock()
            .map_err(|_| IcnError::Network("Failed to acquire networking lock".to_string()))?
            .stop()
            .await?;

        *self.node_state
            .lock()
            .map_err(|_| IcnError::Other("Failed to acquire node state lock".to_string()))? = NodeState::ShuttingDown;

        Ok(())
    }

    /// Adds a new block to the blockchain with the provided transactions.
    ///
    /// This function creates a new block with the given transactions and adds it to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of `String` objects representing the transactions to include in the new block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the block is successfully added, or an `IcnError` otherwise.
    pub fn add_block(&self, transactions: Vec<String>) -> IcnResult<()> {
        let mut blockchain = self.blockchain
            .lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let previous_hash = blockchain.latest_block()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));

        let proposer_id = self.consensus
            .lock()
            .map_err(|_| IcnError::Consensus("Failed to acquire consensus lock".to_string()))?
            .select_proposer()?;

        blockchain.add_block(transactions, previous_hash, proposer_id)
    }

    /// Validates the latest block in the blockchain.
    ///
    /// This function retrieves the latest block from the blockchain and validates it 
    /// using the consensus mechanism.
    ///
    /// # Returns
    ///
    /// * `IcnResult<bool>` - Returns `Ok(true)` if the block is valid, or an `IcnError` if validation fails.
    pub fn validate_latest_block(&self) -> IcnResult<bool> {
        let blockchain = self.blockchain
            .lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let latest_block = blockchain.latest_block()
            .ok_or_else(|| IcnError::Blockchain("Blockchain is empty".to_string()))?;

        self.consensus
            .lock()
            .map_err(|_| IcnError::Consensus("Failed to acquire consensus lock".to_string()))?
            .validate(latest_block)
    }
}