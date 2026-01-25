#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

mod guild;

use guild::membership::{
    add_member, create_guild, get_all_members, get_member, has_permission, is_member,
    remove_member, update_role,
};
use guild::storage;
use guild::types::{Member, Role};

mod bounty;
use bounty::{
    approve_completion, cancel_bounty, claim_bounty, create_bounty, expire_bounty, fund_bounty,
    get_bounty_data, get_guild_bounties_list, release_escrow, submit_work, Bounty, BountyStatus,
};

mod treasury;
use treasury::initialize_treasury_storage;
use treasury::{
    approve_transaction as core_approve_transaction, deposit as core_deposit,
    emergency_pause as core_emergency_pause, execute_transaction as core_execute_transaction,
    get_balance as core_get_balance, get_transaction_history as core_get_transaction_history,
    grant_allowance as core_grant_allowance, initialize_treasury as core_initialize_treasury,
    propose_withdrawal as core_propose_withdrawal, set_budget as core_set_budget, Transaction,
};

mod governance;
use governance::{
    cancel_proposal as gov_cancel_proposal, create_proposal as gov_create_proposal,
    delegate_vote as gov_delegate_vote, execute_proposal as gov_execute_proposal,
    finalize_proposal as gov_finalize_proposal, get_active_proposals as gov_get_active_proposals,
    get_proposal as gov_get_proposal, undelegate_vote as gov_undelegate_vote,
    update_governance_config as gov_update_governance_config, vote as gov_vote, ExecutionPayload,
    GovernanceConfig, Proposal, ProposalStatus, ProposalType, VoteDecision,
};

mod milestone;
use milestone::{
    add_milestone as ms_add_milestone, approve_milestone as ms_approve_milestone,
    cancel_project as ms_cancel_project, create_project as ms_create_project,
    extend_milestone_deadline as ms_extend_deadline, get_milestone_view as ms_get_milestone,
    get_project_progress as ms_get_progress, reject_milestone as ms_reject_milestone,
    release_milestone_payment as ms_release_payment, start_milestone as ms_start_milestone,
    submit_milestone as ms_submit_milestone, Milestone, MilestoneInput, MilestoneStatus, Project,
    ProjectStatus,
};

mod payment;
use payment::{
    add_recipient as pay_add_recipient, batch_distribute as pay_batch_distribute,
    cancel_distribution as pay_cancel_distribution, create_payment_pool as pay_create_payment_pool,
    execute_distribution as pay_execute_distribution, get_pool_status as pay_get_pool_status,
    get_recipient_amount as pay_get_recipient_amount,
    validate_distribution as pay_validate_distribution, DistributionRule, DistributionStatus,
};

/// Stellar Guilds - Main Contract Entry Point
///
/// This is the foundational contract for the Stellar Guilds platform.
/// It enables users to create guilds, add members, assign roles, and manage
/// permissions on-chain. This forms the foundation for decentralized communities,
/// voting, and role-based governance.
///
/// # Core Features
/// - Guild creation with metadata
/// - Member management with role assignments
/// - Permission-based access control
/// - Event tracking for all state changes
/// - Efficient on-chain storage management

#[contract]
pub struct StellarGuildsContract;

#[contractimpl]
impl StellarGuildsContract {
    pub fn initialize(env: Env) -> bool {
        storage::initialize(&env);
        true
    }

    /// Get contract version
    pub fn version(_env: Env) -> String {
        String::from_str(&_env, "0.1.0")
    }

    /// Create a new guild
    ///
    /// # Arguments
    /// * `name` - The name of the guild
    /// * `description` - The description of the guild
    /// * `owner` - The address of the guild owner
    ///
    /// # Returns
    /// The ID of the newly created guild
    pub fn create_guild(env: Env, name: String, description: String, owner: Address) -> u64 {
        owner.require_auth();
        match create_guild(&env, name, description, owner) {
            Ok(id) => id,
            Err(_) => panic!("create_guild error"),
        }
    }

    /// Add a member to a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member to add
    /// * `role` - The role to assign
    /// * `caller` - The address making the request (must have permission)
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn add_member(
        env: Env,
        guild_id: u64,
        address: Address,
        role: Role,
        caller: Address,
    ) -> bool {
        caller.require_auth();
        match add_member(&env, guild_id, address, role, caller) {
            Ok(result) => result,
            Err(_) => panic!("add_member error"),
        }
    }

    /// Remove a member from a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member to remove
    /// * `caller` - The address making the request
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn remove_member(env: Env, guild_id: u64, address: Address, caller: Address) -> bool {
        caller.require_auth();
        match remove_member(&env, guild_id, address, caller) {
            Ok(result) => result,
            Err(_) => panic!("remove_member error"),
        }
    }

