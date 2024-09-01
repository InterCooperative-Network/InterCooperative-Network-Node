use serde::{Serialize, Deserialize};
use icn_shared::{IcnError, IcnResult};

/// Represents the different types of transactions supported by the blockchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// A transfer of assets between accounts.
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
    /// Deployment of a smart contract.
    DeployContract {
        code: String,
        initial_state: String,
    },
    /// Execution of a smart contract method.
    SmartContractExecution {
        contract_id: String,
        method: String,
        params: Vec<String>,
    },
    /// Validation of a proof.
    ProofValidation {
        proof_id: String,
        data: Vec<u8>,
    },
}

/// Represents a transaction in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// The unique identifier for the transaction.
    pub id: String,
    /// The type of the transaction.
    pub transaction_type: TransactionType,
    /// The digital signature of the transaction.
    pub signature: Option<String>,
    /// Additional metadata associated with the transaction.
    pub metadata: Option<String>,
}

impl Transaction {
    /// Creates a new `Transaction` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the transaction.
    /// * `transaction_type` - The type of transaction being performed.
    /// * `signature` - An optional digital signature.
    /// * `metadata` - Optional metadata related to the transaction.
    ///
    /// # Returns
    ///
    /// A new `Transaction` instance.
    pub fn new(id: String, transaction_type: TransactionType, signature: Option<String>, metadata: Option<String>) -> Self {
        Transaction {
            id,
            transaction_type,
            signature,
            metadata,
        }
    }

    /// Validates the transaction.
    ///
    /// This function checks the validity of the transaction by validating its type,
    /// checking its signature, and ensuring any additional criteria are met.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the transaction is valid, otherwise an error.
    pub fn validate(&self) -> IcnResult<()> {
        // Validate the transaction type
        self.validate_transaction_type()?;

        // Check if signature exists
        if self.signature.is_none() {
            return Err(IcnError::Transaction("Transaction must have a signature".into()));
        }

        // Validate the signature (placeholder - implement actual signature verification)
        self.verify_signature()?;

        // Additional validation logic can be added here
        Ok(())
    }

    /// Validates the transaction type and its associated data.
    fn validate_transaction_type(&self) -> IcnResult<()> {
        match &self.transaction_type {
            TransactionType::Transfer { from, to, amount } => {
                if from.is_empty() || to.is_empty() {
                    return Err(IcnError::Transaction("Invalid addresses in transfer".into()));
                }
                if *amount == 0 {
                    return Err(IcnError::Transaction("Transfer amount must be greater than zero".into()));
                }
            }
            TransactionType::DeployContract { code, .. } => {
                if code.is_empty() {
                    return Err(IcnError::Transaction("Contract code cannot be empty".into()));
                }
            }
            TransactionType::SmartContractExecution { contract_id, method, .. } => {
                if contract_id.is_empty() || method.is_empty() {
                    return Err(IcnError::Transaction("Invalid contract execution parameters".into()));
                }
            }
            TransactionType::ProofValidation { proof_id, data } => {
                if proof_id.is_empty() || data.is_empty() {
                    return Err(IcnError::Transaction("Invalid proof validation parameters".into()));
                }
            }
        }
        Ok(())
    }

    /// Verifies the digital signature of the transaction.
    fn verify_signature(&self) -> IcnResult<()> {
        // Placeholder for signature verification logic
        // In a real implementation, you would verify the signature against the transaction data
        Ok(())
    }

