use soroban_sdk::{Env};

use crate::dispute::storage;
use crate::dispute::types::{DisputeStatus, Vote, VoteDecision};
use crate::guild::storage as guild_storage;
use crate::governance::types::role_weight;

/// Calculate voting weight for a guild member based on role.
pub fn calculate_vote_weight(env: &Env, guild_id: u64, voter: &soroban_sdk::Address) -> u32 {
    let member = guild_storage::get_member(env, guild_id, voter)
        .unwrap_or_else(|| panic!("voter must be guild member"));

    let weight = role_weight(&member.role);
    if weight < 0 {
        0
    } else {
        weight as u32
    }
}

/// Cast a weighted vote for a dispute.
pub fn cast_vote(
    env: &Env,
    dispute_id: u64,
    voter: soroban_sdk::Address,
    decision: VoteDecision,
) -> bool {
    voter.require_auth();

    let mut dispute = storage::get_dispute(env, dispute_id).expect("dispute not found");

    if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Expired {
        panic!("dispute is closed");
    }

    let now = env.ledger().timestamp();
    if now > dispute.voting_deadline {
        panic!("voting period ended");
    }

    if voter == dispute.plaintiff || voter == dispute.defendant {
        panic!("parties cannot vote");
    }

    if storage::get_vote(env, dispute_id, &voter).is_some() {
        panic!("voter already voted");
    }

    // ensure voter is a guild member
    let _member = guild_storage::get_member(env, dispute.guild_id, &voter)
        .unwrap_or_else(|| panic!("voter must be guild member"));

    let weight = calculate_vote_weight(env, dispute.guild_id, &voter) as i128;

    let vote = Vote {
        voter: voter.clone(),
        dispute_id,
        decision: decision.clone(),
        weight,
        timestamp: now,
    };

    storage::store_vote(env, &vote);

    // Update tallies for gas-optimized tallying
    match decision {
        VoteDecision::FavorPlaintiff => dispute.votes_for_plaintiff += weight,
        VoteDecision::FavorDefendant => dispute.votes_for_defendant += weight,
        VoteDecision::Split => dispute.votes_split += weight,
    }
    dispute.vote_count = dispute.vote_count.saturating_add(1);

    if dispute.status == DisputeStatus::Open {
        dispute.status = DisputeStatus::Voting;
    }

    storage::store_dispute(env, &dispute);

    let event = crate::dispute::types::VoteCastEvent {
        dispute_id,
        voter,
        decision,
        weight,
    };
    env.events().publish(("DisputeVote",), event);

    true
}
