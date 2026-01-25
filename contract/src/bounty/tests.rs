//! Bounty Escrow Contract Tests
//!
//! Comprehensive test coverage for bounty creation, funding, claiming,
//! submission, approval, escrow release, cancellation, and expiration.
//!
//! NOTE: These tests use the contract client to test through the main lib.rs
//! contract interface, ensuring proper contract context execution.

use crate::bounty::types::BountyStatus;
use crate::guild::types::Role;
use crate::StellarGuildsContract;
use crate::StellarGuildsContractClient;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{token, Address, Env, String};

// ============ Test Helpers ============

fn setup_env() -> Env {
    let env = Env::default();
    env.budget().reset_unlimited();
    env
}

fn set_ledger_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 20,
        sequence_number: 0,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 1000000,
    });
}

fn register_and_init_contract(env: &Env) -> Address {
    let contract_id = env.register_contract(None, StellarGuildsContract);
    let client = StellarGuildsContractClient::new(env, &contract_id);
    client.initialize();
    contract_id
}

fn create_mock_token(env: &Env, admin: &Address) -> Address {
    let token_contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_contract_id.address()
}

fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let client = token::StellarAssetClient::new(env, token);
    client.mint(to, &amount);
}

fn get_token_balance(env: &Env, token: &Address, addr: &Address) -> i128 {
    let client = token::TokenClient::new(env, token);
    client.balance(addr)
}

fn setup_guild(client: &StellarGuildsContractClient<'_>, env: &Env, owner: &Address) -> u64 {
    let name = String::from_str(env, "Test Guild");
    let description = String::from_str(env, "A test guild for bounties");
    client.create_guild(&name, &description, owner)
}

// ============ Bounty Creation Tests ============

#[test]
fn test_create_bounty_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Fix bug in contract");
    let description = String::from_str(&env, "There is a critical bug that needs fixing");
    let reward_amount = 100i128;
    let expiry = 2000u64;

    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &reward_amount,
        &token,
        &expiry,
    );

    assert_eq!(bounty_id, 1);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.id, 1);
    assert_eq!(bounty.guild_id, guild_id);
    assert_eq!(bounty.creator, owner);
    assert_eq!(bounty.reward_amount, reward_amount);
    assert_eq!(bounty.funded_amount, 0);
    assert_eq!(bounty.status, BountyStatus::AwaitingFunds);
    assert_eq!(bounty.expires_at, expiry);
    assert!(bounty.claimer.is_none());
}

#[test]
fn test_create_bounty_zero_reward_is_open() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Community task");
    let description = String::from_str(&env, "Help the community");

    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &0i128,
        &token,
        &2000u64,
    );

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Open);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_create_bounty_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let non_member = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");

    // Non-member tries to create bounty
    client.create_bounty(
        &guild_id,
        &non_member,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );
}

#[test]
#[should_panic(expected = "Invalid reward amount")]
fn test_create_bounty_negative_reward_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");

    client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &-100i128,
        &token,
        &2000u64,
    );
}

#[test]
#[should_panic(expected = "Expiry must be in the future")]
fn test_create_bounty_past_expiry_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 2000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");

    client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &1000u64, // Past expiry
    );
}

#[test]
#[should_panic(expected = "Title must be between 1 and 256 characters")]
fn test_create_bounty_empty_title_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "");
    let description = String::from_str(&env, "Description");

    client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );
}

// ============ Bounty Funding Tests ============

#[test]
fn test_fund_bounty_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    // Mint tokens to funder
    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    // Fund the bounty
    let result = client.fund_bounty(&bounty_id, &funder, &100i128);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.funded_amount, 100);
    assert_eq!(bounty.status, BountyStatus::Open);
}

#[test]
fn test_fund_bounty_partial_funding() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    // Partial fund
    client.fund_bounty(&bounty_id, &funder, &50i128);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.funded_amount, 50);
    assert_eq!(bounty.status, BountyStatus::AwaitingFunds);

    // Complete funding
    client.fund_bounty(&bounty_id, &funder, &50i128);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.funded_amount, 100);
    assert_eq!(bounty.status, BountyStatus::Open);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_fund_bounty_zero_amount_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &0i128);
}

// ============ Bounty Claiming Tests ============

#[test]
fn test_claim_bounty_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    // Claim the bounty
    let result = client.claim_bounty(&bounty_id, &claimer);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Claimed);
    assert_eq!(bounty.claimer, Some(claimer));
}

