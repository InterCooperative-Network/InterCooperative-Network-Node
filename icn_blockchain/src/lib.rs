// file: icn_blockchain/src/lib.rs

//! This module provides the implementation of the blockchain structure and
//! consensus mechanism for the InterCooperative Network. The main components
//! include the `Chain` struct, which manages blocks, and the integration
//! with various consensus algorithms.

pub mod chain;

pub use crate::chain::Chain;

use icn_shared::{Block, IcnError, IcnResult};
use icn_consensus::Consensus;
use std::sync::Arc;

// Additional code related to blockchain initialization, consensus integration, etc.
