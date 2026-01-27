use soroban_sdk::{Address, Env, Vec};

use crate::bounty::escrow::release_funds;
use crate::bounty::storage as bounty_storage;
use crate::bounty::types::BountyStatus;
use crate::dispute::storage;
use crate::dispute::types::{
    DisputeReference, DisputeStatus, FundDistribution, Resolution, VoteDecision,
};
use crate::guild::storage as guild_storage;
use crate::milestone::storage as milestone_storage;
use crate::milestone::types::{MilestoneStatus, ProjectStatus};
use crate::treasury::execute_milestone_payment;

const QUORUM_PERCENTAGE: u32 = 30;

fn quorum_reached(env: &Env, guild_id: u64, vote_count: u32) -> bool {
    let members = guild_storage::get_all_members(env, guild_id);
    let total = members.len() as u32;
    if total == 0 {
        return false;
    }

    vote_count.saturating_mul(100) / total >= QUORUM_PERCENTAGE
}

fn decide_winner(
    votes_for_plaintiff: i128,
    votes_for_defendant: i128,
    votes_split: i128,
) -> VoteDecision {
    if votes_for_plaintiff > votes_for_defendant && votes_for_plaintiff > votes_split {
        VoteDecision::FavorPlaintiff
    } else if votes_for_defendant > votes_for_plaintiff && votes_for_defendant > votes_split {
        VoteDecision::FavorDefendant
    } else {
        VoteDecision::Split
    }
}

/// Tally votes for a dispute without mutating state.
pub fn tally_votes(env: &Env, dispute_id: u64) -> Resolution {
    let dispute = storage::get_dispute(env, dispute_id).expect("dispute not found");
    let quorum = quorum_reached(env, dispute.guild_id, dispute.vote_count);

    let winner = if quorum {
        match decide_winner(
            dispute.votes_for_plaintiff,
            dispute.votes_for_defendant,
            dispute.votes_split,
        ) {
            VoteDecision::FavorPlaintiff => Some(dispute.plaintiff.clone()),
            VoteDecision::FavorDefendant => Some(dispute.defendant.clone()),
            VoteDecision::Split => None,
        }
    } else {
        None
    };

    Resolution {
        winner,
        fund_distribution: Vec::new(env),
        vote_count: dispute.vote_count,
        votes_for_plaintiff: dispute.votes_for_plaintiff,
        votes_for_defendant: dispute.votes_for_defendant,
        votes_split: dispute.votes_split,
        quorum_reached: quorum,
    }
}

/// Resolve a dispute after the voting deadline and execute payouts.
pub fn resolve_dispute(env: &Env, dispute_id: u64) -> Resolution {
    let mut dispute = storage::get_dispute(env, dispute_id).expect("dispute not found");

    if dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Expired {
        panic!("dispute already closed");
    }

    let now = env.ledger().timestamp();
    if now < dispute.voting_deadline {
        panic!("voting period still active");
    }

    let mut resolution = tally_votes(env, dispute_id);

    if !resolution.quorum_reached {
        dispute.status = DisputeStatus::Expired;
        dispute.resolved_at = Some(now);
        storage::store_dispute(env, &dispute);
        storage::unlock_reference(env, &dispute.reference_type, dispute.reference_id);

        // Refund rules for expired disputes
        if dispute.reference_type == DisputeReference::Bounty {
            let mut bounty = bounty_storage::get_bounty(env, dispute.reference_id)
                .expect("bounty not found");

            if bounty.status != BountyStatus::Cancelled
                && bounty.status != BountyStatus::Expired
                && bounty.funded_amount > 0
            {
                release_funds(env, &bounty.token, &bounty.creator, bounty.funded_amount);
                bounty.funded_amount = 0;
                bounty_storage::store_bounty(env, &bounty);
            }
        }

        let event = crate::dispute::types::DisputeExpiredEvent { dispute_id };
        env.events().publish(("DisputeExpired",), event);

        return resolution;
    }

    dispute.status = DisputeStatus::Resolved;
    dispute.resolved_at = Some(now);
    storage::store_dispute(env, &dispute);
    storage::unlock_reference(env, &dispute.reference_type, dispute.reference_id);

    let event = crate::dispute::types::DisputeResolvedEvent {
        dispute_id,
        status: dispute.status.clone(),
    };
    env.events().publish(("DisputeResolved",), event);

    let distributions = execute_resolution(env, dispute_id);
    resolution.fund_distribution = distributions;

    resolution
}

