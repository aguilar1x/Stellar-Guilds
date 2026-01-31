//! Bounty Escrow Module
//!
//! This module handles bounty creation, funding, claiming, and escrow management
//! for the Stellar Guilds platform. It enables trustless payments by locking funds
//! in escrow until work is verified and approved.
//!
//! # Features
//! - Bounty creation with metadata and rewards
//! - Escrow funding from any address
//! - First-come-first-served bounty claiming
//! - Work submission and approval flow
//! - Automatic escrow release on completion
//! - Cancellation with refund support
//! - Expiration handling

pub mod escrow;
pub mod storage;
pub mod types;

use crate::bounty::escrow::{lock_funds, release_funds};
use crate::bounty::storage::{get_bounty, get_guild_bounties, get_next_bounty_id, store_bounty};
use crate::bounty::types::{
    BountyApprovedEvent, BountyCancelledEvent, BountyClaimedEvent, BountyCreatedEvent,
    BountyExpiredEvent, BountyFundedEvent, EscrowReleasedEvent, WorkSubmittedEvent,
};
use crate::guild::membership::has_permission;
use crate::guild::types::Role;
use soroban_sdk::{Address, Env, String, Symbol, Vec};

// Re-export types for external use
pub use types::{Bounty, BountyStatus};

/// Create a new bounty
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild creating the bounty
/// * `creator` - Address of the bounty creator (must be guild admin/owner)
/// * `title` - Short title for the bounty
/// * `description` - Detailed description of the task
/// * `reward_amount` - Amount of tokens as reward
/// * `token` - Address of the token contract (XLM wrapped or custom Stellar asset)
/// * `expiry` - Absolute timestamp when the bounty expires
///
/// # Returns
/// The ID of the newly created bounty
///
/// # Panics
/// - If creator is not a guild admin or owner
/// - If reward_amount is negative
/// - If expiry is in the past
pub fn create_bounty(
    env: &Env,
    guild_id: u64,
    creator: Address,
    title: String,
    description: String,
    reward_amount: i128,
    token: Address,
    expiry: u64,
) -> u64 {
    creator.require_auth();

    // Verify creator has Admin or Owner permissions in the guild
    if !has_permission(env, guild_id, creator.clone(), Role::Admin) {
        panic!("Unauthorized: Creator must be a guild admin or owner");
    }

    // Validate inputs
    if reward_amount < 0 {
        panic!("Invalid reward amount: must be non-negative");
    }

    let created_at = env.ledger().timestamp();
    if expiry <= created_at {
        panic!("Expiry must be in the future");
    }

    if title.len() == 0 || title.len() > 256 {
        panic!("Title must be between 1 and 256 characters");
    }

    if description.len() > 2048 {
        panic!("Description must be at most 2048 characters");
    }

    let bounty_id = get_next_bounty_id(env);

    // Determine initial status based on reward amount
    let status = if reward_amount == 0 {
        BountyStatus::Open // Zero-reward bounties are immediately open
    } else {
        BountyStatus::AwaitingFunds
    };

    let bounty = Bounty {
        id: bounty_id,
        guild_id,
        creator: creator.clone(),
        title,
        description,
        reward_amount,
        funded_amount: 0,
        token: token.clone(),
        status,
        claimer: None,
        submission_url: None,
        created_at,
        expires_at: expiry,
    };

    store_bounty(env, &bounty);

    // Emit creation event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "created")),
        BountyCreatedEvent {
            bounty_id,
            guild_id,
            creator,
            reward_amount,
            token,
            expires_at: expiry,
        },
    );

    bounty_id
}

