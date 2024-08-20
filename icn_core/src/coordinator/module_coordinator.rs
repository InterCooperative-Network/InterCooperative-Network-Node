use std::sync::{Arc, Mutex};
use icn_blockchain::{Chain, Transaction, Block};
use icn_consensus::ProofOfCooperation;
use icn_networking::Networking;
use icn_shared::{NodeState, IcnResult, IcnError};
use config::Config; // Use the correct import for `Config`

pub struct ModuleCoordinator {
    blockchain: Arc<Mutex<Chain>>,
    consensus: Arc<Mutex<ProofOfCooperation>>,
    networking: Arc<Mutex<Networking>>,
    node_state: Arc<Mutex<NodeState>>,
}

impl ModuleCoordinator {
    pub fn new() -> Self {
        ModuleCoordinator {
            blockchain: Arc::new(Mutex::new(Chain::new())),
            consensus: Arc::new(Mutex::new(ProofOfCooperation::new())),
            networking: Arc::new(Mutex::new(Networking::new())),
            node_state: Arc::new(Mutex::new(NodeState::Initializing)),
        }
    }

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

    pub fn add_block(&self, transactions: Vec<Transaction>) -> IcnResult<()> {
        let mut blockchain = self.blockchain
            .lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let previous_hash = blockchain.blocks.last()
            .map(|block| block.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));

        let proposer_id = self.consensus
            .lock()
            .map_err(|_| IcnError::Consensus("Failed to acquire consensus lock".to_string()))?
            .select_proposer()?;

        let new_block = Block::new(
            blockchain.blocks.len() as u64,
            transactions,
            previous_hash,
            proposer_id,
        );

        blockchain.add_block(new_block.transactions, new_block.previous_hash, new_block.proposer_id);
        Ok(())
    }

    pub fn validate_latest_block(&self) -> IcnResult<bool> {
        let blockchain = self.blockchain
            .lock()
            .map_err(|_| IcnError::Blockchain("Failed to acquire blockchain lock".to_string()))?;

        let latest_block = blockchain.blocks.last()
            .ok_or_else(|| IcnError::Blockchain("Blockchain is empty".to_string()))?;

        self.consensus
            .lock()
            .map_err(|_| IcnError::Consensus("Failed to acquire consensus lock".to_string()))?
            .validate(latest_block)
    }
}
