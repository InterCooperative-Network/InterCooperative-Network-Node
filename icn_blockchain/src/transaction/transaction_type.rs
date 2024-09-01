// File Location: icn_blockchain/src/transaction/transaction_type.rs

//! Defines the types of transactions that can be performed on the ICN blockchain.
//!
//! This module includes the enumeration of different transaction types,
//! each representing a specific operation or interaction within the blockchain.

/// Enum representing various transaction types in the ICN blockchain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransactionType {
    /// A simple transfer of assets between accounts.
    Transfer { from: String, to: String, amount: u64 },

    /// A smart contract execution transaction.
    SmartContractExecution { contract_id: String, method: String, params: Vec<String> },

    /// A transaction for deploying a new smart contract.
    SmartContractDeployment { code: Vec<u8>, initial_state: String },

    /// A transaction for validating a proof within the blockchain.
    ProofValidation { proof_id: String, data: Vec<u8> },

    /// Additional transaction types can be added here as needed.
}

impl TransactionType {
    /// Validates the transaction type based on its specific rules.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            TransactionType::Transfer { from, to, amount } => {
                // Example validation: Ensure 'from' and 'to' are valid addresses
                if from.is_empty() || to.is_empty() {
                    return Err("Invalid addresses".into());
                }
                if *amount == 0 {
                    return Err("Amount must be greater than zero".into());
                }
                Ok(())
            }
            TransactionType::SmartContractExecution { .. } => {
                // Validation logic for smart contract execution
                Ok(())
            }
            TransactionType::SmartContractDeployment { .. } => {
                // Validation logic for smart contract deployment
                Ok(())
            }
            TransactionType::ProofValidation { .. } => {
                // Validation logic for proof validation
                Ok(())
            }
        }
    }
}
