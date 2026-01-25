use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::governance::storage::{
    get_config, get_guild_proposals, get_next_proposal_id, get_proposal as load_proposal,
    set_config, store_proposal,
};
use crate::governance::types::{
    ExecutionPayload, GovernanceConfig, GovernanceConfigUpdatedEvent, Proposal,
    ProposalCreatedEvent, ProposalStatus, ProposalType,
};
use crate::guild::storage as guild_storage;
use crate::guild::types::Member;

const EVENT_TOPIC_PROPOSAL_CREATED: &str = "proposal_created";
const EVENT_TOPIC_CONFIG_UPDATED: &str = "gov_config_updated";

fn validate_execution_payload(
    env: &Env,
    guild_id: u64,
    proposal_type: &ProposalType,
    payload: &ExecutionPayload,
) {
    // Validate that payload type matches proposal type
    match (proposal_type, payload) {
        (ProposalType::TreasurySpend, ExecutionPayload::TreasurySpend) => {}
        (ProposalType::AddMember, ExecutionPayload::AddMember) => {}
        (ProposalType::RemoveMember, ExecutionPayload::RemoveMember) => {}
        (ProposalType::RuleChange, ExecutionPayload::RuleChange) => {}
        (ProposalType::GeneralDecision, ExecutionPayload::GeneralDecision) => {}
        _ => {
            panic!("execution payload does not match proposal type");
        }
    }

    // Ensure guild exists
    let _guild =
        guild_storage::get_guild(env, guild_id).unwrap_or_else(|| panic!("guild not found"));
}

fn get_member(env: &Env, guild_id: u64, address: &Address) -> Option<Member> {
    guild_storage::get_member(env, guild_id, address)
}

fn ensure_guild_member(env: &Env, guild_id: u64, address: &Address) {
    if get_member(env, guild_id, address).is_none() {
        panic!("only guild members can perform this action");
    }
}

pub fn create_proposal(
    env: &Env,
    guild_id: u64,
    proposer: Address,
    proposal_type: ProposalType,
    title: String,
    description: String,
    execution_payload: ExecutionPayload,
) -> u64 {
    proposer.require_auth();

    // must be guild member
    let member = get_member(env, guild_id, &proposer)
        .unwrap_or_else(|| panic!("proposer must be a guild member"));

    let cfg: GovernanceConfig = get_config(env, guild_id);

    // proposer reputation based on role weight
    let reputation = crate::governance::types::role_weight(&member.role) as u32;
    if reputation < cfg.min_proposer_reputation {
        panic!("insufficient reputation to create proposal");
    }

    if title.len() == 0 || title.len() > 200 {
        panic!("proposal title length invalid");
    }

    if description.len() > 2000 {
        panic!("proposal description too long");
    }

    validate_execution_payload(env, guild_id, &proposal_type, &execution_payload);

    let id = get_next_proposal_id(env);
    let now = env.ledger().timestamp();
    let voting_period_secs = (cfg.voting_period_days as u64) * 24 * 60 * 60;

    let proposal = Proposal {
        id,
        guild_id,
        proposer: proposer.clone(),
        proposal_type: proposal_type.clone(),
        title,
        description,
        voting_start: now,
        voting_end: now + voting_period_secs,
        status: ProposalStatus::Active,
        votes_for: 0,
        votes_against: 0,
        votes_abstain: 0,
        execution_payload,
        passed_at: None,
        executed_at: None,
    };

    store_proposal(env, &proposal);

    let event = ProposalCreatedEvent {
        proposal_id: id,
        guild_id,
        proposer,
        proposal_type,
    };

    env.events().publish(
        (
            Symbol::new(env, EVENT_TOPIC_PROPOSAL_CREATED),
            Symbol::new(env, "v0"),
        ),
        event,
    );

    id
}

pub fn cancel_proposal(env: &Env, proposal_id: u64, canceller: Address) -> bool {
    canceller.require_auth();

    let mut proposal =
        load_proposal(env, proposal_id).unwrap_or_else(|| panic!("proposal not found"));

    if !matches!(
        proposal.status,
        ProposalStatus::Active | ProposalStatus::Draft
    ) {
        panic!("only active or draft proposals can be cancelled");
    }

    // allow proposer or guild owner to cancel
    let guild = guild_storage::get_guild(env, proposal.guild_id)
        .unwrap_or_else(|| panic!("guild not found"));

    if canceller != proposal.proposer && canceller != guild.owner {
        panic!("only proposer or guild owner can cancel");
    }

    proposal.status = ProposalStatus::Cancelled;
    store_proposal(env, &proposal);

    let event = crate::governance::types::ProposalCancelledEvent { proposal_id };
    env.events().publish(
        (
            Symbol::new(env, "proposal_cancelled"),
            Symbol::new(env, "v0"),
        ),
        event,
    );

    true
}

pub fn get_proposal(env: &Env, proposal_id: u64) -> Proposal {
    load_proposal(env, proposal_id).unwrap_or_else(|| panic!("proposal not found"))
}

pub fn get_active_proposals(env: &Env, guild_id: u64) -> Vec<Proposal> {
    let all = get_guild_proposals(env, guild_id);
    let mut active = Vec::new(env);
    for p in all.iter() {
        if matches!(p.status, ProposalStatus::Active) {
            active.push_back(p);
        }
    }
    active
}

pub fn update_governance_config(
    env: &Env,
    guild_id: u64,
    caller: Address,
    config: GovernanceConfig,
) -> bool {
    // only guild owner can update config
    let guild =
        guild_storage::get_guild(env, guild_id).unwrap_or_else(|| panic!("guild not found"));

    if caller != guild.owner {
        panic!("only guild owner can update governance config");
    }
    caller.require_auth();

    if config.quorum_percentage == 0 || config.quorum_percentage > 100 {
        panic!("invalid quorum percentage");
    }
    if config.approval_threshold == 0 || config.approval_threshold > 100 {
        panic!("invalid approval threshold");
    }

    set_config(env, guild_id, &config);

    let event = GovernanceConfigUpdatedEvent { guild_id };
    env.events().publish(
        (
            Symbol::new(env, EVENT_TOPIC_CONFIG_UPDATED),
            Symbol::new(env, "v0"),
        ),
        event,
    );

    true
}