/// Execute fund redistribution for a resolved dispute.
pub fn execute_resolution(env: &Env, dispute_id: u64) -> Vec<FundDistribution> {
    let mut dispute = storage::get_dispute(env, dispute_id).expect("dispute not found");

    if dispute.status != DisputeStatus::Resolved {
        panic!("dispute not resolved");
    }

    if dispute.resolution_executed {
        panic!("resolution already executed");
    }

    let decision = decide_winner(
        dispute.votes_for_plaintiff,
        dispute.votes_for_defendant,
        dispute.votes_split,
    );

    let mut distributions = Vec::new(env);

    match dispute.reference_type {
        DisputeReference::Bounty => {
            let mut bounty = bounty_storage::get_bounty(env, dispute.reference_id)
                .expect("bounty not found");

            let total = bounty.funded_amount;
            if total > 0 {
                match decision {
                    VoteDecision::FavorPlaintiff => {
                        release_funds(env, &bounty.token, &dispute.plaintiff, total);
                        distributions.push_back(FundDistribution {
                            recipient: dispute.plaintiff.clone(),
                            amount: total,
                        });
                    }
                    VoteDecision::FavorDefendant => {
                        release_funds(env, &bounty.token, &dispute.defendant, total);
                        distributions.push_back(FundDistribution {
                            recipient: dispute.defendant.clone(),
                            amount: total,
                        });
                    }
                    VoteDecision::Split => {
                        let half = total / 2;
                        let remainder = total - half;
                        release_funds(env, &bounty.token, &dispute.plaintiff, half);
                        release_funds(env, &bounty.token, &dispute.defendant, remainder);
                        distributions.push_back(FundDistribution {
                            recipient: dispute.plaintiff.clone(),
                            amount: half,
                        });
                        distributions.push_back(FundDistribution {
                            recipient: dispute.defendant.clone(),
                            amount: remainder,
                        });
                    }
                }

                bounty.funded_amount = 0;
                bounty_storage::store_bounty(env, &bounty);
            }
        }
        DisputeReference::Milestone => {
            let mut milestone = milestone_storage::get_milestone(env, dispute.reference_id)
                .expect("milestone not found");
            let mut project = milestone_storage::get_project(env, milestone.project_id)
                .expect("project not found");

            let total = milestone.payment_amount;
            if total > 0 {
                let (plaintiff_amt, defendant_amt) = match decision {
                    VoteDecision::FavorPlaintiff => (total, 0),
                    VoteDecision::FavorDefendant => (0, total),
                    VoteDecision::Split => {
                        let half = total / 2;
                        (half, total - half)
                    }
                };

                let new_released = project
                    .released_amount
                    .checked_add(total)
                    .expect("overflow");
                if new_released > project.total_amount {
                    panic!("project budget exceeded");
                }

                if plaintiff_amt > 0 {
                    execute_milestone_payment(
                        env,
                        project.treasury_id,
                        project.token.clone(),
                        dispute.plaintiff.clone(),
                        plaintiff_amt,
                    );
                    distributions.push_back(FundDistribution {
                        recipient: dispute.plaintiff.clone(),
                        amount: plaintiff_amt,
                    });
                }
                if defendant_amt > 0 {
                    execute_milestone_payment(
                        env,
                        project.treasury_id,
                        project.token.clone(),
                        dispute.defendant.clone(),
                        defendant_amt,
                    );
                    distributions.push_back(FundDistribution {
                        recipient: dispute.defendant.clone(),
                        amount: defendant_amt,
                    });
                }

                project.released_amount = new_released;
                milestone.is_payment_released = true;
                milestone.last_updated_at = env.ledger().timestamp();

                milestone.status = match decision {
                    VoteDecision::FavorDefendant => MilestoneStatus::Rejected,
                    _ => MilestoneStatus::Approved,
                };

                milestone_storage::store_project(env, &project);
                milestone_storage::store_milestone(env, &milestone);

                let ids = milestone_storage::get_project_milestone_ids(env, project.id);
                let mut all_done = true;
                for id in ids.iter() {
                    if let Some(m) = milestone_storage::get_milestone(env, id) {
                        if !m.is_payment_released && m.status != MilestoneStatus::Expired {
                            all_done = false;
                            break;
                        }
                    }
                }

                if all_done && project.status != ProjectStatus::Completed {
                    project.status = ProjectStatus::Completed;
                    milestone_storage::store_project(env, &project);
                }
            }
        }
    }

    dispute.resolution_executed = true;
    storage::store_dispute(env, &dispute);

    let event = crate::dispute::types::ResolutionExecutedEvent { dispute_id };
    env.events().publish(("ResolutionExecuted",), event);

    distributions
}
