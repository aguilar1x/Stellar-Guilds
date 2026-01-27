//! Dispute Resolution Module
//!
//! Provides dispute creation, evidence submission, voting, and resolution
//! for bounties and milestones with weighted guild voting.

pub mod resolution;
pub mod storage;
pub mod types;
pub mod voting;

use soroban_sdk::{Address, Env, String};

use crate::bounty::storage as bounty_storage;
use crate::bounty::types::BountyStatus;
use crate::dispute::resolution as dispute_resolution;
use crate::dispute::storage as dispute_storage;
use crate::dispute::types::{
    Dispute, DisputeCreatedEvent, DisputeReference, DisputeStatus, EvidenceSubmittedEvent,
};
use crate::milestone::storage as milestone_storage;
use crate::milestone::types::ProjectStatus;

const VOTING_PERIOD_SECONDS: u64 = 7 * 24 * 60 * 60;
const MAX_REASON_LEN: u32 = 1024;
const MAX_EVIDENCE_LEN: u32 = 1024;

/// Create a new dispute tied to a bounty or milestone reference.
///
/// # Arguments
/// * `reference_id` - Bounty or milestone ID
/// * `plaintiff` - Address opening the dispute
/// * `defendant` - Address responding to the dispute
/// * `reason` - Short reason for the dispute
/// * `evidence_url` - Initial evidence URL from plaintiff
pub fn create_dispute(
    env: &Env,
    reference_id: u64,
    plaintiff: Address,
    defendant: Address,
    reason: String,
    evidence_url: String,
) -> u64 {
    plaintiff.require_auth();

    if plaintiff == defendant {
        panic!("plaintiff and defendant must differ");
    }

    if reason.len() == 0 || reason.len() > MAX_REASON_LEN {
        panic!("invalid reason length");
    }

    if evidence_url.len() == 0 || evidence_url.len() > MAX_EVIDENCE_LEN {
        panic!("invalid evidence url");
    }

    let bounty = bounty_storage::get_bounty(env, reference_id);
    let milestone = milestone_storage::get_milestone(env, reference_id);

    let (reference_type, guild_id) = match (bounty, milestone) {
        (Some(b), None) => {
            if b.status == BountyStatus::Cancelled || b.status == BountyStatus::Expired {
                panic!("bounty not disputable");
            }
            if b.funded_amount <= 0 {
                panic!("bounty has no locked funds");
            }
            (DisputeReference::Bounty, b.guild_id)
        }
        (None, Some(m)) => {
            let project = milestone_storage::get_project(env, m.project_id)
                .expect("project not found");
            if project.status == ProjectStatus::Cancelled {
                panic!("project cancelled");
            }
            if m.is_payment_released {
                panic!("milestone already paid");
            }
            (DisputeReference::Milestone, project.guild_id)
        }
        (Some(_), Some(_)) => panic!("ambiguous reference id"),
        (None, None) => panic!("reference not found"),
    };

    if dispute_storage::is_reference_locked(env, &reference_type, reference_id) {
        panic!("dispute already active for reference");
    }

    let now = env.ledger().timestamp();
    let voting_deadline = now + VOTING_PERIOD_SECONDS;
    let dispute_id = dispute_storage::get_next_dispute_id(env);

    let dispute = Dispute {
        id: dispute_id,
        reference_id,
        reference_type: reference_type.clone(),
        guild_id,
        plaintiff: plaintiff.clone(),
        defendant: defendant.clone(),
        reason,
        status: DisputeStatus::Open,
        created_at: now,
        voting_deadline,
        evidence_plaintiff: Some(evidence_url),
        evidence_defendant: None,
        votes_for_plaintiff: 0,
        votes_for_defendant: 0,
        votes_split: 0,
        vote_count: 0,
        resolved_at: None,
        resolution_executed: false,
    };

    dispute_storage::store_dispute(env, &dispute);
    dispute_storage::lock_reference(env, &reference_type, reference_id, dispute_id);

    let event = DisputeCreatedEvent {
        dispute_id,
        guild_id,
        reference_id,
        reference_type,
        plaintiff,
        defendant,
    };
    env.events().publish(("DisputeCreated",), event);

    dispute_id
}

/// Submit evidence for an active dispute.
///
/// Evidence can only be submitted by the plaintiff or defendant
/// during the active voting window.
pub fn submit_evidence(
    env: &Env,
    dispute_id: u64,
    party: Address,
    evidence_url: String,
) -> bool {
    party.require_auth();

    if evidence_url.len() == 0 || evidence_url.len() > MAX_EVIDENCE_LEN {
        panic!("invalid evidence url");
    }

    let mut dispute = dispute_storage::get_dispute(env, dispute_id).expect("dispute not found");
    if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Expired {
        panic!("dispute closed");
    }

    let now = env.ledger().timestamp();
    if now > dispute.voting_deadline {
        panic!("evidence period ended");
    }

    if party == dispute.plaintiff {
        dispute.evidence_plaintiff = Some(evidence_url);
    } else if party == dispute.defendant {
        dispute.evidence_defendant = Some(evidence_url);
    } else {
        panic!("only parties can submit evidence");
    }

    dispute_storage::store_dispute(env, &dispute);

    let event = EvidenceSubmittedEvent {
        dispute_id,
        party,
    };
    env.events().publish(("DisputeEvidence",), event);

    true
}

/// Cast a weighted vote for a dispute.
pub fn cast_vote(
    env: &Env,
    dispute_id: u64,
    voter: Address,
    decision: crate::dispute::types::VoteDecision,
) -> bool {
    voting::cast_vote(env, dispute_id, voter, decision)
}

/// Calculate a voter's weight based on their guild role.
pub fn calculate_vote_weight(env: &Env, guild_id: u64, voter: Address) -> u32 {
    voting::calculate_vote_weight(env, guild_id, &voter)
}

/// Tally votes for a dispute and return the resolution summary.
pub fn tally_votes(env: &Env, dispute_id: u64) -> crate::dispute::types::Resolution {
    dispute_resolution::tally_votes(env, dispute_id)
}

/// Resolve a dispute after the voting deadline and execute fund distribution.
pub fn resolve_dispute(env: &Env, dispute_id: u64) -> crate::dispute::types::Resolution {
    dispute_resolution::resolve_dispute(env, dispute_id)
}

/// Execute the fund distribution for a resolved dispute.
pub fn execute_resolution(
    env: &Env,
    dispute_id: u64,
) -> soroban_sdk::Vec<crate::dispute::types::FundDistribution> {
    dispute_resolution::execute_resolution(env, dispute_id)
}

#[cfg(test)]
mod tests;
