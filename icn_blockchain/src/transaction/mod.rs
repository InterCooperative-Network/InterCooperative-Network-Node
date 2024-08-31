// icn_blockchain/src/transaction/mod.rs

use serde::{Serialize, Deserialize};

/// Represents the different types of transactions supported by the blockchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    DeployContract,
    // Add more variants as needed
}

/// The `Transaction` struct defines a blockchain transaction, including its sender,
/// receiver, amount, and type of transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub transaction_type: TransactionType,
}

impl Transaction {
    /// Creates a new `Transaction` instance with the given sender, receiver, and amount.
    ///
    /// # Arguments
    ///
    /// * `sender` - The ID of the sender.
    /// * `receiver` - The ID of the receiver.
    /// * `amount` - The amount being transferred in the transaction.
    ///
    /// # Returns
    ///
    /// * `Transaction` - A new `Transaction` instance of type `Transfer`.
    pub fn new(sender: &str, receiver: &str, amount: u64) -> Self {
        Transaction {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount,
            transaction_type: TransactionType::Transfer,
        }
    }

    /// Validates the transaction based on its type.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the transaction is valid, `false` otherwise.
    ///
    /// This method performs basic validation depending on the transaction type.
    pub fn validate_transaction(&self) -> bool {
        match self.transaction_type {
            TransactionType::Transfer => {
                !self.sender.is_empty() && !self.receiver.is_empty() && self.amount > 0
            }
            TransactionType::DeployContract => {
                // Validation logic specific to DeployContract transactions
                // Add your validation logic here
                false // Placeholder for now
            }
            // Add more match arms for other transaction types as needed
        }
    }
}
