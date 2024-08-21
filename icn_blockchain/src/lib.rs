// icn_blockchain/src/lib.rs

pub mod chain;
pub mod transaction;

pub use icn_shared::Block; // Use Block from icn_shared
pub use chain::Chain;
pub use transaction::Transaction;
