// file: icn_blockchain/src/lib.rs

pub mod chain; // Ensure the `chain` module is correctly referenced

// Import the Chain type from the chain module
pub use crate::chain::Chain;

use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus;
use std::sync::Arc;

/// A high-level description of the blockchain module.
/// This module provides the implementation of the blockchain structure and consensus mechanism.