    /// Update a member's role
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    /// * `new_role` - The new role to assign
    /// * `caller` - The address making the request (must have permission)
    ///
    /// # Returns
    /// true if successful, panics with error message otherwise
    pub fn update_role(
        env: Env,
        guild_id: u64,
        address: Address,
        new_role: Role,
        caller: Address,
    ) -> bool {
        caller.require_auth();
        match update_role(&env, guild_id, address, new_role, caller) {
            Ok(result) => result,
            Err(_) => panic!("update_role error"),
        }
    }

    /// Get a member from a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    ///
    /// # Returns
    /// The Member if found, panics with error message otherwise
    pub fn get_member(env: Env, guild_id: u64, address: Address) -> Member {
        match get_member(&env, guild_id, address) {
            Ok(member) => member,
            Err(_) => panic!("get_member error"),
        }
    }

    /// Get all members of a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// A vector of all members in the guild
    pub fn get_all_members(env: Env, guild_id: u64) -> Vec<Member> {
        get_all_members(&env, guild_id)
    }

    /// Check if an address is a member of a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if the address is a member, false otherwise
    pub fn is_member(env: Env, guild_id: u64, address: Address) -> bool {
        is_member(&env, guild_id, address)
    }

    /// Check if a member has permission for a required role
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `address` - The address of the member
    /// * `required_role` - The required role level
    ///
    /// # Returns
    /// true if the member has the required permission, false otherwise
    pub fn has_permission(env: Env, guild_id: u64, address: Address, required_role: Role) -> bool {
        has_permission(&env, guild_id, address, required_role)
    }

    // ============ Payment Functions ============

