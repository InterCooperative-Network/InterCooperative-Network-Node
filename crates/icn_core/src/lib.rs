// File: crates/icn_core/src/lib.rs

use icn_common::{Config, Transaction, Proposal, ProposalStatus, Vote, CurrencyType, IcnResult, IcnError, NetworkStats};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::{GovernanceSystem, ProposalType, ProposalCategory};
use icn_identity::IdentityService;
use icn_network::NetworkManager;
use icn_sharding::ShardingManager;
use icn_vm::SmartContractExecutor;
use icn_storage::StorageManager;
use icn_zkp::ZKPManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use log::{info, warn, error};

pub struct IcnNode {
    config: Config,
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<GovernanceSystem>>,
    identity_service: Arc<RwLock<IdentityService>>,
    network_manager: Arc<RwLock<NetworkManager>>,
    sharding_manager: Arc<RwLock<ShardingManager>>,
    smart_contract_executor: Arc<RwLock<SmartContractExecutor>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    zkp_manager: Arc<RwLock<ZKPManager>>,
}

impl IcnNode {
    pub async fn new(config: Config) -> IcnResult<Self> {
        let blockchain = Arc::new(RwLock::new(Blockchain::new(config.difficulty)));
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(config.consensus_threshold, config.consensus_quorum)?));
        let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
        let governance = Arc::new(RwLock::new(GovernanceSystem::new()));
        let identity_service = Arc::new(RwLock::new(IdentityService::new()));
        let network_manager = Arc::new(RwLock::new(NetworkManager::new(config.network_port)));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(config.shard_count)));
        let smart_contract_executor = Arc::new(RwLock::new(SmartContractExecutor::new()));
        let storage_manager = Arc::new(RwLock::new(StorageManager::new(3))); // Assuming a replication factor of 3
        let zkp_manager = Arc::new(RwLock::new(ZKPManager::new(64))); // Assuming a max bitsize of 64

        Ok(Self {
            config,
            blockchain,
            consensus,
            currency_system,
            governance,
            identity_service,
            network_manager,
            sharding_manager,
            smart_contract_executor,
            storage_manager,
            zkp_manager,
        })
    }

    pub async fn start(&self) -> IcnResult<()> {
        self.consensus.write().await.start()?;
        self.network_manager.write().await.start()?;
        Ok(())
    }

    pub async fn stop(&self) -> IcnResult<()> {
        self.consensus.write().await.stop()?;
        self.network_manager.write().await.stop()?;
        Ok(())
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        self.verify_transaction(&transaction).await?;
        let shard_id = self.sharding_manager.read().await.get_shard_for_address(&transaction.from);
        self.blockchain.write().await.add_transaction(transaction.clone())?;
        self.currency_system.write().await.process_transaction(&transaction)?;
        self.sharding_manager.write().await.process_transaction(shard_id, &transaction)?;
        Ok(())
    }

    pub async fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        self.verify_proposal(&proposal).await?;
        let proposal_id = self.governance.write().await.create_proposal(proposal)?;
        self.network_manager.read().await.broadcast_proposal(&proposal_id)?;
        Ok(proposal_id)
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        self.currency_system.read().await.get_balance(address, currency_type)
    }

    pub async fn create_identity(&self, attributes: HashMap<String, String>) -> IcnResult<String> {
        self.identity_service.write().await.create_identity(attributes)
    }

    pub async fn allocate_resource(&self, resource_type: &str, amount: u64) -> IcnResult<()> {
        self.sharding_manager.write().await.allocate_resource(resource_type, amount)
    }

    pub async fn get_network_stats(&self) -> IcnResult<NetworkStats> {
        self.network_manager.read().await.get_stats()
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> IcnResult<Option<Proposal>> {
        self.governance.read().await.get_proposal(proposal_id)
    }

    pub async fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        self.governance.read().await.list_active_proposals()
    }

    pub async fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        self.governance.write().await.vote_on_proposal(proposal_id, voter, in_favor, weight)
    }

    pub async fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        self.governance.write().await.finalize_proposal(proposal_id)
    }

    pub async fn execute_proposal(&self, proposal_id: &str) -> IcnResult<()> {
        self.governance.write().await.execute_proposal(proposal_id)
    }

    pub async fn mint_currency(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        self.currency_system.write().await.mint(address, currency_type, amount)
    }

    pub async fn get_identity(&self, id: &str) -> IcnResult<HashMap<String, String>> {
        self.identity_service.read().await.get_identity(id)
    }

    pub async fn update_identity(&self, id: &str, attributes: HashMap<String, String>) -> IcnResult<()> {
        self.identity_service.write().await.update_identity(id, attributes)
    }

    pub async fn execute_smart_contract(&self, contract_id: &str, function: &str, args: Vec<icn_vm::Value>) -> IcnResult<Option<icn_vm::Value>> {
        let contract_code = self.storage_manager.read().await.retrieve_data(contract_id)?;
        let mut executor = self.smart_contract_executor.write().await;
        executor.load_contract(contract_id, &String::from_utf8(contract_code)?)?;
        let result = executor.execute_contract(contract_id, function, args)?;

        if let Some(state_changes) = executor.get_state_changes(contract_id) {
            for (key, value) in state_changes {
                self.storage_manager.write().await.store_data(&format!("{}:{}", contract_id, key), value.to_vec())?;
            }
        }

        Ok(result)
    }

    pub async fn get_blockchain(&self) -> IcnResult<Vec<icn_blockchain::Block>> {
        Ok(self.blockchain.read().await.chain.clone())
    }

    pub async fn get_shard_count(&self) -> u64 {
        self.config.shard_count
    }

    pub async fn get_consensus_threshold(&self) -> f64 {
        self.config.consensus_threshold
    }

    pub async fn get_consensus_quorum(&self) -> f64 {
        self.config.consensus_quorum
    }

    pub async fn get_network_port(&self) -> u16 {
        self.config.network_port
    }

    pub async fn get_proposal_status(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.governance.read().await.get_proposal(proposal_id)?
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;
        Ok(proposal.status)
    }

    pub async fn get_total_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let mut total_balance = 0.0;
        for shard_id in 0..self.config.shard_count {
            total_balance += self.sharding_manager.read().await.get_shard_balance(shard_id, address, currency_type)?;
        }
        Ok(total_balance)
    }

    pub async fn list_active_proposals_with_status(&self) -> IcnResult<Vec<(Proposal, f64)>> {
        let proposals = self.governance.read().await.list_active_proposals()?;
        let mut proposals_with_status = Vec::new();
        
        for proposal in proposals {
            let votes = self.governance.read().await.get_votes(&proposal.id)?;
            let total_votes: f64 = votes.iter().map(|v| v.weight).sum();
            let votes_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();
            let status = if total_votes > 0.0 { votes_in_favor / total_votes } else { 0.0 };
            proposals_with_status.push((proposal, status));
        }
        
        Ok(proposals_with_status)
    }

    pub async fn check_sufficient_balance(&self, address: &str, amount: f64, currency_type: &CurrencyType) -> IcnResult<bool> {
        let balance = self.get_total_balance(address, currency_type).await?;
        Ok(balance >= amount)
    }

    pub async fn get_node_reputation(&self, node_id: &str) -> IcnResult<f64> {
        self.consensus.read().await.get_node_reputation(node_id)
    }

    pub async fn update_node_reputation(&self, node_id: &str, change: f64) -> IcnResult<()> {
        let mut consensus = self.consensus.write().await;
        let current_reputation = consensus.get_node_reputation(node_id)?;
        let new_reputation = (current_reputation + change).max(0.0).min(1.0);
        consensus.set_node_reputation(node_id, new_reputation)?;
        Ok(())
    }

    pub async fn get_shard_for_address(&self, address: &str) -> u64 {
        self.sharding_manager.read().await.get_shard_for_address(address)
    }

    pub async fn create_smart_contract(&self, code: String) -> IcnResult<String> {
        let contract_id = uuid::Uuid::new_v4().to_string();
        self.storage_manager.write().await.store_data(&contract_id, code.into_bytes())?;
        Ok(contract_id)
    }

    pub async fn get_smart_contract(&self, contract_id: &str) -> IcnResult<Option<String>> {
        let contract_code = self.storage_manager.read().await.retrieve_data(contract_id)?;
        Ok(Some(String::from_utf8(contract_code)?))
    }

    pub async fn update_smart_contract(&self, contract_id: &str, new_code: String) -> IcnResult<()> {
        self.storage_manager.write().await.store_data(contract_id, new_code.into_bytes())
    }

    pub async fn delete_smart_contract(&self, contract_id: &str) -> IcnResult<()> {
        self.storage_manager.write().await.remove_data(contract_id)
    }

    pub async fn create_zkp(&self, transaction: &Transaction) -> IcnResult<(Vec<u8>, Vec<u8>)> {
        let zkp_manager = self.zkp_manager.read().await;
        let (proof, committed_values) = zkp_manager.create_proof(transaction)?;
        Ok((proof.to_bytes(), serde_json::to_vec(&committed_values)?))
    }

    async fn verify_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        if !transaction.verify()? {
            return Err(IcnError::Blockchain("Invalid transaction signature".into()));
        }

        let sender_balance = self.get_balance(&transaction.from, &transaction.currency_type).await?;
        if sender_balance < transaction.amount {
            return Err(IcnError::Currency("Insufficient balance".into()));
        }

        Ok(())
    }

    async fn verify_proposal(&self, proposal: &Proposal) -> IcnResult<()> {
        if self.get_identity(&proposal.proposer).await.is_err() {
            return Err(IcnError::Governance("Proposer does not exist".into()));
        }

        if !matches!(proposal.proposal_type, ProposalType::Constitutional | ProposalType::EconomicAdjustment | ProposalType::NetworkUpgrade | ProposalType::CommunityInitiative) {
            return Err(IcnError::Governance("Invalid proposal type".into()));
        }

        if proposal.required_quorum < 0.0 || proposal.required_quorum > 1.0 {
            return Err(IcnError::Governance("Invalid quorum value".into()));
        }

        if proposal.required_majority < 0.5 || proposal.required_majority > 1.0 {
            return Err(IcnError::Governance("Invalid majority value".into()));
        }

        if proposal.voting_ends_at <= Utc::now() {
            return Err(IcnError::Governance("Voting period has already ended".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    async fn create_test_node() -> IcnNode {
        let config = Config {
            shard_count: 1,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
            difficulty: 2,
        };
        IcnNode::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_node_creation_and_lifecycle() {
        let node = create_test_node().await;
        assert_eq!(node.get_shard_count().await, 1);
        assert_eq!(node.get_consensus_threshold().await, 0.66);
        assert_eq!(node.get_consensus_quorum().await, 0.51);
        assert_eq!(node.get_network_port().await, 8080);

        assert!(node.start().await.is_ok());
        assert!(node.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_processing() {
        let node = create_test_node().await;

        // Mint some currency for testing
        assert!(node.mint_currency("Alice", &CurrencyType::BasicNeeds, 1000.0).await.is_ok());

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 100.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        assert!(node.process_transaction(transaction).await.is_ok());

        // Check balances
        let alice_balance = node.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        let bob_balance = node.get_balance("Bob", &CurrencyType::BasicNeeds).await.unwrap();
        assert_eq!(alice_balance, 900.0);
        assert_eq!(bob_balance, 100.0);
    }

    #[tokio::test]
    async fn test_proposal_lifecycle() {
        let node = create_test_node().await;

        let proposal = Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.51,
            required_majority: 0.66,
            execution_threshold: None,
            execution_delay: None,
        };

        // Create proposal
        let proposal_id = node.create_proposal(proposal).await.unwrap();

        // Check if proposal exists
        let retrieved_proposal = node.get_proposal(&proposal_id).await.unwrap();
        assert!(retrieved_proposal.is_some());

        // List active proposals
        let active_proposals = node.list_active_proposals().await.unwrap();
        assert_eq!(active_proposals.len(), 1);

        // Vote on proposal
        assert!(node.vote_on_proposal(&proposal_id, "Alice".to_string(), true, 1.0).await.is_ok());
        assert!(node.vote_on_proposal(&proposal_id, "Bob".to_string(), false, 1.0).await.is_ok());

        // Get proposal status
        let status = node.get_proposal_status(&proposal_id).await.unwrap();
        assert_eq!(status, ProposalStatus::Active);

        // Finalize proposal
        let final_status = node.finalize_proposal(&proposal_id).await.unwrap();
        assert_eq!(final_status, ProposalStatus::Passed);

        // Execute proposal
        assert!(node.execute_proposal(&proposal_id).await.is_ok());
        let executed_proposal_status = node.get_proposal_status(&proposal_id).await.unwrap();
        assert_eq!(executed_proposal_status, ProposalStatus::Executed);
    }

    #[tokio::test]
    async fn test_smart_contract_execution() {
        let node = create_test_node().await;

        // Create a simple smart contract
        let contract_code = r#"
            fn add(a: i64, b: i64) -> i64 {
                a + b
            }
        "#.to_string();
        let contract_id = node.create_smart_contract(contract_code).await.unwrap();

        // Execute the smart contract
        let result = node.execute_smart_contract(&contract_id, "add", vec![icn_vm::Value::Int(5), icn_vm::Value::Int(3)]).await.unwrap();
        assert_eq!(result, Some(icn_vm::Value::Int(8)));

        // Test contract state persistence
        let state_key = "last_result";
        node.execute_smart_contract(&contract_id, "add", vec![icn_vm::Value::Int(10), icn_vm::Value::Int(15)]).await.unwrap();
        let state_value = node.storage_manager.read().await.retrieve_data(&format!("{}:{}", contract_id, state_key)).unwrap();
        assert_eq!(String::from_utf8(state_value).unwrap(), "25");
    }

    #[tokio::test]
    async fn test_node_reputation_management() {
        let node = create_test_node().await;
        let node_id = "test_node";

        // Set initial reputation
        node.update_node_reputation(node_id, 0.5).await.unwrap();
        let initial_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(initial_reputation, 0.5);

        // Increase reputation
        node.update_node_reputation(node_id, 0.2).await.unwrap();
        let increased_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(increased_reputation, 0.7);

        // Decrease reputation
        node.update_node_reputation(node_id, -0.3).await.unwrap();
        let decreased_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(decreased_reputation, 0.4);

        // Test upper bound
        node.update_node_reputation(node_id, 1.0).await.unwrap();
        let max_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(max_reputation, 1.0);

        // Test lower bound
        node.update_node_reputation(node_id, -2.0).await.unwrap();
        let min_reputation = node.get_node_reputation(node_id).await.unwrap();
        assert_eq!(min_reputation, 0.0);
    }

    #[tokio::test]
    async fn test_zkp_creation() {
        let node = create_test_node().await;

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };

        let (proof, committed_values) = node.create_zkp(&transaction).await.unwrap();
        assert!(!proof.is_empty());
        assert!(!committed_values.is_empty());
    }
}
