// icn_blockchain/src/block/mod.rs

#[derive(Clone)] // Added Clone derive
/// The `Block` struct represents a block in the blockchain.
/// It contains essential data such as index, timestamp, transactions, and hashes.
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<String>,  // Placeholder for actual transactions
    pub previous_hash: String,
    pub hash: String,
    pub proposer_id: String,  // Added field for proposer ID
}

impl Block {
    /// Creates a new block with the given parameters.
    pub fn new(
        index: u64,
        timestamp: u64,
        transactions: Vec<String>,
        previous_hash: String,
        hash: String,
        proposer_id: String,
    ) -> Self {
        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            proposer_id,
        }
    }

    /// Placeholder method for validating transactions in a block.
    pub fn validate_transactions(&self) -> bool {
        !self.transactions.is_empty()
    }
}
