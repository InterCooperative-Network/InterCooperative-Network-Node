// File: icn_consensus/src/lib.rs

//! This module defines the consensus mechanisms for the InterCooperative Network (ICN) project.
//! It includes traits and structures for implementing various consensus algorithms.

pub mod consensus;
pub mod proof_of_cooperation;

pub use crate::consensus::Consensus;
pub use crate::proof_of_cooperation::ProofOfCooperation;