/// Fund a bounty with tokens
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty to fund
/// * `funder` - Address providing the funds
/// * `amount` - Amount of tokens to fund
///
/// # Returns
/// `true` if funding was successful
///
/// # Panics
/// - If bounty is not found
/// - If amount is not positive
/// - If bounty is not in a fundable state
pub fn fund_bounty(env: &Env, bounty_id: u64, funder: Address, amount: i128) -> bool {
    funder.require_auth();

    if amount <= 0 {
        panic!("Amount must be positive");
    }

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Check for expiration
    let now = env.ledger().timestamp();
    if now > bounty.expires_at {
        bounty.status = BountyStatus::Expired;
        store_bounty(env, &bounty);
        env.events().publish(
            (Symbol::new(env, "bounty"), Symbol::new(env, "expired")),
            BountyExpiredEvent { bounty_id },
        );
        panic!("Bounty has expired");
    }

    // Can only fund if awaiting funds or open (partial funding support)
    match bounty.status {
        BountyStatus::AwaitingFunds | BountyStatus::Open => {}
        _ => panic!("Bounty cannot be funded in current status"),
    }

    // Transfer tokens to contract (escrow)
    lock_funds(env, &bounty.token, &funder, amount);

    // Update funded amount
    bounty.funded_amount += amount;

    let is_fully_funded = bounty.funded_amount >= bounty.reward_amount;

    // Transition to Open if fully funded
    if is_fully_funded && bounty.status == BountyStatus::AwaitingFunds {
        bounty.status = BountyStatus::Open;
    }

    store_bounty(env, &bounty);

    // Emit funding event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "funded")),
        BountyFundedEvent {
            bounty_id,
            funder,
            amount,
            total_funded: bounty.funded_amount,
            is_fully_funded,
        },
    );

    true
}

/// Claim a bounty (first-come-first-served)
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty to claim
/// * `claimer` - Address of the contributor claiming the bounty
///
/// # Returns
/// `true` if claiming was successful
///
/// # Panics
/// - If bounty is not found
/// - If bounty is not open for claiming
/// - If bounty has expired
pub fn claim_bounty(env: &Env, bounty_id: u64, claimer: Address) -> bool {
    claimer.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Check for expiration
    let now = env.ledger().timestamp();
    if now > bounty.expires_at {
        bounty.status = BountyStatus::Expired;
        store_bounty(env, &bounty);
        env.events().publish(
            (Symbol::new(env, "bounty"), Symbol::new(env, "expired")),
            BountyExpiredEvent { bounty_id },
        );
        panic!("Bounty has expired");
    }

    // Must be Open to claim
    if bounty.status != BountyStatus::Open {
        panic!("Bounty is not open for claiming");
    }

    // Update bounty state
    bounty.status = BountyStatus::Claimed;
    bounty.claimer = Some(claimer.clone());

    store_bounty(env, &bounty);

    // Emit claim event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "claimed")),
        BountyClaimedEvent { bounty_id, claimer },
    );

    true
}

/// Submit work for a claimed bounty
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty
/// * `submission_url` - URL or reference to the submitted work
///
/// # Returns
/// `true` if submission was successful
///
/// # Panics
/// - If bounty is not found
/// - If caller is not the claimer
/// - If bounty is not in Claimed status
pub fn submit_work(env: &Env, bounty_id: u64, submission_url: String) -> bool {
    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Verify claimer
    let claimer = bounty.claimer.clone().expect("No claimer for this bounty");
    claimer.require_auth();

    // Must be in Claimed status
    if bounty.status != BountyStatus::Claimed {
        panic!("Bounty is not in claimed status");
    }

    // Validate submission URL
    if submission_url.len() == 0 || submission_url.len() > 512 {
        panic!("Submission URL must be between 1 and 512 characters");
    }

    // Update status
    bounty.status = BountyStatus::UnderReview;
    bounty.submission_url = Some(submission_url.clone());

    store_bounty(env, &bounty);

    // Emit submission event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "submitted")),
        WorkSubmittedEvent {
            bounty_id,
            claimer,
            submission_url,
        },
    );

    true
}

/// Approve completion of a bounty
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty to approve
/// * `approver` - Address of the approver (must be guild admin/owner)
///
/// # Returns
/// `true` if approval was successful
///
/// # Panics
/// - If bounty is not found
/// - If approver is not a guild admin/owner
/// - If bounty is not under review
pub fn approve_completion(env: &Env, bounty_id: u64, approver: Address) -> bool {
    approver.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Verify approver permissions
    if !has_permission(env, bounty.guild_id, approver.clone(), Role::Admin) {
        panic!("Unauthorized: Approver must be a guild admin or owner");
    }

    // Must be under review
    if bounty.status != BountyStatus::UnderReview {
        panic!("Bounty is not under review");
    }

    // Update status
    bounty.status = BountyStatus::Completed;

    store_bounty(env, &bounty);

    // Emit approval event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "approved")),
        BountyApprovedEvent {
            bounty_id,
            approver,
        },
    );

    true
}

