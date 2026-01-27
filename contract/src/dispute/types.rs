use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeReference {
    Bounty,
    Milestone,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Open,
    Voting,
    Resolved,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteDecision {
    FavorPlaintiff,
    FavorDefendant,
    Split,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub id: u64,
    pub reference_id: u64,
    pub reference_type: DisputeReference,
    pub guild_id: u64,
    pub plaintiff: Address,
    pub defendant: Address,
    pub reason: String,
    pub status: DisputeStatus,
    pub created_at: u64,
    pub voting_deadline: u64,
    pub evidence_plaintiff: Option<String>,
    pub evidence_defendant: Option<String>,
    pub votes_for_plaintiff: i128,
    pub votes_for_defendant: i128,
    pub votes_split: i128,
    pub vote_count: u32,
    pub resolved_at: Option<u64>,
    pub resolution_executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub voter: Address,
    pub dispute_id: u64,
    pub decision: VoteDecision,
    pub weight: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundDistribution {
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub winner: Option<Address>,
    pub fund_distribution: Vec<FundDistribution>,
    pub vote_count: u32,
    pub votes_for_plaintiff: i128,
    pub votes_for_defendant: i128,
    pub votes_split: i128,
    pub quorum_reached: bool,
}

// Events

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeCreatedEvent {
    pub dispute_id: u64,
    pub guild_id: u64,
    pub reference_id: u64,
    pub reference_type: DisputeReference,
    pub plaintiff: Address,
    pub defendant: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSubmittedEvent {
    pub dispute_id: u64,
    pub party: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCastEvent {
    pub dispute_id: u64,
    pub voter: Address,
    pub decision: VoteDecision,
    pub weight: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolvedEvent {
    pub dispute_id: u64,
    pub status: DisputeStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionExecutedEvent {
    pub dispute_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeExpiredEvent {
    pub dispute_id: u64,
}
