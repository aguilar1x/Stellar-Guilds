pub mod distribution;
pub mod storage;
/// Payment distribution module
///
/// This module provides automated payment splitting functionality for the Stellar Guilds platform.
/// It enables fair distribution of rewards among multiple contributors based on configurable rules.
///
/// # Overview
/// - `types`: Defines all core data structures and events
/// - `storage`: Manages persistent storage of payment pools and recipients
/// - `distribution`: Core functions for payment pool management and distribution
///
/// # Distribution Rules
/// - **Percentage**: Recipients get fixed percentage shares (must sum to 100%)
/// - **EqualSplit**: All recipients get equal shares
/// - **Weighted**: Recipients get shares proportional to their weights
///
/// # Key Features
/// - Atomic distribution execution
/// - Support for native XLM and custom tokens
/// - Dust amount protection
/// - Comprehensive validation
/// - Event emission for transparency
pub mod types;

// Re-export main functions for convenience
pub use distribution::{
    add_recipient, batch_distribute, cancel_distribution, create_payment_pool,
    execute_distribution, get_pool_status, get_recipient_amount, validate_distribution,
};
// pub use storage::initialize_payment_storage;
pub use types::{DistributionRule, DistributionStatus};

#[cfg(test)]
mod tests;