#[test]
#[should_panic(expected = "Bounty is not open for claiming")]
fn test_claim_bounty_not_open_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    // Try to claim without funding
    client.claim_bounty(&bounty_id, &claimer);
}

#[test]
#[should_panic(expected = "Bounty is not open for claiming")]
fn test_claim_bounty_already_claimed_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer1 = Address::generate(&env);
    let claimer2 = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer1);

    // Double-claim should fail
    client.claim_bounty(&bounty_id, &claimer2);
}

// ============ Work Submission Tests ============

#[test]
fn test_submit_work_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    let result = client.submit_work(&bounty_id, &submission);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::UnderReview);
    assert_eq!(bounty.submission_url, Some(submission));
}

#[test]
#[should_panic(expected = "No claimer for this bounty")]
fn test_submit_work_no_claimer_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    // Submit without claiming
    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);
}

// ============ Approval Tests ============

#[test]
fn test_approve_completion_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);

    let result = client.approve_completion(&bounty_id, &owner);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Completed);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_approve_completion_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);

    // Non-admin tries to approve
    client.approve_completion(&bounty_id, &non_admin);
}

#[test]
#[should_panic(expected = "Bounty is not under review")]
fn test_approve_completion_wrong_status_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    // Approve without submission
    client.approve_completion(&bounty_id, &owner);
}

// ============ Escrow Release Tests ============

#[test]
fn test_release_escrow_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);
    client.approve_completion(&bounty_id, &owner);

    // Release escrow
    let result = client.release_escrow(&bounty_id);
    assert_eq!(result, true);

    // Claimer should have received the funds
    let claimer_balance = get_token_balance(&env, &token, &claimer);
    assert_eq!(claimer_balance, 100);
}

#[test]
#[should_panic(expected = "Bounty is not completed")]
fn test_release_escrow_not_completed_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    // Try to release without completion
    client.release_escrow(&bounty_id);
}

// ============ Cancellation Tests ============

#[test]
fn test_cancel_bounty_by_creator() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    let result = client.cancel_bounty(&bounty_id, &owner);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Cancelled);
    assert_eq!(bounty.funded_amount, 0);

    // Creator should have received the refund
    let creator_balance = get_token_balance(&env, &token, &owner);
    assert_eq!(creator_balance, 100);
}

#[test]
fn test_cancel_bounty_after_claim_refunds_creator() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    // Owner cancels even after claim
    let result = client.cancel_bounty(&bounty_id, &owner);
    assert_eq!(result, true);

    // Funds go to creator, not claimer
    let creator_balance = get_token_balance(&env, &token, &owner);
    let claimer_balance = get_token_balance(&env, &token, &claimer);
    assert_eq!(creator_balance, 100);
    assert_eq!(claimer_balance, 0);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_cancel_bounty_non_creator_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let random_user = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    // Random user tries to cancel
    client.cancel_bounty(&bounty_id, &random_user);
}

#[test]
#[should_panic(expected = "Bounty cannot be cancelled")]
fn test_cancel_bounty_completed_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);
    client.approve_completion(&bounty_id, &owner);

    // Try to cancel a completed bounty
    client.cancel_bounty(&bounty_id, &owner);
}

// ============ Expiration Tests ============

#[test]
fn test_expire_bounty_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &1500u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    // Advance time past expiry
    set_ledger_timestamp(&env, 2000);

    let result = client.expire_bounty(&bounty_id);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Expired);
    assert_eq!(bounty.funded_amount, 0);

    // Creator should have received the refund
    let creator_balance = get_token_balance(&env, &token, &owner);
    assert_eq!(creator_balance, 100);
}

#[test]
fn test_expire_bounty_not_expired_yet() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    // Still before expiry
    let result = client.expire_bounty(&bounty_id);
    assert_eq!(result, false);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Open);
}

// ============ Query Tests ============

#[test]
fn test_get_guild_bounties() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    // Create multiple bounties
    let title1 = String::from_str(&env, "Task 1");
    let title2 = String::from_str(&env, "Task 2");
    let title3 = String::from_str(&env, "Task 3");
    let description = String::from_str(&env, "Description");

    client.create_bounty(
        &guild_id,
        &owner,
        &title1,
        &description,
        &100i128,
        &token,
        &2000u64,
    );
    client.create_bounty(
        &guild_id,
        &owner,
        &title2,
        &description,
        &100i128,
        &token,
        &2000u64,
    );
    client.create_bounty(
        &guild_id,
        &owner,
        &title3,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    let bounties = client.get_guild_bounties(&guild_id);
    assert_eq!(bounties.len(), 3);
}