    pub fn create_payment_pool(
        env: Env,
        total_amount: i128,
        token: Option<Address>,
        rule: DistributionRule,
        creator: Address,
    ) -> u64 {
        match pay_create_payment_pool(&env, total_amount, token, rule, creator) {
            Ok(id) => id,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn add_recipient(
        env: Env,
        pool_id: u64,
        recipient: Address,
        share: u32,
        caller: Address,
    ) -> bool {
        match pay_add_recipient(&env, pool_id, recipient, share, caller) {
            Ok(result) => result,
            Err(e) => {
                // Convert error enum to panic message
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn validate_distribution(env: Env, pool_id: u64) -> bool {
        match pay_validate_distribution(&env, pool_id) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn get_recipient_amount(env: Env, pool_id: u64, recipient: Address) -> i128 {
        match pay_get_recipient_amount(&env, pool_id, recipient) {
            Ok(amount) => amount,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn cancel_distribution(env: Env, pool_id: u64, caller: Address) -> bool {
        match pay_cancel_distribution(&env, pool_id, caller) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    pub fn get_pool_status(env: Env, pool_id: u64) -> DistributionStatus {
        match pay_get_pool_status(&env, pool_id) {
            Ok(status) => status,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    /// Execute distribution for a payment pool
    ///
    /// # Arguments
    /// * `pool_id` - The ID of the pool to execute
    /// * `caller` - The address executing the distribution (must be pool creator)
    ///
    /// # Returns
    /// `true` if distribution was successful
    pub fn execute_distribution(env: Env, pool_id: u64, caller: Address) -> bool {
        match pay_execute_distribution(&env, pool_id, caller) {
            Ok(result) => result,
            Err(e) => {
                let msg = match e as u32 {
                    1 => "PoolNotFound",
                    2 => "PoolNotPending",
                    3 => "Unauthorized",
                    4 => "InvalidShare",
                    5 => "DuplicateRecipient",
                    6 => "SharesNot100Percent",
                    7 => "NoRecipients",
                    8 => "InsufficientBalance",
                    9 => "TransferFailed",
                    10 => "ArithmeticOverflow",
                    11 => "InvalidAmount",
                    _ => "Unknown error",
                };
                panic!("{}", msg);
            }
        }
    }

    /// Execute distribution for multiple payment pools in batch
    ///
    /// # Arguments
    /// * `pool_ids` - Vector of pool IDs to execute
    /// * `caller` - The address executing the distributions (must be pool creator for each)
    ///
    /// # Returns
    /// Vector of results (true for success, false for failure) for each pool
    pub fn batch_distribute(env: Env, pool_ids: Vec<u64>, caller: Address) -> Vec<bool> {
        pay_batch_distribute(&env, pool_ids, caller)
    }

    // ============ Treasury Functions ============

    /// Initialize a new treasury for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `signers` - Vector of signer addresses (first is owner)
    /// * `approval_threshold` - Number of approvals required for transactions
    ///
    /// # Returns
    /// The ID of the newly created treasury
    pub fn initialize_treasury(
        env: Env,
        guild_id: u64,
        signers: Vec<Address>,
        approval_threshold: u32,
    ) -> u64 {
        core_initialize_treasury(&env, guild_id, signers, approval_threshold)
    }

    /// Deposit funds into a treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `depositor` - Address making the deposit
    /// * `amount` - Amount to deposit
    /// * `token` - Token address (None for XLM)
    ///
    /// # Returns
    /// `true` if deposit was successful
    pub fn deposit_treasury(
        env: Env,
        treasury_id: u64,
        depositor: Address,
        amount: i128,
        token: Option<Address>,
    ) -> bool {
        core_deposit(&env, treasury_id, depositor, amount, token)
    }

    /// Propose a withdrawal from treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `proposer` - Address proposing the withdrawal
    /// * `recipient` - Address to receive the funds
    /// * `amount` - Amount to withdraw
    /// * `token` - Token address (None for XLM)
    /// * `reason` - Reason for the withdrawal
    ///
    /// # Returns
    /// The ID of the proposed transaction
    pub fn propose_withdrawal(
        env: Env,
        treasury_id: u64,
        proposer: Address,
        recipient: Address,
        amount: i128,
        token: Option<Address>,
        reason: String,
    ) -> u64 {
        core_propose_withdrawal(
            &env,
            treasury_id,
            proposer,
            recipient,
            amount,
            token,
            reason,
        )
    }

    /// Approve a proposed transaction
    ///
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to approve
    /// * `approver` - Address approving the transaction
    ///
    /// # Returns
    /// `true` if approval was successful
    pub fn approve_transaction(env: Env, tx_id: u64, approver: Address) -> bool {
        core_approve_transaction(&env, tx_id, approver)
    }

    /// Execute an approved transaction
    ///
    /// # Arguments
    /// * `tx_id` - The ID of the transaction to execute
    /// * `executor` - Address executing the transaction
    ///
    /// # Returns
    /// `true` if execution was successful
    pub fn execute_transaction(env: Env, tx_id: u64, executor: Address) -> bool {
        core_execute_transaction(&env, tx_id, executor)
    }

    /// Set a budget for a treasury category
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `category` - Budget category name
    /// * `amount` - Budget amount
    /// * `period_seconds` - Budget period in seconds
    /// * `caller` - Address making the request (must be signer)
    ///
    /// # Returns
    /// `true` if budget was set successfully
    pub fn set_budget(
        env: Env,
        treasury_id: u64,
        category: String,
        amount: i128,
        period_seconds: u64,
        caller: Address,
    ) -> bool {
        core_set_budget(&env, treasury_id, caller, category, amount, period_seconds)
    }

    /// Get treasury balance for a token
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `token` - Token address (None for XLM)
    ///
    /// # Returns
    /// The balance amount
    pub fn get_treasury_balance(env: Env, treasury_id: u64, token: Option<Address>) -> i128 {
        core_get_balance(&env, treasury_id, token)
    }

    /// Get transaction history for a treasury
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `limit` - Maximum number of transactions to return
    ///
    /// # Returns
    /// Vector of transactions
    pub fn get_transaction_history(env: Env, treasury_id: u64, limit: u32) -> Vec<Transaction> {
        core_get_transaction_history(&env, treasury_id, limit)
    }

    /// Grant an allowance to an admin
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `admin` - Address to grant allowance to
    /// * `amount` - Allowance amount per period
    /// * `token` - Token address (None for XLM)
    /// * `period_seconds` - Allowance period in seconds
    /// * `owner` - Treasury owner making the request
    ///
    /// # Returns
    /// `true` if allowance was granted successfully
    pub fn grant_allowance(
        env: Env,
        treasury_id: u64,
        admin: Address,
        amount: i128,
        token: Option<Address>,
        period_seconds: u64,
        owner: Address,
    ) -> bool {
        core_grant_allowance(
            &env,
            treasury_id,
            owner,
            admin,
            amount,
            token,
            period_seconds,
        )
    }

    /// Emergency pause treasury operations
    ///
    /// # Arguments
    /// * `treasury_id` - The ID of the treasury
    /// * `signer` - Address making the request (must be signer)
    /// * `paused` - Whether to pause or unpause
    ///
    /// # Returns
    /// `true` if pause state was changed successfully
    pub fn emergency_pause(env: Env, treasury_id: u64, signer: Address, paused: bool) -> bool {
        core_emergency_pause(&env, treasury_id, signer, paused)
    }

    // ============ Milestone Tracking Functions ============

    /// Create a new project with milestones
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `contributor` - Address of the project contributor
    /// * `milestones` - Vector of milestone definitions
    /// * `total_amount` - Total project budget
    /// * `treasury_id` - Treasury ID for payments
    /// * `token` - Token address (None for XLM)
    /// * `is_sequential` - Whether milestones must be completed in order
    ///
    /// # Returns
    /// The ID of the newly created project
    pub fn create_project(
        env: Env,
        guild_id: u64,
        contributor: Address,
        milestones: Vec<MilestoneInput>,
        total_amount: i128,
        treasury_id: u64,
        token: Option<Address>,
        is_sequential: bool,
    ) -> u64 {
        ms_create_project(
            &env,
            guild_id,
            contributor,
            milestones,
            total_amount,
            treasury_id,
            token,
            is_sequential,
        )
    }

    /// Add a new milestone to an existing project
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    /// * `title` - Milestone title
    /// * `description` - Milestone description
    /// * `amount` - Payment amount for this milestone
    /// * `deadline` - Deadline timestamp
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// The ID of the newly created milestone
    pub fn add_milestone(
        env: Env,
        project_id: u64,
        title: String,
        description: String,
        amount: i128,
        deadline: u64,
        caller: Address,
    ) -> u64 {
        ms_add_milestone(
            &env,
            project_id,
            title,
            description,
            amount,
            deadline,
            caller,
        )
    }

    /// Start working on a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `contributor` - Address of the contributor starting work
    ///
    /// # Returns
    /// `true` if successful
    pub fn start_milestone(env: Env, milestone_id: u64, contributor: Address) -> bool {
        ms_start_milestone(&env, milestone_id, contributor)
    }

    /// Submit completed work for a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `proof_url` - URL to proof of work
    ///
    /// # Returns
    /// `true` if successful
    pub fn submit_milestone(env: Env, milestone_id: u64, proof_url: String) -> bool {
        ms_submit_milestone(&env, milestone_id, proof_url)
    }

    /// Approve a submitted milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `approver` - Address of the approver (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn approve_milestone(env: Env, milestone_id: u64, approver: Address) -> bool {
        ms_approve_milestone(&env, milestone_id, approver)
    }

    /// Reject a submitted milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `approver` - Address of the approver (must be guild admin)
    /// * `reason` - Reason for rejection
    ///
    /// # Returns
    /// `true` if successful
    pub fn reject_milestone(
        env: Env,
        milestone_id: u64,
        approver: Address,
        reason: String,
    ) -> bool {
        ms_reject_milestone(&env, milestone_id, approver, reason)
    }

    /// Get project progress statistics
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    ///
    /// # Returns
    /// Tuple of (completed_count, total_count, percentage)
    pub fn get_project_progress(env: Env, project_id: u64) -> (u32, u32, u32) {
        ms_get_progress(&env, project_id)
    }

    /// Get milestone details
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    ///
    /// # Returns
    /// The Milestone struct
    pub fn get_milestone(env: Env, milestone_id: u64) -> Milestone {
        ms_get_milestone(&env, milestone_id)
    }

    /// Release payment for an approved milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    ///
    /// # Returns
    /// `true` if successful
    pub fn release_milestone_payment(env: Env, milestone_id: u64) -> bool {
        ms_release_payment(&env, milestone_id)
    }

    /// Extend the deadline of a milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone
    /// * `new_deadline` - New deadline timestamp
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn extend_milestone_deadline(
        env: Env,
        milestone_id: u64,
        new_deadline: u64,
        caller: Address,
    ) -> bool {
        ms_extend_deadline(&env, milestone_id, new_deadline, caller)
    }

    /// Cancel a project
    ///
    /// # Arguments
    /// * `project_id` - The ID of the project
    /// * `caller` - Address making the request (must be guild admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn cancel_project(env: Env, project_id: u64, caller: Address) -> bool {
        ms_cancel_project(&env, project_id, caller)
    }

    // ============ Governance Functions ============

    /// Create a new governance proposal
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `proposer` - Address of the proposer
    /// * `proposal_type` - Type of the proposal
    /// * `title` - Proposal title
    /// * `description` - Detailed description
    ///
    /// # Returns
    /// The ID of the newly created proposal
    pub fn create_proposal(
        env: Env,
        guild_id: u64,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
    ) -> u64 {
        gov_create_proposal(
            &env,
            guild_id,
            proposer,
            proposal_type,
            title,
            description,
            ExecutionPayload::GeneralDecision,
        )
    }

    /// Get a proposal by ID
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// The Proposal struct
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        gov_get_proposal(&env, proposal_id)
    }

    /// Get all active proposals for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// Vector of active proposals
    pub fn get_active_proposals(env: Env, guild_id: u64) -> Vec<Proposal> {
        gov_get_active_proposals(&env, guild_id)
    }

    /// Cast a vote on a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    /// * `voter` - Address of the voter
    /// * `decision` - Vote decision (For, Against, Abstain)
    ///
    /// # Returns
    /// `true` if successful
    pub fn vote(env: Env, proposal_id: u64, voter: Address, decision: VoteDecision) -> bool {
        gov_vote(&env, proposal_id, voter, decision)
    }

    /// Delegate voting power to another member
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `delegator` - Address delegating their vote
    /// * `delegate` - Address receiving the delegation
    ///
    /// # Returns
    /// `true` if successful
    pub fn delegate_vote(env: Env, guild_id: u64, delegator: Address, delegate: Address) -> bool {
        gov_delegate_vote(&env, guild_id, delegator, delegate)
    }

    /// Remove vote delegation
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `delegator` - Address removing their delegation
    ///
    /// # Returns
    /// `true` if successful
    pub fn undelegate_vote(env: Env, guild_id: u64, delegator: Address) -> bool {
        gov_undelegate_vote(&env, guild_id, delegator)
    }

    /// Finalize a proposal after voting period ends
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// The final status of the proposal
    pub fn finalize_proposal(env: Env, proposal_id: u64) -> ProposalStatus {
        gov_finalize_proposal(&env, proposal_id)
    }

    /// Execute a passed proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal to execute
    ///
    /// # Returns
    /// `true` if execution was successful
    pub fn execute_proposal(env: Env, proposal_id: u64) -> bool {
        gov_execute_proposal(&env, proposal_id)
    }

    /// Cancel a proposal
    ///
    /// # Arguments
    /// * `proposal_id` - The ID of the proposal
    /// * `caller` - Address making the request (must be proposer or admin)
    ///
    /// # Returns
    /// `true` if successful
    pub fn cancel_proposal(env: Env, proposal_id: u64, caller: Address) -> bool {
        gov_cancel_proposal(&env, proposal_id, caller)
    }

    /// Update governance configuration
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    /// * `caller` - Address making the request (must be owner)
    /// * `config` - New governance configuration
    ///
    /// # Returns
    /// `true` if successful
    pub fn update_governance_config(
        env: Env,
        guild_id: u64,
        caller: Address,
        config: GovernanceConfig,
    ) -> bool {
        gov_update_governance_config(&env, guild_id, caller, config)
    }

    // ============ Bounty Escrow Functions ============

    /// Create a new bounty
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild creating the bounty
    /// * `creator` - Address of the bounty creator (must be guild admin/owner)
    /// * `title` - Short title for the bounty
    /// * `description` - Detailed description of the task
    /// * `reward_amount` - Amount of tokens as reward
    /// * `token` - Address of the token contract
    /// * `expiry` - Absolute timestamp when the bounty expires
    ///
    /// # Returns
    /// The ID of the newly created bounty
    pub fn create_bounty(
        env: Env,
        guild_id: u64,
        creator: Address,
        title: String,
        description: String,
        reward_amount: i128,
        token: Address,
        expiry: u64,
    ) -> u64 {
        create_bounty(
            &env,
            guild_id,
            creator,
            title,
            description,
            reward_amount,
            token,
            expiry,
        )
    }

    /// Fund a bounty with tokens
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to fund
    /// * `funder` - Address providing the funds
    /// * `amount` - Amount of tokens to fund
    ///
    /// # Returns
    /// `true` if funding was successful
    pub fn fund_bounty(env: Env, bounty_id: u64, funder: Address, amount: i128) -> bool {
        fund_bounty(&env, bounty_id, funder, amount)
    }

    /// Claim a bounty (first-come-first-served)
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to claim
    /// * `claimer` - Address of the contributor claiming the bounty
    ///
    /// # Returns
    /// `true` if claiming was successful
    pub fn claim_bounty(env: Env, bounty_id: u64, claimer: Address) -> bool {
        claim_bounty(&env, bounty_id, claimer)
    }

    /// Submit work for a claimed bounty
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty
    /// * `submission_url` - URL or reference to the submitted work
    ///
    /// # Returns
    /// `true` if submission was successful
    pub fn submit_work(env: Env, bounty_id: u64, submission_url: String) -> bool {
        submit_work(&env, bounty_id, submission_url)
    }

    /// Approve completion of a bounty
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to approve
    /// * `approver` - Address of the approver (must be guild admin/owner)
    ///
    /// # Returns
    /// `true` if approval was successful
    pub fn approve_completion(env: Env, bounty_id: u64, approver: Address) -> bool {
        approve_completion(&env, bounty_id, approver)
    }

    /// Release escrow funds to the bounty claimer
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the completed bounty
    ///
    /// # Returns
    /// `true` if release was successful
    pub fn release_escrow(env: Env, bounty_id: u64) -> bool {
        release_escrow(&env, bounty_id)
    }

    /// Cancel a bounty and refund escrowed funds
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to cancel
    /// * `canceller` - Address attempting to cancel (must be creator or guild admin)
    ///
    /// # Returns
    /// `true` if cancellation was successful
    pub fn cancel_bounty(env: Env, bounty_id: u64, canceller: Address) -> bool {
        cancel_bounty(&env, bounty_id, canceller)
    }

    /// Handle expired bounty - refund funds and update status
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty to check/expire
    ///
    /// # Returns
    /// `true` if bounty was expired and refunded
    pub fn expire_bounty(env: Env, bounty_id: u64) -> bool {
        expire_bounty(&env, bounty_id)
    }

    /// Get bounty by ID
    ///
    /// # Arguments
    /// * `bounty_id` - The ID of the bounty
    ///
    /// # Returns
    /// The Bounty struct
    pub fn get_bounty(env: Env, bounty_id: u64) -> Bounty {
        get_bounty_data(&env, bounty_id)
    }

    /// Get all bounties for a guild
    ///
    /// # Arguments
    /// * `guild_id` - The ID of the guild
    ///
    /// # Returns
    /// Vector of all bounties belonging to the guild
    pub fn get_guild_bounties(env: Env, guild_id: u64) -> Vec<Bounty> {
        get_guild_bounties_list(&env, guild_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.budget().reset_unlimited();

        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let member = Address::generate(&env);
        let non_member = Address::generate(&env);

        (env, owner, admin, member, non_member)
    }

    fn register_and_init_contract(env: &Env) -> Address {
        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(env, &contract_id);

        client.initialize();

        contract_id
    }

    // ============ Initialization Tests ============

    #[test]
    fn test_initialize() {
        let (env, _, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);

        // Verify initialization was successful
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let result = client.initialize();
        assert_eq!(result, true);
    }

    #[test]
    fn test_version() {
        let (env, _, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);

        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let version = client.version();
        assert_eq!(version, String::from_str(&env, "0.1.0"));
    }

    // ============ Guild Creation Tests ============

    #[test]
    fn test_create_guild_success() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Test Guild");
        let description = String::from_str(&env, "A test guild");

        let guild_id = client.create_guild(&name, &description, &owner);
        assert_eq!(guild_id, 1u64);
    }

    #[test]
    fn test_create_guild_owner_is_member() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Owner should be a member after creation
        let is_member = client.is_member(&guild_id, &owner);
        assert_eq!(is_member, true);

        let member = client.get_member(&guild_id, &owner);
        assert_eq!(member.role, Role::Owner);
    }

    #[test]
    #[should_panic]
    fn test_create_guild_invalid_name_empty() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "");
        let description = String::from_str(&env, "Description");

        client.create_guild(&name, &description, &owner);
    }

    #[test]
    #[should_panic]
    fn test_create_guild_invalid_description_too_long() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        // Create a description that is too long (> 512 chars)
        let long_desc = "x".repeat(513);
        let description = String::from_str(&env, &long_desc);

        client.create_guild(&name, &description, &owner);
    }

    #[test]
    fn test_create_multiple_guilds() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name1 = String::from_str(&env, "Guild 1");
        let description1 = String::from_str(&env, "First guild");

        let guild_id_1 = client.create_guild(&name1, &description1, &owner);

        let name2 = String::from_str(&env, "Guild 2");
        let description2 = String::from_str(&env, "Second guild");

        let guild_id_2 = client.create_guild(&name2, &description2, &owner);

        // Guild IDs should be unique and incremental
        assert_eq!(guild_id_1, 1u64);
        assert_eq!(guild_id_2, 2u64);
    }

    // ============ Member Addition Tests ============

    #[test]
    fn test_add_member_by_owner() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Owner adds admin
        let result = client.add_member(&guild_id, &admin, &Role::Admin, &owner);
        assert_eq!(result, true);

        let member = client.get_member(&guild_id, &admin);
        assert_eq!(member.role, Role::Admin);
    }

    #[test]
    #[should_panic]
    fn test_add_member_duplicate() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member once
        client.add_member(&guild_id, &admin, &Role::Member, &owner);

        // Try to add same member again - should panic
        client.add_member(&guild_id, &admin, &Role::Member, &owner);
    }

    #[test]
    #[should_panic]
    fn test_add_member_permission_denied() {
        let (env, owner, admin, member, non_member) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Non-member tries to add someone - should panic
        client.add_member(&guild_id, &non_member, &Role::Member, &non_member);
    }

    #[test]
    #[should_panic]
    fn test_add_admin_by_non_owner() {
        let (env, owner, admin, member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Member tries to add an owner - should panic
        let new_owner = Address::generate(&env);
        env.mock_all_auths();

        client.add_member(&guild_id, &new_owner, &Role::Owner, &member);
    }

    // ============ Member Removal Tests ============

    #[test]
    fn test_remove_member_by_owner() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Verify member exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, true);

        // Remove member
        let result = client.remove_member(&guild_id, &member, &owner);
        assert_eq!(result, true);

        // Verify member no longer exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, false);
    }

