// File: icn_blockchain/src/lib.rs
// Description: This file defines the Blockchain structure and handles operations like adding blocks,
// executing transactions, and interacting with the virtual machine.

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
    /// The chain of blocks.
    pub chain: Chain<C>,
    /// The consensus mechanism used for validating and adding blocks.
    pub consensus: Arc<RwLock<C>>,
    /// The virtual machine for executing smart contracts.
    pub vm: VirtualMachine,
}

impl<C: Consensus> Blockchain<C> {
    /// Creates a new blockchain with the given consensus algorithm and initializes the VM.
    ///
    /// # Arguments
    ///
    /// * `consensus` - The consensus algorithm to use for validating and adding blocks.
    ///
    /// # Returns
    ///
    /// * `Blockchain` - A new instance of the blockchain.
    pub fn new(consensus: Arc<RwLock<C>>) -> Self {
        Blockchain {
            chain: Chain::new(consensus.clone()),
            consensus,
            vm: VirtualMachine::new(),
        }
    }

    /// Adds a new block to the blockchain after validating it.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A vector of serialized transaction strings to be included in the block.
    /// * `proposer_id` - The ID of the node proposing the new block.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the block is successfully added, otherwise an error.
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
        let consensus_read = self.consensus.read()
            .map_err(|_| IcnError::Consensus("Failed to acquire read lock on consensus".to_string()))?;
        
        if consensus_read.validate(&new_block)? {
            // If validation is successful, acquire a write lock to modify the chain
            drop(consensus_read); // Release the read lock before acquiring the write lock
            let mut consensus_write = self.consensus.write()
                .map_err(|_| IcnError::Consensus("Failed to acquire write lock on consensus".to_string()))?;
            
            self.chain.add_block(new_block)?;
            
            // Update the consensus state if necessary
            (*consensus_write).update_state(&self.chain)?;
            Ok(())
        } else {
            Err(IcnError::Blockchain("Invalid block".to_string()))
        }
    }

    /// Executes a transaction, updating the blockchain state accordingly.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be executed.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the transaction is successfully executed, otherwise an error.
    pub fn execute_transaction(&mut self, transaction: Transaction) -> IcnResult<()> {
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
    ///
    /// # Arguments
    ///
    /// * `account` - The account ID to update.
    /// * `change` - The amount to change the balance by (positive for increase, negative for decrease).
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the balance is successfully updated, otherwise an error.
    fn update_balance(&mut self, account: &str, change: i64) -> IcnResult<()> {
        if account.is_empty() {
            return Err(IcnError::Blockchain("Account ID cannot be empty".to_string()));
        }
        // TODO: Implement actual balance update logic
        println!("Updating balance of account {} by {}", account, change);
        Ok(())
    }

    /// Validates a proof submitted to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `proof_id` - The ID of the proof to validate.
    /// * `data` - The data associated with the proof.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns Ok if the proof is successfully validated, otherwise an error.
    fn validate_proof(&self, proof_id: &str, data: &[u8]) -> IcnResult<()> {
        if proof_id.is_empty() || data.is_empty() {
            return Err(IcnError::Blockchain("Invalid proof validation parameters".to_string()));
        }
        // TODO: Implement actual proof validation logic
        println!("Validating proof {} with data length {}", proof_id, data.len());
        Ok(())
    }

    /// Validates the integrity of the blockchain.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the blockchain is valid, otherwise false.
    pub fn is_valid_chain(&self) -> bool {
        self.chain.is_valid()
    }

    /// Gets the current block count of the blockchain.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of blocks in the chain.
    pub fn block_count(&self) -> usize {
        self.chain.block_count()
    }

    /// Gets the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * `Option<&Block>` - The latest block if the chain is not empty, otherwise None.
    pub fn latest_block(&self) -> Option<&Block> {
        self.chain.latest_block()
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
        let transactions = vec!["tx1".to_string(), "tx2".to_string()];
        let result = blockchain.add_block(transactions, "proposer1".to_string());
        assert!(result.is_ok());
        assert_eq!(blockchain.block_count(), 1);
    }

    #[test]
    fn test_execute_transaction() {
        let mut blockchain = setup_blockchain();
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
    }

    #[test]
    fn test_invalid_block_addition() {
        let mut blockchain = setup_blockchain();
        let invalid_transactions = vec![];
        let result = blockchain.add_block(invalid_transactions, "proposer1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_deploy_contract() {
        let mut blockchain = setup_blockchain();
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
    }

    #[test]
    fn test_smart_contract_execution() {
        let mut blockchain = setup_blockchain();
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
    }

    #[test]
    fn test_proof_validation() {
        let mut blockchain = setup_blockchain();
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
    }
}