// ============ Full Lifecycle Integration Test ============

#[test]
fn test_full_bounty_lifecycle() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    // Mint tokens to funder
    mint_tokens(&env, &token, &funder, 1000);

    // 1. Create bounty
    let title = String::from_str(&env, "Implement feature X");
    let description = String::from_str(&env, "Build the amazing feature X");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &5000u64,
    );

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::AwaitingFunds);

    // 2. Fund bounty
    client.fund_bounty(&bounty_id, &funder, &100i128);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Open);

    // 3. Claim bounty
    client.claim_bounty(&bounty_id, &claimer);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Claimed);
    assert_eq!(bounty.claimer, Some(claimer.clone()));

    // 4. Submit work
    let submission = String::from_str(&env, "https://github.com/stellar-guilds/pr/42");
    client.submit_work(&bounty_id, &submission);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::UnderReview);
    assert_eq!(bounty.submission_url, Some(submission));

    // 5. Approve completion
    client.approve_completion(&bounty_id, &owner);
    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Completed);

    // 6. Release escrow
    let funder_balance_before = get_token_balance(&env, &token, &funder);
    assert_eq!(funder_balance_before, 900); // 1000 - 100 funded

    client.release_escrow(&bounty_id);

    // Claimer should have received the payment
    let claimer_balance = get_token_balance(&env, &token, &claimer);
    assert_eq!(claimer_balance, 100);
}

#[test]
fn test_multiple_bounties_per_guild() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    mint_tokens(&env, &token, &funder, 10000);

    // Create bounties with different rewards
    let desc = String::from_str(&env, "Description");

    let title1 = String::from_str(&env, "Task 1");
    let bounty_id_1 = client.create_bounty(
        &guild_id, &owner, &title1, &desc, &100i128, &token, &2000u64,
    );

    let title2 = String::from_str(&env, "Task 2");
    let bounty_id_2 = client.create_bounty(
        &guild_id, &owner, &title2, &desc, &200i128, &token, &2000u64,
    );

    let title3 = String::from_str(&env, "Task 3");
    let bounty_id_3 = client.create_bounty(
        &guild_id, &owner, &title3, &desc, &300i128, &token, &2000u64,
    );

    let title4 = String::from_str(&env, "Task 4");
    client.create_bounty(
        &guild_id, &owner, &title4, &desc, &400i128, &token, &2000u64,
    );

    let title5 = String::from_str(&env, "Task 5");
    client.create_bounty(
        &guild_id, &owner, &title5, &desc, &500i128, &token, &2000u64,
    );

    // Fund some bounties
    client.fund_bounty(&bounty_id_1, &funder, &100i128);
    client.fund_bounty(&bounty_id_3, &funder, &300i128);

    // Get all bounties
    let bounties = client.get_guild_bounties(&guild_id);
    assert_eq!(bounties.len(), 5);

    // Check funded vs unfunded
    let bounty_0 = client.get_bounty(&bounty_id_1);
    let bounty_1 = client.get_bounty(&bounty_id_2);
    let bounty_2 = client.get_bounty(&bounty_id_3);

    assert_eq!(bounty_0.status, BountyStatus::Open);
    assert_eq!(bounty_1.status, BountyStatus::AwaitingFunds);
    assert_eq!(bounty_2.status, BountyStatus::Open);
}

// ============ Admin Operations Tests ============

#[test]
fn test_admin_can_approve_bounty() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let claimer = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    // Add admin to guild
    client.add_member(&guild_id, &admin, &Role::Admin, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);
    client.claim_bounty(&bounty_id, &claimer);

    let submission = String::from_str(&env, "https://github.com/pr/123");
    client.submit_work(&bounty_id, &submission);

    // Admin approves
    let result = client.approve_completion(&bounty_id, &admin);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Completed);
}

#[test]
fn test_admin_can_cancel_bounty() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let token = create_mock_token(&env, &owner);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    // Add admin to guild
    client.add_member(&guild_id, &admin, &Role::Admin, &owner);

    mint_tokens(&env, &token, &funder, 1000);

    let title = String::from_str(&env, "Task");
    let description = String::from_str(&env, "Description");
    let bounty_id = client.create_bounty(
        &guild_id,
        &owner,
        &title,
        &description,
        &100i128,
        &token,
        &2000u64,
    );

    client.fund_bounty(&bounty_id, &funder, &100i128);

    // Admin cancels
    let result = client.cancel_bounty(&bounty_id, &admin);
    assert_eq!(result, true);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.status, BountyStatus::Cancelled);
}
