pub mod storage;
pub mod tracker;
/// Milestone tracking module
///
/// This module provides project and milestone management for long-running
/// work with phased payments. It integrates with the treasury module using
/// `TransactionType::MilestonePayment` for accounting.
///
/// - `types`: Core data structures and events
/// - `storage`: Persistent storage helpers
/// - `tracker`: Core milestone lifecycle and payment logic
pub mod types;

// Re-export main functions
pub use tracker::{
    add_milestone, approve_milestone, cancel_project, create_project, extend_milestone_deadline,
    get_milestone_view, get_project_progress, reject_milestone, release_milestone_payment,
    start_milestone, submit_milestone,
};
pub use types::{Milestone, MilestoneInput, MilestoneStatus, Project, ProjectStatus};

// Tests are disabled pending treasury integration
#[cfg(test)]
mod tests;
