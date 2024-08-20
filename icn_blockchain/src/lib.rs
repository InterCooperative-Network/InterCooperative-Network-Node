// icn_blockchain/src/lib.rs

pub mod block;
pub mod chain;
pub mod transaction; // Ensure this line is included to expose the transaction module

pub use block::Block;
pub use chain::Chain;
pub use transaction::Transaction; // Add this to expose Transaction if needed
