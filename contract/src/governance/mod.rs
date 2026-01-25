pub mod types;
pub mod storage;
pub mod proposals;
pub mod voting;
pub mod execution;

pub use types::{
    ExecutionPayload,
    GovernanceConfig,
    Proposal,
    ProposalStatus,
    ProposalType,
    VoteDecision,
};

pub use proposals::{
    cancel_proposal,
    create_proposal,
    get_active_proposals,
    get_proposal,
    update_governance_config,
};

pub use voting::{
    delegate_vote,
    finalize_proposal,
    undelegate_vote,
    vote,
};

pub use execution::execute_proposal;

#[cfg(test)]
mod tests;
