// File: icn_consensus/src/lib.rs

//! This module defines the consensus mechanisms for the InterCooperative Network (ICN) project.
//! It includes traits and structures for implementing various consensus algorithms.

pub mod consensus;
pub mod proof_of_cooperation;

use icn_shared::{Block, IcnError}; // Importing necessary types from icn_shared
use crate::consensus::Consensus as CoreConsensus; // Renaming to avoid conflict

pub use crate::consensus::Consensus; // Exporting the `Consensus` trait
pub use crate::proof_of_cooperation::ProofOfCooperation;