    /// Processes the transaction within the blockchain.
    ///
    /// This function executes the transaction according to its type and updates the blockchain state accordingly.
    ///
    /// # Returns
    ///
    /// * `IcnResult<()>` - Returns `Ok(())` if the transaction is processed successfully, otherwise an error.
    pub fn process(&self) -> IcnResult<()> {
        match &self.transaction_type {
            TransactionType::Transfer { from, to, amount } => {
                // Logic for processing a transfer transaction
                println!("Processing transfer of {} from {} to {}", amount, from, to);
                // Implement actual balance updates here
                Ok(())
            }
            TransactionType::DeployContract { code, initial_state } => {
                // Logic for processing a smart contract deployment
                println!("Deploying smart contract with code length {} and initial state {}", code.len(), initial_state);
                // Implement actual contract deployment logic here
                Ok(())
            }
            TransactionType::SmartContractExecution { contract_id, method, params } => {
                // Logic for processing a smart contract execution
                println!("Executing method {} on contract {} with {} parameters", method, contract_id, params.len());
                // Implement actual contract execution logic here
                Ok(())
            }
            TransactionType::ProofValidation { proof_id, data } => {
                // Logic for processing a proof validation
                println!("Validating proof {} with data length {}", proof_id, data.len());
                // Implement actual proof validation logic here
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
            },
            Some("signature".to_string()),
            None,
        );
        assert_eq!(tx.id, "tx1");
        assert!(matches!(tx.transaction_type, TransactionType::Transfer { .. }));
    }

    #[test]
    fn test_transaction_validation() {
        let valid_tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
            },
            Some("signature".to_string()),
            None,
        );
        assert!(valid_tx.validate().is_ok());

        let invalid_tx = Transaction::new(
            "tx2".to_string(),
            TransactionType::Transfer {
                from: "".to_string(),
                to: "bob".to_string(),
                amount: 0,
            },
            None,
            None,
        );
        assert!(invalid_tx.validate().is_err());
    }

    #[test]
    fn test_transaction_processing() {
        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: 100,
            },
            Some("signature".to_string()),
            None,
        );
        assert!(tx.process().is_ok());
    }

    #[test]
    fn test_deploy_contract_transaction() {
        let tx = Transaction::new(
            "tx2".to_string(),
            TransactionType::DeployContract {
                code: "contract code".to_string(),
                initial_state: "{}".to_string(),
            },
            Some("signature".to_string()),
            None,
        );
        assert!(tx.validate().is_ok());
        assert!(tx.process().is_ok());
    }

    #[test]
    fn test_smart_contract_execution_transaction() {
        let tx = Transaction::new(
            "tx3".to_string(),
            TransactionType::SmartContractExecution {
                contract_id: "contract1".to_string(),
                method: "transfer".to_string(),
                params: vec!["recipient".to_string(), "100".to_string()],
            },
            Some("signature".to_string()),
            None,
        );
        assert!(tx.validate().is_ok());
        assert!(tx.process().is_ok());
    }

    #[test]
    fn test_proof_validation_transaction() {
        let tx = Transaction::new(
            "tx4".to_string(),
            TransactionType::ProofValidation {
                proof_id: "proof1".to_string(),
                data: vec![1, 2, 3, 4, 5],
            },
            Some("signature".to_string()),
            None,
        );
        assert!(tx.validate().is_ok());
        assert!(tx.process().is_ok());
    }

    #[test]
    fn test_invalid_transactions() {
        let invalid_transfer = Transaction::new(
            "tx5".to_string(),
            TransactionType::Transfer {
                from: "".to_string(),
                to: "bob".to_string(),
                amount: 0,
            },
            Some("signature".to_string()),
            None,
        );
        assert!(invalid_transfer.validate().is_err());

        let invalid_contract_deploy = Transaction::new(
            "tx6".to_string(),
            TransactionType::DeployContract {
                code: "".to_string(),
                initial_state: "{}".to_string(),
            },
            Some("signature".to_string()),
            None,
        );
        assert!(invalid_contract_deploy.validate().is_err());

        let invalid_contract_execution = Transaction::new(
            "tx7".to_string(),
            TransactionType::SmartContractExecution {
                contract_id: "".to_string(),
                method: "".to_string(),
                params: vec![],
            },
            Some("signature".to_string()),
            None,
        );
        assert!(invalid_contract_execution.validate().is_err());

        let invalid_proof_validation = Transaction::new(
            "tx8".to_string(),
            TransactionType::ProofValidation {
                proof_id: "".to_string(),
                data: vec![],
            },
            Some("signature".to_string()),
            None,
        );
        assert!(invalid_proof_validation.validate().is_err());
    }
}
