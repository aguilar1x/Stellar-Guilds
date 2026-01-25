use soroban_sdk::{contracttype, Address, String};

/// Status of a bounty lifecycle
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BountyStatus {
    Open = 0,
    Claimed = 1,
    UnderReview = 2,
    Completed = 3,
    Cancelled = 4,
    Expired = 5,
    AwaitingFunds = 6,
}

/// Bounty struct containing all bounty metadata and state
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bounty {
    /// Unique identifier for the bounty
    pub id: u64,
    /// ID of the guild this bounty belongs to
    pub guild_id: u64,
    /// Address of the creator (must be guild admin/owner)
    pub creator: Address,
    /// Short title metadata
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Amount of tokens defined as reward
    pub reward_amount: i128,
    /// Amount of tokens currently funded
    pub funded_amount: i128,
    /// Address of the token contract (XLM or custom)
    pub token: Address,
    /// Current status of the bounty
    pub status: BountyStatus,
    /// Address of the contributor who claimed the bounty (optional)
    pub claimer: Option<Address>,
    /// Submission URL when work is submitted
    pub submission_url: Option<String>,
    /// Creation timestamp (seconds)
    pub created_at: u64,
    /// Expiration timestamp (seconds)
    pub expires_at: u64,
}

// ============ Events ============

/// Event emitted when a bounty is created
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyCreatedEvent {
    pub bounty_id: u64,
    pub guild_id: u64,
    pub creator: Address,
    pub reward_amount: i128,
    pub token: Address,
    pub expires_at: u64,
}

/// Event emitted when a bounty is funded
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyFundedEvent {
    pub bounty_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub total_funded: i128,
    pub is_fully_funded: bool,
}

/// Event emitted when a bounty is claimed
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyClaimedEvent {
    pub bounty_id: u64,
    pub claimer: Address,
}

/// Event emitted when work is submitted
#[contracttype]
#[derive(Clone, Debug)]
pub struct WorkSubmittedEvent {
    pub bounty_id: u64,
    pub claimer: Address,
    pub submission_url: String,
}

/// Event emitted when work is approved
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyApprovedEvent {
    pub bounty_id: u64,
    pub approver: Address,
}

/// Event emitted when escrow is released
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowReleasedEvent {
    pub bounty_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub token: Address,
}

/// Event emitted when a bounty is cancelled
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyCancelledEvent {
    pub bounty_id: u64,
    pub canceller: Address,
    pub refund_amount: i128,
    pub refund_recipient: Address,
}

/// Event emitted when a bounty expires
#[contracttype]
#[derive(Clone, Debug)]
pub struct BountyExpiredEvent {
    pub bounty_id: u64,
}