/// Release escrow funds to the bounty claimer
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the completed bounty
///
/// # Returns
/// `true` if release was successful
///
/// # Panics
/// - If bounty is not found
/// - If bounty is not completed
/// - If no claimer exists
pub fn release_escrow(env: &Env, bounty_id: u64) -> bool {
    let bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Must be completed
    if bounty.status != BountyStatus::Completed {
        panic!("Bounty is not completed");
    }

    let claimer = bounty.claimer.clone().expect("No claimer for this bounty");

    // Release funds to claimer
    if bounty.funded_amount > 0 {
        release_funds(env, &bounty.token, &claimer, bounty.funded_amount);

        // Emit release event
        env.events().publish(
            (Symbol::new(env, "bounty"), Symbol::new(env, "released")),
            EscrowReleasedEvent {
                bounty_id,
                recipient: claimer,
                amount: bounty.funded_amount,
                token: bounty.token,
            },
        );
    }

    true
}

/// Cancel a bounty and refund escrowed funds
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty to cancel
/// * `canceller` - Address attempting to cancel (must be creator or guild admin)
///
/// # Returns
/// `true` if cancellation was successful
///
/// # Panics
/// - If bounty is not found
/// - If canceller is not authorized
/// - If bounty is already completed or cancelled
pub fn cancel_bounty(env: &Env, bounty_id: u64, canceller: Address) -> bool {
    canceller.require_auth();

    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Cannot cancel completed or already cancelled bounties
    match bounty.status {
        BountyStatus::Completed | BountyStatus::Cancelled => {
            panic!("Bounty cannot be cancelled in current status");
        }
        _ => {}
    }

    // Authorization: creator or guild admin can cancel
    let is_creator = bounty.creator == canceller;
    let is_admin = has_permission(env, bounty.guild_id, canceller.clone(), Role::Admin);

    if !is_creator && !is_admin {
        panic!("Unauthorized: Only creator or guild admin can cancel");
    }

    let refund_amount = bounty.funded_amount;
    let refund_recipient = bounty.creator.clone();

    // Refund escrowed funds to creator
    if refund_amount > 0 {
        release_funds(env, &bounty.token, &refund_recipient, refund_amount);
        bounty.funded_amount = 0;
    }

    bounty.status = BountyStatus::Cancelled;
    store_bounty(env, &bounty);

    // Emit cancellation event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "cancelled")),
        BountyCancelledEvent {
            bounty_id,
            canceller,
            refund_amount,
            refund_recipient,
        },
    );

    true
}

/// Handle expired bounty - refund funds and update status
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty to check/expire
///
/// # Returns
/// `true` if bounty was expired and refunded
pub fn expire_bounty(env: &Env, bounty_id: u64) -> bool {
    let mut bounty = get_bounty(env, bounty_id).expect("Bounty not found");

    // Already expired or completed
    if bounty.status == BountyStatus::Expired
        || bounty.status == BountyStatus::Completed
        || bounty.status == BountyStatus::Cancelled
    {
        return false;
    }

    let now = env.ledger().timestamp();
    if now <= bounty.expires_at {
        return false; // Not expired yet
    }

    // Refund escrowed funds
    if bounty.funded_amount > 0 {
        release_funds(env, &bounty.token, &bounty.creator, bounty.funded_amount);
        bounty.funded_amount = 0;
    }

    bounty.status = BountyStatus::Expired;
    store_bounty(env, &bounty);

    // Emit expiration event
    env.events().publish(
        (Symbol::new(env, "bounty"), Symbol::new(env, "expired")),
        BountyExpiredEvent { bounty_id },
    );

    true
}

// ============ Query Functions ============

/// Get bounty by ID
///
/// # Arguments
/// * `env` - The contract environment
/// * `bounty_id` - The ID of the bounty
///
/// # Returns
/// The Bounty struct
///
/// # Panics
/// If bounty is not found
pub fn get_bounty_data(env: &Env, bounty_id: u64) -> Bounty {
    get_bounty(env, bounty_id).expect("Bounty not found")
}

/// Get all bounties for a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
///
/// # Returns
/// Vector of all bounties belonging to the guild
pub fn get_guild_bounties_list(env: &Env, guild_id: u64) -> Vec<Bounty> {
    get_guild_bounties(env, guild_id)
}

// Legacy function name for compatibility
pub fn cancel_bounty_auth(env: &Env, bounty_id: u64, canceller: Address) -> bool {
    cancel_bounty(env, bounty_id, canceller)
}

#[cfg(test)]
mod tests;
