use soroban_sdk::{contracttype, Address, String, Vec};

use crate::guild::types::Role;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalType {
    TreasurySpend,
    AddMember,
    RemoveMember,
    RuleChange,
    GeneralDecision,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteDecision {
    For,
    Against,
    Abstain,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceConfig {
    pub quorum_percentage: u32,
    pub approval_threshold: u32,
    pub voting_period_days: u32,
    pub min_proposer_reputation: u32,
}

impl GovernanceConfig {
    pub fn default() -> Self {
        Self {
            quorum_percentage: 30,
            approval_threshold: 60,
            voting_period_days: 7,
            min_proposer_reputation: 0,
        }
    }
}

/// Simple execution payload for Soroban compatibility.
/// Complex payloads are stored as serialized strings or handled externally.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionPayload {
    /// Treasury spend: (treasury_id, amount, recipient_str, reason)
    TreasurySpend,
    /// Add member to guild
    AddMember,
    /// Remove member from guild
    RemoveMember,
    /// Change a guild rule
    RuleChange,
    /// General decision (signalling only)
    GeneralDecision,
}

/// Detailed payload data stored separately for complex operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasurySpendData {
    pub treasury_id: u64,
    pub amount: i128,
    pub token: Option<Address>,
    pub recipient: Address,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddMemberData {
    pub address: Address,
    pub role: Role,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoveMemberData {
    pub address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleChangeData {
    pub key: String,
    pub value: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneralDecisionData {
    pub meta: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub guild_id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub voting_start: u64,
    pub voting_end: u64,
    pub status: ProposalStatus,
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstain: i128,
    pub execution_payload: ExecutionPayload,
    pub passed_at: Option<u64>,
    pub executed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub voter: Address,
    pub proposal_id: u64,
    pub decision: VoteDecision,
    pub weight: i128,
    pub timestamp: u64,
}

// Events

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub guild_id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub decision: VoteDecision,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteDelegatedEvent {
    pub guild_id: u64,
    pub delegator: Address,
    pub delegate: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteUndelegatedEvent {
    pub guild_id: u64,
    pub delegator: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalFinalizedEvent {
    pub proposal_id: u64,
    pub status: ProposalStatus,
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstain: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalExecutedEvent {
    pub proposal_id: u64,
    pub success: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCancelledEvent {
    pub proposal_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernanceConfigUpdatedEvent {
    pub guild_id: u64,
}

pub fn role_weight(role: &Role) -> i128 {
    match role {
        Role::Owner => 10,
        Role::Admin => 5,
        Role::Member => 2,
        Role::Contributor => 1,
    }
}
