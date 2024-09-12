// File: icn_blockchain/src/lib.rs

use std::sync::{Arc, RwLock};
use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus;
use icn_virtual_machine::VirtualMachine;

pub mod chain;
pub mod transaction;

use crate::chain::Chain;
use crate::transaction::{Transaction, TransactionType};

/// Represents the blockchain and its operations.
pub struct Blockchain<C: Consensus> {
    pub chain: Chain<C>,
    pub consensus: Arc<RwLock<C>>,
    pub vm: VirtualMachine,
    state: RwLock<std::collections::HashMap<String, i64>>,
}

impl<C: Consensus> Blockchain<C> {
    /// Creates a new blockchain with the given consensus algorithm and initializes the VM.
    pub fn new(consensus: Arc<RwLock<C>>) -> Self {
        Blockchain {
            chain: Chain::new(consensus.clone()),
            consensus,
            vm: VirtualMachine::new(),
            state: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Adds a new block to the blockchain after validating it.
    pub fn add_block(&mut self, transactions: Vec<String>, proposer_id: String) -> IcnResult<()> {
        let previous_block = self.chain.latest_block()
            .ok_or_else(|| IcnError::Blockchain("Empty blockchain".to_string()))?;
        let new_block = Block::new(
            self.chain.block_count() as u64,
            transactions,
            previous_block.hash.clone(),
            proposer_id,
        );

        // Validate the block using the consensus mechanism
        let consensus = self.consensus.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock on consensus".to_string()))?;
        
        if consensus.validate(&new_block)? {
            drop(consensus); // Release the read lock before acquiring the write lock
            
            // Execute all transactions in the block
            for tx in &new_block.transactions {
                let transaction: Transaction = serde_json::from_str(tx)
                    .map_err(|e| IcnError::Blockchain(format!("Failed to deserialize transaction: {}", e)))?;
                self.execute_transaction(transaction)?;
            }

            // Add the block to the chain
            self.chain.add_block(new_block.clone())?;
            
            // Update the consensus state
            let mut consensus = self.consensus.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock on consensus".to_string()))?;
            consensus.update_state(&new_block)?;
            
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid block".to_string()))
        }
    }

    /// Executes a transaction, updating the blockchain state accordingly.
    pub fn execute_transaction(&self, transaction: Transaction) -> IcnResult<()> {
        match &transaction.transaction_type {
            TransactionType::Transfer { from, to, amount } => {
                self.update_balance(from, -(*amount as i64))?;
                self.update_balance(to, *amount as i64)?;
                Ok(())
            }
            TransactionType::DeployContract { code, .. } => {
                let bytecode = self.vm.compile_contract(code)?;
                self.vm.deploy_contract(bytecode)?;
                Ok(())
            }
            TransactionType::SmartContractExecution { contract_id, method, params } => {
                self.vm.execute_contract(contract_id, method, params)?;
                Ok(())
            }
            TransactionType::ProofValidation { proof_id, data } => {
                self.validate_proof(proof_id, data)?;
                Ok(())
            }
        }
    }

    /// Updates the balance of an account.
    fn update_balance(&self, account: &str, change: i64) -> IcnResult<()> {
        let mut state = self.state.write()
            .map_err(|_| IcnError::Blockchain("Failed to acquire write lock on state".to_string()))?;
        let balance = state.entry(account.to_string()).or_insert(0);
        *balance += change;
        if *balance < 0 {
            return Err(IcnError::Blockchain(format!("Insufficient balance for account {}", account)));
        }
        Ok(())
    }

    /// Validates a proof submitted to the blockchain.
    fn validate_proof(&self, proof_id: &str, data: &[u8]) -> IcnResult<()> {
        // TODO: Implement actual proof validation logic
        if !proof_id.is_empty() && !data.is_empty() {
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid proof data".to_string()))
        }
    }

    /// Validates the integrity of the blockchain.
    pub fn is_valid_chain(&self) -> bool {
        self.chain.is_valid()
    }

    /// Gets the current block count of the blockchain.
    pub fn block_count(&self) -> usize {
        self.chain.block_count()
    }

    /// Gets the latest block in the blockchain.
    pub fn latest_block(&self) -> Option<&Block> {
        self.chain.latest_block()
    }

    /// Gets the balance of an account.
    pub fn get_balance(&self, account: &str) -> IcnResult<i64> {
        let state = self.state.read()
            .map_err(|_| IcnError::Blockchain("Failed to acquire read lock on state".to_string()))?;
        state.get(account)
            .cloned()
            .ok_or_else(|| IcnError::Blockchain(format!("Account {} not found", account)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_consensus::ProofOfCooperation;

    fn setup_blockchain() -> Blockchain<ProofOfCooperation> {
        let consensus = Arc::new(RwLock::new(ProofOfCooperation::new()));
        Blockchain::new(consensus)
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = setup_blockchain();
        assert!(blockchain.is_valid_chain());
        assert_eq!(blockchain.block_count(), 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = setup_blockchain();
        let transactions = vec![
            serde_json::to_string(&Transaction::new(
                "1".to_string(),
                TransactionType::Transfer {
                    from: "account1".to_string(),
                    to: "account2".to_string(),
                    amount: 100,
                },
                None,
                None,
            )).unwrap(),
        ];
        let result = blockchain.add_block(transactions, "proposer1".to_string());
        assert!(result.is_ok());
        assert_eq!(blockchain.block_count(), 1);
    }

    #[test]
    fn test_execute_transaction() {
        let blockchain = setup_blockchain();
        let transaction = Transaction::new(
            "1".to_string(),
            TransactionType::Transfer {
                from: "from_account".to_string(),
                to: "to_account".to_string(),
                amount: 100,
            },
            None,
            None,
        );
        assert!(blockchain.execute_transaction(transaction).is_ok());
        assert_eq!(blockchain.get_balance("from_account").unwrap(), -100);
        assert_eq!(blockchain.get_balance("to_account").unwrap(), 100);
    }

    #[test]
    fn test_invalid_block_addition() {
        let mut blockchain = setup_blockchain();
        let invalid_transactions = vec![];
        let result = blockchain.add_block(invalid_transactions, "proposer1".to_string());
        assert!(result.is_err());
        assert_eq!(blockchain.block_count(), 0);
    }

    #[test]
    fn test_deploy_contract() {
        let blockchain = setup_blockchain();
        let transaction = Transaction::new(
            "2".to_string(),
            TransactionType::DeployContract {
                code: "contract code".to_string(),
                initial_state: "{}".to_string(),
            },
            None,
            None,
        );
        assert!(blockchain.execute_transaction(transaction).is_ok());
        // In a real implementation, we would check if the contract was actually deployed
        // For now, we just check that the operation doesn't fail
    }

    #[test]
    fn test_smart_contract_execution() {
        let blockchain = setup_blockchain();
        let transaction = Transaction::new(
            "3".to_string(),
            TransactionType::SmartContractExecution {
                contract_id: "contract1".to_string(),
                method: "transfer".to_string(),
                params: vec!["recipient".to_string(), "100".to_string()],
            },
            None,
            None,
        );
        assert!(blockchain.execute_transaction(transaction).is_ok());
        // In a real implementation, we would check the effects of the contract execution
        // For now, we just check that the operation doesn't fail
    }

    #[test]
    fn test_proof_validation() {
        let blockchain = setup_blockchain();
        let transaction = Transaction::new(
            "4".to_string(),
            TransactionType::ProofValidation {
                proof_id: "proof1".to_string(),
                data: vec![1, 2, 3, 4, 5],
            },
            None,
            None,
        );
        assert!(blockchain.execute_transaction(transaction).is_ok());
        // In a real implementation, we would check if the proof was actually validated
        // For now, we just check that the operation doesn't fail
    }

    #[test]
    fn test_insufficient_balance() {
        let blockchain = setup_blockchain();
        let transaction = Transaction::new(
            "5".to_string(),
            TransactionType::Transfer {
                from: "empty_account".to_string(),
                to: "any_account".to_string(),
                amount: 100,
            },
            None,
            None,
        );
        let result = blockchain.execute_transaction(transaction);
        assert!(result.is_err());
        if let Err(IcnError::Blockchain(msg)) = result {
            assert!(msg.contains("Insufficient balance"));
        } else {
            panic!("Expected IcnError::Blockchain");
        }
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = setup_blockchain();
        
        // Add a valid block
        let transactions = vec![
            serde_json::to_string(&Transaction::new(
                "1".to_string(),
                TransactionType::Transfer {
                    from: "account1".to_string(),
                    to: "account2".to_string(),
                    amount: 50,
                },
                None,
                None,
            )).unwrap(),
        ];
        assert!(blockchain.add_block(transactions, "proposer1".to_string()).is_ok());
        
        assert!(blockchain.is_valid_chain());
        
        // Attempt to tamper with the blockchain
        if let Some(block) = blockchain.chain.blocks.get_mut(0) {
            block.transactions[0] = serde_json::to_string(&Transaction::new(
                "1".to_string(),
                TransactionType::Transfer {
                    from: "account1".to_string(),
                    to: "account2".to_string(),
                    amount: 1000, // Changed amount
                },
                None,
                None,
            )).unwrap();
        }
        
        assert!(!blockchain.is_valid_chain());
    }
}