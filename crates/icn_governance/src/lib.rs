// File: crates/icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError, ProposalStatus, ProposalType, ProposalCategory, CurrencyType};
use icn_identity::DecentralizedIdentity;
use icn_reputation::ReputationSystem;
use icn_currency::CurrencySystem;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents an enhanced proposal in the InterCooperative Network governance system.
/// This structure includes all necessary information for a proposal, including
/// its current status and potential resource allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: DecentralizedIdentity,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
    pub resource_allocation: Option<ResourceAllocation>,
}

/// Defines the resource allocation associated with a proposal.
/// This structure is used when a proposal involves distributing resources
/// to beneficiaries upon approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub resource_type: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub beneficiaries: Vec<DecentralizedIdentity>,
}

/// Represents a weighted vote cast by a member of the network.
/// The weight of the vote is determined by the voter's reputation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedVote {
    pub voter: DecentralizedIdentity,
    pub weight: f64,
    pub in_favor: bool,
    pub timestamp: DateTime<Utc>,
}

/// The main structure for the governance system.
/// It manages proposals, votes, and interacts with the reputation and currency systems.
pub struct GovernanceSystem {
    proposals: HashMap<String, EnhancedProposal>,
    votes: HashMap<String, Vec<WeightedVote>>,
    reputation_system: Arc<RwLock<ReputationSystem>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
}

impl GovernanceSystem {
    /// Creates a new instance of the GovernanceSystem.
    ///
    /// # Arguments
    ///
    /// * `reputation_system` - A shared reference to the reputation system
    /// * `currency_system` - A shared reference to the currency system
    ///
    /// # Returns
    ///
    /// A new GovernanceSystem instance
    pub fn new(
        reputation_system: Arc<RwLock<ReputationSystem>>,
        currency_system: Arc<RwLock<CurrencySystem>>
    ) -> Self {
        GovernanceSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            reputation_system,
            currency_system,
        }
    }

    /// Creates a new proposal in the governance system.
    ///
    /// # Arguments
    ///
    /// * `proposal` - The EnhancedProposal to be added to the system
    ///
    /// # Returns
    ///
    /// The ID of the newly created proposal, or an error if creation fails
    pub async fn create_proposal(&mut self, proposal: EnhancedProposal) -> IcnResult<String> {
        if self.proposals.contains_key(&proposal.id) {
            return Err(IcnError::Governance("Proposal ID already exists".into()));
        }
        let proposal_id = proposal.id.clone();
        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), Vec::new());
        Ok(proposal_id)
    }

    /// Registers a vote for a specific proposal.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The ID of the proposal being voted on
    /// * `voter` - The DecentralizedIdentity of the voter
    /// * `in_favor` - Whether the vote is in favor of the proposal
    ///
    /// # Returns
    ///
    /// Ok(()) if the vote was successfully registered, or an error otherwise
    pub async fn vote(&mut self, proposal_id: &str, voter: DecentralizedIdentity, in_favor: bool) -> IcnResult<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        // Check if the proposal is still active and within the voting period
        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }
        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".into()));
        }

        // Calculate the vote weight based on the voter's reputation
        let reputation = self.reputation_system.read().await.get_reputation(&voter.id).await?;
        let weight = self.calculate_vote_weight(reputation);

        let weighted_vote = WeightedVote {
            voter,
            weight,
            in_favor,
            timestamp: Utc::now(),
        };

        // Add the vote to the proposal's vote collection
        let votes = self.votes.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        // Remove any previous vote by this voter
        votes.retain(|v| v.voter.id != weighted_vote.voter.id);
        votes.push(weighted_vote);

        Ok(())
    }

    /// Finalizes a proposal after its voting period has ended.
    ///
    /// # Arguments
    ///
    /// * `proposal_id` - The ID of the proposal to finalize
    ///
    /// # Returns
    ///
    /// The final status of the proposal, or an error if finalization fails
    pub async fn finalize_proposal(&mut self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }
        if Utc::now() < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".into()));
        }

        let votes = self.votes.get(proposal_id)
            .ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        // Calculate the total weight and weight in favor
        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let weight_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        // Determine the proposal outcome
        if total_weight < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
        } else if weight_in_favor / total_weight > 0.5 {
            proposal.status = ProposalStatus::Passed;
            // Trigger resource allocation if applicable
            if let Some(resource_allocation) = &proposal.resource_allocation {
                self.allocate_resources(proposal_id, resource