    #[test]
    fn test_self_removal() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Member removes themselves
        let result = client.remove_member(&guild_id, &member, &member);
        assert_eq!(result, true);

        // Verify member no longer exists
        let is_member = client.is_member(&guild_id, &member);
        assert_eq!(is_member, false);
    }

    #[test]
    #[should_panic]
    fn test_remove_last_owner_fails() {
        let (env, owner, _, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Try to remove the only owner - should panic
        client.remove_member(&guild_id, &owner, &owner);
    }

    #[test]
    #[should_panic]
    fn test_remove_non_owner_by_non_owner_fails() {
        let (env, owner, admin, member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member and admin
        client.add_member(&guild_id, &member, &Role::Member, &owner);
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Member tries to remove admin - should panic
        client.remove_member(&guild_id, &admin, &member);
    }

    // ============ Role Update Tests ============

    #[test]
    fn test_update_role_by_owner() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add member
        client.add_member(&guild_id, &member, &Role::Member, &owner);

        // Update to admin
        let result = client.update_role(&guild_id, &member, &Role::Admin, &owner);
        assert_eq!(result, true);

        let updated_member = client.get_member(&guild_id, &member);
        assert_eq!(updated_member.role, Role::Admin);
    }

    #[test]
    #[should_panic]
    fn test_update_role_permission_denied() {
        let (env, owner, member1, member2, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add members
        client.add_member(&guild_id, &member1, &Role::Member, &owner);
        client.add_member(&guild_id, &member2, &Role::Member, &owner);

        // Member1 tries to change member2's role - should panic
        client.update_role(&guild_id, &member2, &Role::Admin, &member1);
    }

    #[test]
    #[should_panic]
    fn test_cannot_demote_last_owner() {
        let (env, owner, admin, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Try to demote the last owner - should panic
        client.update_role(&guild_id, &owner, &Role::Admin, &owner);
    }

    #[test]
    fn test_can_demote_owner_if_multiple() {
        let (env, owner1, owner2, member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner1);

        // Add owner2
        client.add_member(&guild_id, &owner2, &Role::Owner, &owner1);

        // Now owner1 can be demoted
        let result = client.update_role(&guild_id, &owner1, &Role::Admin, &owner1);
        assert_eq!(result, true);
    }

    // ============ Member Query Tests ============

    #[test]
    fn test_get_member() {
        let (env, owner, member, _, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &member, &Role::Member, &owner);

        let member_data = client.get_member(&guild_id, &member);
        assert_eq!(member_data.address, member);
        assert_eq!(member_data.role, Role::Member);
    }

    #[test]
    #[should_panic]
    fn test_get_member_not_found() {
        let (env, owner, member, non_member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &member, &Role::Member, &owner);

        client.get_member(&guild_id, &non_member);
    }

    #[test]
    fn test_get_all_members() {
        let (env, owner, member1, member2, member3) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Initially should have 1 member (owner)
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 1);

        // Add more members
        client.add_member(&guild_id, &member1, &Role::Member, &owner);
        client.add_member(&guild_id, &member2, &Role::Admin, &owner);
        client.add_member(&guild_id, &member3, &Role::Contributor, &owner);

        // Should now have 4 members
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 4);
    }

    #[test]
    fn test_is_member() {
        let (env, owner, member, non_member, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        assert_eq!(client.is_member(&guild_id, &owner), true);
        assert_eq!(client.is_member(&guild_id, &member), false);

        client.add_member(&guild_id, &member, &Role::Member, &owner);
        assert_eq!(client.is_member(&guild_id, &member), true);
        assert_eq!(client.is_member(&guild_id, &non_member), false);
    }

    // ============ Permission Tests ============

    #[test]
    fn test_has_permission() {
        let (env, owner, admin, member, contributor) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        client.add_member(&guild_id, &admin, &Role::Admin, &owner);
        client.add_member(&guild_id, &member, &Role::Member, &owner);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Owner has all permissions
        assert_eq!(client.has_permission(&guild_id, &owner, &Role::Owner), true);
        assert_eq!(client.has_permission(&guild_id, &owner, &Role::Admin), true);
        assert_eq!(
            client.has_permission(&guild_id, &owner, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &owner, &Role::Contributor),
            true
        );

        // Admin has admin and below permissions
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Owner),
            false
        );
        assert_eq!(client.has_permission(&guild_id, &admin, &Role::Admin), true);
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &admin, &Role::Contributor),
            true
        );

        // Member has member and below permissions
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Owner),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Admin),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Member),
            true
        );
        assert_eq!(
            client.has_permission(&guild_id, &member, &Role::Contributor),
            true
        );

        // Contributor has only contributor permissions
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Owner),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Admin),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Member),
            false
        );
        assert_eq!(
            client.has_permission(&guild_id, &contributor, &Role::Contributor),
            true
        );
    }

    // ============ Guild Lifecycle Integration Tests ============

    #[test]
    fn test_full_guild_lifecycle() {
        let (env, owner, admin, member1, member2) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        // Create guild
        let name = String::from_str(&env, "Community Guild");
        let description = String::from_str(&env, "A thriving community");

        let guild_id = client.create_guild(&name, &description, &owner);
        assert_eq!(guild_id, 1u64);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Add members
        client.add_member(&guild_id, &member1, &Role::Member, &admin);
        client.add_member(&guild_id, &member2, &Role::Contributor, &owner);

        // Verify all members exist
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 4); // owner + admin + member1 + member2

        // Promote member1 to member
        client.update_role(&guild_id, &member1, &Role::Member, &admin);

        // member1 removes themselves
        client.remove_member(&guild_id, &member1, &member1);

        // Verify member1 is gone
        let members = client.get_all_members(&guild_id);
        assert_eq!(members.len(), 3);

        assert_eq!(client.is_member(&guild_id, &member1), false);
        assert_eq!(client.is_member(&guild_id, &member2), true);
    }

    #[test]
    fn test_admin_can_add_members_and_contributors() {
        let (env, owner, admin, member, contributor) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Admin adds member and contributor
        let result1 = client.add_member(&guild_id, &member, &Role::Member, &admin);
        assert_eq!(result1, true);

        let result2 = client.add_member(&guild_id, &contributor, &Role::Contributor, &admin);
        assert_eq!(result2, true);

        // Verify they were added
        assert_eq!(client.is_member(&guild_id, &member), true);
        assert_eq!(client.is_member(&guild_id, &contributor), true);
    }

    #[test]
    #[should_panic]
    fn test_admin_cannot_add_owner() {
        let (env, owner, admin, new_owner, _) = setup();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();

        let name = String::from_str(&env, "Guild");
        let description = String::from_str(&env, "Description");

        let guild_id = client.create_guild(&name, &description, &owner);

        // Add admin
        client.add_member(&guild_id, &admin, &Role::Admin, &owner);

        // Admin tries to add owner - should panic
        client.add_member(&guild_id, &new_owner, &Role::Owner, &admin);
    }

    // ============ Payment Distribution Tests ============

    #[test]
    fn test_create_payment_pool_percentage() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let token = Some(Address::generate(&env)); // Mock token address

        // Mock auth
        env.mock_all_auths();

        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
        assert_eq!(pool_id, 1u64);
    }

    #[test]
    fn test_add_recipient_and_validate() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        // Create pool
        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        // Add recipients
        client.add_recipient(&pool_id, &recipient1, &50u32, &creator);
        client.add_recipient(&pool_id, &recipient2, &50u32, &creator);

        // Validate distribution
        let is_valid = client.validate_distribution(&pool_id);
        assert_eq!(is_valid, true);
    }

    #[test]
    fn test_get_recipient_amount_percentage() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        // Create pool
        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        // Add recipient with 25% share
        client.add_recipient(&pool_id, &recipient, &25u32, &creator);

        // Get recipient amount
        let amount = client.get_recipient_amount(&pool_id, &recipient);
        assert_eq!(amount, 250i128); // 25% of 1000
    }

    #[test]
    fn test_equal_split_distribution() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let recipient3 = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        // Create pool
        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::EqualSplit, &creator);

        // Add recipients
        client.add_recipient(&pool_id, &recipient1, &1u32, &creator); // share value doesn't matter for equal split
        client.add_recipient(&pool_id, &recipient2, &1u32, &creator);
        client.add_recipient(&pool_id, &recipient3, &1u32, &creator);

        // Get recipient amounts
        let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
        let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
        let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

        // Each should get 1000 / 3 = 333 (integer division)
        assert_eq!(amount1, 333i128);
        assert_eq!(amount2, 333i128);
        assert_eq!(amount3, 333i128);
    }

    #[test]
    fn test_cancel_distribution() {
        let env = Env::default();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        client.initialize();

        let creator = Address::generate(&env);
        let token = Some(Address::generate(&env));

        env.mock_all_auths();

        // Create pool
        let pool_id =
            client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

        // Cancel pool
        let result = client.cancel_distribution(&pool_id, &creator);
        assert_eq!(result, true);

        // Check status
        let status = client.get_pool_status(&pool_id);
        assert_eq!(status, DistributionStatus::Cancelled);
    }
}
