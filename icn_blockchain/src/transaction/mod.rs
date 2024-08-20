use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,

    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}

impl Transaction {
    pub fn new(sender: &str, receiver: &str, amount: u64) -> Self {
        Transaction {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.sender.is_empty() && !self.receiver.is_empty() && self.amount > 0
    }
}

use crate::transaction_type::TransactionType; // Import the TransactionType enum

pub fn validate_transaction(transaction: &Transaction) -> bool {
    match transaction.transaction_type {
        TransactionType::Transfer => {
            // Validation logic specific to Transfer transactions
            // ...
        }
        TransactionType::DeployContract => {
            // Validation logic specific to DeployContract transactions
            // ...
        }
        // Add more match arms for other transaction types as needed
    }
}

use crate::transaction_type::TransactionType; // Import the TransactionType enum

pub fn validate_transaction(transaction: &Transaction) -> bool {
    match transaction.transaction_type {
        TransactionType::Transfer => {
            // Validation logic specific to Transfer transactions
            // ...
        }
        TransactionType::DeployContract => {
            // Validation logic specific to DeployContract transactions
            // ...
        }
        // Add more match arms for other transaction types as needed
    }
}
