// File: crates/icn_governance/src/lib.rs

use icn_common::{IcnResult, IcnError};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
    CommunityInitiative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalCategory {
    Economic,
    Technical,
    Social,
    Environmental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub execution_deadline: Option<DateTime<Utc>>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub required_majority: f64,
    pub execution_threshold: Option<f64>,
    pub execution_delay: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub weight: f64,
    pub in_favor: bool,
    pub timestamp: DateTime<Utc>,
}

pub struct GovernanceSystem {
    proposals: Arc<RwLock<HashMap<String, Proposal>>>,
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,
}

impl GovernanceSystem {
    pub fn new() -> Self {
        GovernanceSystem {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_proposal(&self, proposal: Proposal) -> IcnResult<String> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for proposals".into()))?;
        
        if proposals.contains_key(&proposal.id) {
            return Err(IcnError::Governance("Proposal ID already exists".into()));
        }

        let proposal_id = proposal.id.clone();
        proposals.insert(proposal_id.clone(), proposal);
        self.votes.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for votes".into()))?.insert(proposal_id.clone(), Vec::new());

        Ok(proposal_id)
    }

    pub fn get_proposal(&self, proposal_id: &str) -> IcnResult<Proposal> {
        let proposals = self.proposals.read().map_err(|_| IcnError::Governance("Failed to acquire read lock for proposals".into()))?;
        proposals.get(proposal_id).cloned().ok_or_else(|| IcnError::Governance("Proposal not found".into()))
    }

    pub fn vote_on_proposal(&self, proposal_id: &str, voter: String, in_favor: bool, weight: f64) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for proposals".into()))?;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has ended".into()));
        }

        let mut votes = self.votes.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for votes".into()))?;
        let proposal_votes = votes.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        if proposal_votes.iter().any(|v| v.voter == voter) {
            return Err(IcnError::Governance("Voter has already voted on this proposal".into()));
        }

        proposal_votes.push(Vote {
            voter,
            weight,
            in_favor,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    pub fn finalize_proposal(&self, proposal_id: &str) -> IcnResult<ProposalStatus> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for proposals".into()))?;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(IcnError::Governance("Proposal is not active".into()));
        }

        let now = Utc::now();
        if now < proposal.voting_ends_at {
            return Err(IcnError::Governance("Voting period has not ended yet".into()));
        }

        let votes = self.votes.read().map_err(|_| IcnError::Governance("Failed to acquire read lock for votes".into()))?;
        let proposal_votes = votes.get(proposal_id).ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        let total_votes: f64 = proposal_votes.iter().map(|v| v.weight).sum();
        let votes_in_favor: f64 = proposal_votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        if total_votes < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
        } else if votes_in_favor / total_votes > proposal.required_majority {
            proposal.status = ProposalStatus::Passed;
            if let Some(delay) = proposal.execution_delay {
                proposal.execution_deadline = Some(now + delay);
            }
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(proposal.status.clone())
    }

    pub fn execute_proposal(&self, proposal_id: &str) -> IcnResult<()> {
        let mut proposals = self.proposals.write().map_err(|_| IcnError::Governance("Failed to acquire write lock for proposals".into()))?;
        let proposal = proposals.get_mut(proposal_id).ok_or_else(|| IcnError::Governance("Proposal not found".into()))?;

        if proposal.status != ProposalStatus::Passed {
            return Err(IcnError::Governance("Proposal has not passed".into()));
        }

        if let Some(deadline) = proposal.execution_deadline {
            if Utc::now() > deadline {
                proposal.status = ProposalStatus::Expired;
                return Err(IcnError::Governance("Proposal execution deadline has passed".into()));
            }
        }

        if let Some(threshold) = proposal.execution_threshold {
            let votes = self.votes.read().map_err(|_| IcnError::Governance("Failed to acquire read lock for votes".into()))?;
            let proposal_votes = votes.get(proposal_id).ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;
            let total_votes: f64 = proposal_votes.iter().map(|v| v.weight).sum();
            let votes_in_favor: f64 = proposal_votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

            if votes_in_favor / total_votes < threshold {
                return Err(IcnError::Governance("Execution threshold not met".into()));
            }
        }

        // Here you would implement the actual execution logic based on the proposal type
        match proposal.proposal_type {
            ProposalType::Constitutional => {
                // Implement constitutional change logic
            }
            ProposalType::EconomicAdjustment => {
                // Implement economic adjustment logic
            }
            ProposalType::NetworkUpgrade => {
                // Implement network upgrade logic
            }
            ProposalType::CommunityInitiative => {
                // Implement community initiative logic
            }
        }

        proposal.status = ProposalStatus::Executed;
        Ok(())
    }

    pub fn list_active_proposals(&self) -> IcnResult<Vec<Proposal>> {
        let proposals = self.proposals.read().map_err(|_| IcnError::Governance("Failed to acquire read lock for proposals".into()))?;
        Ok(proposals.values().filter(|p| p.status == ProposalStatus::Active).cloned().collect())
    }

    pub fn get_proposal_results(&self, proposal_id: &str) -> IcnResult<(f64, f64)> {
        let votes = self.votes.read().map_err(|_| IcnError::Governance("Failed to acquire read lock for votes".into()))?;
        let proposal_votes = votes.get(proposal_id).ok_or_else(|| IcnError::Governance("Votes not found for proposal".into()))?;

        let total_votes: f64 = proposal_votes.iter().map(|v| v.weight).sum();
        let votes_in_favor: f64 = proposal_votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        Ok((votes_in_favor, total_votes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_proposal() -> Proposal {
        Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            execution_deadline: Some(Utc::now() + Duration::days(14)),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Technical,
            required_quorum: 0.5,
            required_majority: 0.66,
            execution_threshold: Some(0.75),
            execution_delay: Some(Duration::days(1)),
        }
    }

    #[test]
    fn test_create_and_get_proposal() {
        let governance = GovernanceSystem::new();
        let proposal = create_test_proposal();
        let proposal_id = governance.create_proposal(proposal.clone()).unwrap();

        let retrieved_proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(retrieved_proposal.id, proposal.id);
        assert_eq!(retrieved_proposal.title, proposal.title);
    }

    #[test]
    fn test_vote_on_proposal() {
        let governance = GovernanceSystem::new();
        let proposal = create_test_proposal();
        let proposal_id = governance.create_proposal(proposal).unwrap();

        assert!(governance.vote_on_proposal(&proposal_id, "Bob".to_string(), true, 1.0).is_ok());
        assert!(governance.vote_on_proposal(&proposal_id, "Charlie".to_string(), false, 1.0).is_ok());

        // Test duplicate vote
        assert!(governance.vote_on_proposal(&proposal_id, "Bob".to_string(), false, 1.0).is_err());
    }

    #[test]
    fn test_finalize_proposal() {
        let governance = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.voting_ends_at = Utc::now() - Duration::hours(1);
        let proposal_id = governance.create_proposal(proposal).unwrap();

        governance.vote_on_proposal(&proposal_id, "Bob".to_string(), true, 3.0).unwrap();
        governance.vote_on_proposal(&proposal_id, "Charlie".to_string(), false, 1.0).unwrap();

        let status = governance.finalize_proposal(&proposal_id).unwrap();
        assert_eq!(status, ProposalStatus::Passed);

        let finalized_proposal = governance.get_proposal(&proposal_id).unwrap();
        assert!(finalized_proposal.execution_deadline.is_some());
    }

    #[test]
    fn test_execute_proposal() {
        let governance = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.voting_ends_at = Utc::now() - Duration::hours(2);
        proposal.status = ProposalStatus::Passed;
        let proposal_id = governance.create_proposal(proposal).unwrap();

        governance.vote_on_proposal(&proposal_id, "Bob".to_string(), true, 4.0).unwrap();
        governance.vote_on_proposal(&proposal_id, "Charlie".to_string(), true, 3.0).unwrap();
        governance.vote_on_proposal(&proposal_id, "David".to_string(), false, 2.0).unwrap();

        assert!(governance.execute_proposal(&proposal_id).is_ok());

        let executed_proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(executed_proposal.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_list_active_proposals() {
        let governance = GovernanceSystem::new();
        let active_proposal = create_test_proposal();
        let mut passed_proposal = create_test_proposal();
        passed_proposal.id = "passed_proposal".to_string();
        passed_proposal.status = ProposalStatus::Passed;

        governance.create_proposal(active_proposal).unwrap();
        governance.create_proposal(passed_proposal).unwrap();

        let active_proposals = governance.list_active_proposals().unwrap();
        assert_eq!(active_proposals.len(), 1);
        assert_eq!(active_proposals[0].status, ProposalStatus::Active);
    }

    #[test]
    fn test_get_proposal_results() {
        let governance = GovernanceSystem::new();
        let proposal = create_test_proposal();
        let proposal_id = governance.create_proposal(proposal).unwrap();

        governance.vote_on_proposal(&proposal_id, "Bob".to_string(), true, 3.0).unwrap();
        governance.vote_on_proposal(&proposal_id, "Charlie".to_string(), false, 2.0).unwrap();
        governance.vote_on_proposal(&proposal_id, "David".to_string(), true, 1.0).unwrap();

        let (votes_in_favor, total_votes) = governance.get_proposal_results(&proposal_id).unwrap();
        assert_eq!(votes_in_favor, 4.0);
        assert_eq!(total_votes, 6.0);
    }

    #[test]
    fn test_proposal_expiration() {
        let governance = GovernanceSystem::new();
        let mut proposal = create_test_proposal();
        proposal.voting_ends_at = Utc::now() - Duration::days(8);
        proposal.execution_deadline = Some(Utc::now() - Duration::days(1));
        proposal.status = ProposalStatus::Passed;
        let proposal_id = governance.create_proposal(proposal).unwrap();

        let result = governance.execute_proposal(&proposal_id);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "Governance error: Proposal execution deadline has passed");

        let expired_proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(expired_proposal.status, ProposalStatus::Expired);
    }
}