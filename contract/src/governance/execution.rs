use soroban_sdk::{Address, Env, String, Symbol};

use crate::governance::proposals::get_proposal as load_proposal;
use crate::governance::storage::store_proposal;
use crate::governance::types::{
    ExecutionPayload, Proposal, ProposalExecutedEvent, ProposalStatus, ProposalType,
};
use crate::governance::voting::finalize_proposal;
use crate::guild::membership::{add_member, remove_member};
use crate::guild::storage as guild_storage;
use crate::guild::types::Role;

const EXECUTION_DEADLINE_SECONDS: u64 = 3 * 24 * 60 * 60; // 3 days after passing

fn get_guild_owner(env: &Env, guild_id: u64) -> Address {
    let guild =
        guild_storage::get_guild(env, guild_id).unwrap_or_else(|| panic!("guild not found"));
    guild.owner
}

pub fn execute_proposal(env: &Env, proposal_id: u64) -> bool {
    let mut proposal = load_proposal(env, proposal_id);

    // auto-finalize if still active and voting period ended
    let now = env.ledger().timestamp();
    if matches!(proposal.status, ProposalStatus::Active) && now >= proposal.voting_end {
        let _status = finalize_proposal(env, proposal_id);
        proposal = load_proposal(env, proposal_id);
        if !matches!(proposal.status, ProposalStatus::Passed) {
            panic!("proposal not passed");
        }
    }

    if !matches!(proposal.status, ProposalStatus::Passed) {
        panic!("only passed proposals can be executed");
    }

    if let Some(passed_at) = proposal.passed_at {
        if now > passed_at + EXECUTION_DEADLINE_SECONDS {
            proposal.status = ProposalStatus::Expired;
            store_proposal(env, &proposal);
            panic!("execution window expired");
        }
    }

    let success = match (&proposal.proposal_type, &proposal.execution_payload) {
        (ProposalType::AddMember, ExecutionPayload::AddMember) => {
            // NOTE: With simplified ExecutionPayload, actual member data must be stored separately.
            // For now, this is a signalling-only execution.
            true
        }
        (ProposalType::RemoveMember, ExecutionPayload::RemoveMember) => {
            // NOTE: With simplified ExecutionPayload, actual member data must be stored separately.
            // For now, this is a signalling-only execution.
            true
        }
        (ProposalType::TreasurySpend, ExecutionPayload::TreasurySpend) => {
            // For now, only record that governance approved the spend.
            // Actual treasury movement should be done by a separate call using this payload.
            true
        }
        (ProposalType::RuleChange, ExecutionPayload::RuleChange) => {
            // No concrete rule storage defined yet; treat as signalling and emit event only.
            true
        }
        (ProposalType::GeneralDecision, ExecutionPayload::GeneralDecision) => {
            // Signalling-only proposals, always succeed when executed.
            true
        }
        _ => false,
    };

    let mut proposal_to_update: Proposal = proposal.clone();
    if success {
        proposal_to_update.status = ProposalStatus::Executed;
        proposal_to_update.executed_at = Some(now);
        store_proposal(env, &proposal_to_update);
    }

    let event = ProposalExecutedEvent {
        proposal_id,
        success,
    };
    env.events().publish(
        (
            Symbol::new(env, "proposal_executed"),
            Symbol::new(env, "v0"),
        ),
        event,
    );

    success
}
