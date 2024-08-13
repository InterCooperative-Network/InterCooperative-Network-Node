// File: crates/icn_sharding/src/cross_shard_transaction_manager.rs

use icn_common::{IcnResult, IcnError, Transaction, CurrencyType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

pub struct CrossShardTransactionManager {
    shards: Arc<RwLock<HashMap<u64, Shard>>>,
    channels: HashMap<u64, mpsc::Sender<CrossShardMessage>>,
}

struct Shard {
    id: u64,
    balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pending_transactions: Vec<Transaction>,
}

#[derive(Clone)]
enum CrossShardMessage {
    LockFunds(String, CurrencyType, f64),
    ReleaseFunds(String, CurrencyType, f64),
    TransferFunds(String, String, CurrencyType, f64),
}

impl CrossShardTransactionManager {
    pub fn new(shard_count: u64) -> Self {
        let mut shards = HashMap::new();
        let mut channels = HashMap::new();

        for i in 0..shard_count {
            shards.insert(i, Shard {
                id: i,
                balances: HashMap::new(),
                pending_transactions: Vec::new(),
            });

            let (tx, mut rx) = mpsc::channel(100);
            channels.insert(i, tx);

            let shard_shards = Arc::clone(&Arc::new(RwLock::new(shards.clone())));

            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    let mut shards = shard_shards.write().unwrap();
                    let shard = shards.get_mut(&i).unwrap();
                    match message {
                        CrossShardMessage::LockFunds(address, currency_type, amount) => {
                            if let Some(balance) = shard.balances.get_mut(&address).and_then(|b| b.get_mut(&currency_type)) {
                                if *balance >= amount {
                                    *balance -= amount;
                                }
                            }
                        }
                        CrossShardMessage::ReleaseFunds(address, currency_type, amount) => {
                            shard.balances.entry(address)
                                .or_insert_with(HashMap::new)
                                .entry(currency_type)
                                .and_modify(|balance| *balance += amount)
                                .or_insert(amount);
                        }
                        CrossShardMessage::TransferFunds(from, to, currency_type, amount) => {
                            if let Some(from_balance) = shard.balances.get_mut(&from).and_then(|b| b.get_mut(&currency_type)) {
                                if *from_balance >= amount {
                                    *from_balance -= amount;
                                    shard.balances.entry(to)
                                        .or_insert_with(HashMap::new)
                                        .entry(currency_type)
                                        .and_modify(|balance| *balance += amount)
                                        .or_insert(amount);
                                }
                            }
                        }
                    }
                }
            });
        }

        CrossShardTransactionManager {
            shards: Arc::new(RwLock::new(shards)),
            channels,
        }
    }

    pub async fn process_cross_shard_transaction(&self, transaction: &Transaction) -> IcnResult<()> {
        let from_shard = self.get_shard_for_address(&transaction.from);
        let to_shard = self.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err(IcnError::Sharding("Not a cross-shard transaction".into()));
        }

        // Phase 1: Lock funds in the source shard
        self.channels[&from_shard].send(CrossShardMessage::LockFunds(
            transaction.from.clone(),
            transaction.currency_type.clone(),
            transaction.amount,
        )).await.map_err(|e| IcnError::Sharding(format!("Failed to send lock funds message: {}", e)))?;

        // Phase 2: Transfer funds to the destination shard
        self.channels[&to_shard].send(CrossShardMessage::TransferFunds(
            transaction.from.clone(),
            transaction.to.clone(),
            transaction.currency_type.clone(),
            transaction.amount,
        )).await.map_err(|e| IcnError::Sharding(format!("Failed to send transfer funds message: {}", e)))?;

        // Phase 3: Release funds in the source shard (in case of failure)
        // In a real implementation, you'd need a way to handle failures and rollbacks
        
        Ok(())
    }

    fn get_shard_for_address(&self, address: &str) -> u64 {
        // Simple hash function to determine shard
        let mut hash = 0u64;
        for byte in address.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash % self.shards.read().unwrap().len() as u64
    }

    pub async fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> IcnResult<f64> {
        let shard_id = self.get_shard_for_address(address);
        let shards = self.shards.read().unwrap();
        let shard = shards.get(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".into()))?;
        
        shard.balances.get(address)
            .and_then(|balances| balances.get(currency_type))
            .copied()
            .ok_or_else(|| IcnError::Sharding("Balance not found".into()))
    }

    pub async fn initialize_balance(&self, address: &str, currency_type: &CurrencyType, amount: f64) -> IcnResult<()> {
        let shard_id = self.get_shard_for_address(address);
        let mut shards = self.shards.write().unwrap();
        let shard = shards.get_mut(&shard_id).ok_or_else(|| IcnError::Sharding("Shard not found".into()))?;

        shard.balances.entry(address.to_string())
            .or_insert_with(HashMap::new)
            .insert(currency_type.clone(), amount);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cross_shard_transaction() {
        let manager = CrossShardTransactionManager::new(2);

        manager.initialize_balance("Alice", &CurrencyType::BasicNeeds, 1000.0).await.unwrap();
        manager.initialize_balance("Bob", &CurrencyType::BasicNeeds, 0.0).await.unwrap();

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 500.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        manager.process_cross_shard_transaction(&transaction).await.unwrap();

        // Give some time for the async operations to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let alice_balance = manager.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        let bob_balance = manager.get_balance("Bob", &CurrencyType::BasicNeeds).await.unwrap();

        assert_eq!(alice_balance, 500.0);
        assert_eq!(bob_balance, 500.0);
    }

    #[tokio::test]
    async fn test_same_shard_transaction_error() {
        let manager = CrossShardTransactionManager::new(1);

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 500.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        let result = manager.process_cross_shard_transaction(&transaction).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Sharding error: Not a cross-shard transaction");
    }

    #[tokio::test]
    async fn test_insufficient_balance() {
        let manager = CrossShardTransactionManager::new(2);

        manager.initialize_balance("Alice", &CurrencyType::BasicNeeds, 100.0).await.unwrap();
        manager.initialize_balance("Bob", &CurrencyType::BasicNeeds, 0.0).await.unwrap();

        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 500.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: 0,
            signature: None,
        };

        manager.process_cross_shard_transaction(&transaction).await.unwrap();

        // Give some time for the async operations to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let alice_balance = manager.get_balance("Alice", &CurrencyType::BasicNeeds).await.unwrap();
        let bob_balance = manager.get_balance("Bob", &CurrencyType::BasicNeeds).await.unwrap();

        // The transaction should not have gone through due to insufficient balance
        assert_eq!(alice_balance, 100.0);
        assert_eq!(bob_balance, 0.0);
    }
}