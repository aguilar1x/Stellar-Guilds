//! Payment Distribution Contract Tests
//!
//! Comprehensive test coverage for payment pool creation, recipient management,
//! validation, distribution execution, and batch operations.

use super::*;
use crate::StellarGuildsContract;
use crate::StellarGuildsContractClient;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, String, Vec};

// ============ Test Helpers ============

fn setup_env() -> Env {
    let env = Env::default();
    env.budget().reset_unlimited();
    env
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

// ============ Percentage Distribution Tests ============

#[test]
fn test_create_pool_percentage() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    assert_eq!(pool_id, 1);
}

#[test]
fn test_percentage_distribution_success() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Mint tokens to contract
    mint_tokens(&env, &token_addr, &contract_id, 1000);

    // Create pool
    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Add recipients with percentage shares
    client.add_recipient(&pool_id, &recipient1, &50u32, &creator); // 50%
    client.add_recipient(&pool_id, &recipient2, &30u32, &creator); // 30%
    client.add_recipient(&pool_id, &recipient3, &20u32, &creator); // 20%

    // Validate distribution
    let is_valid = client.validate_distribution(&pool_id);
    assert_eq!(is_valid, true);

    // Check recipient amounts
    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
    let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

    assert_eq!(amount1, 500); // 50% of 1000
    assert_eq!(amount2, 300); // 30% of 1000
    assert_eq!(amount3, 200); // 20% of 1000

    // Execute distribution
    let result = client.execute_distribution(&pool_id, &creator);
    assert_eq!(result, true);

    // Check final balances
    let balance1 = get_token_balance(&env, &token_addr, &recipient1);
    let balance2 = get_token_balance(&env, &token_addr, &recipient2);
    let balance3 = get_token_balance(&env, &token_addr, &recipient3);

    assert_eq!(balance1, 500);
    assert_eq!(balance2, 300);
    assert_eq!(balance3, 200);

    // Check status
    let status = client.get_pool_status(&pool_id);
    assert_eq!(status, DistributionStatus::Executed);
}

#[test]
#[should_panic(expected = "SharesNot100Percent")]
fn test_percentage_not_100_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Add recipients with shares NOT summing to 100
    client.add_recipient(&pool_id, &recipient1, &50u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &30u32, &creator); // Total 80%, not 100%

    // Validation should fail
    client.validate_distribution(&pool_id);
}

#[test]
#[should_panic(expected = "InvalidShare")]
fn test_percentage_over_100_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Try to add recipient with invalid share
    client.add_recipient(&pool_id, &recipient1, &101u32, &creator);
}

#[test]
#[should_panic(expected = "InvalidShare")]
fn test_percentage_zero_share_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Try to add recipient with zero share
    client.add_recipient(&pool_id, &recipient1, &0u32, &creator);
}

// ============ Equal Split Distribution Tests ============

#[test]
fn test_equal_split_distribution_success() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Mint tokens to contract
    mint_tokens(&env, &token_addr, &contract_id, 1000);

    // Create pool with equal split
    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::EqualSplit, &creator);

    // Add recipients (share value doesn't matter for equal split, but must be > 0)
    client.add_recipient(&pool_id, &recipient1, &1u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &1u32, &creator);
    client.add_recipient(&pool_id, &recipient3, &1u32, &creator);

    // Get recipient amounts
    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
    let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

    // Each should get 1000 / 3 = 333 (integer division)
    assert_eq!(amount1, 333);
    assert_eq!(amount2, 333);
    assert_eq!(amount3, 333);

    // Execute distribution
    let result = client.execute_distribution(&pool_id, &creator);
    assert_eq!(result, true);

    // Check final balances
    let balance1 = get_token_balance(&env, &token_addr, &recipient1);
    let balance2 = get_token_balance(&env, &token_addr, &recipient2);
    let balance3 = get_token_balance(&env, &token_addr, &recipient3);

    assert_eq!(balance1, 333);
    assert_eq!(balance2, 333);
    assert_eq!(balance3, 333);
}

#[test]
fn test_equal_split_two_recipients() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::EqualSplit, &creator);

    client.add_recipient(&pool_id, &recipient1, &1u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &1u32, &creator);

    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);

    // Each gets 500
    assert_eq!(amount1, 500);
    assert_eq!(amount2, 500);

    client.execute_distribution(&pool_id, &creator);

    let balance1 = get_token_balance(&env, &token_addr, &recipient1);
    let balance2 = get_token_balance(&env, &token_addr, &recipient2);

    assert_eq!(balance1, 500);
    assert_eq!(balance2, 500);
}

// ============ Weighted Distribution Tests ============

#[test]
fn test_weighted_distribution_success() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    // Create pool with weighted distribution
    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Weighted, &creator);

    // Add recipients with different weights
    client.add_recipient(&pool_id, &recipient1, &5u32, &creator); // Weight 5
    client.add_recipient(&pool_id, &recipient2, &3u32, &creator); // Weight 3
    client.add_recipient(&pool_id, &recipient3, &2u32, &creator); // Weight 2
                                                                  // Total weight = 10

    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
    let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

    // recipient1: 1000 * 5 / 10 = 500
    // recipient2: 1000 * 3 / 10 = 300
    // recipient3: 1000 * 2 / 10 = 200
    assert_eq!(amount1, 500);
    assert_eq!(amount2, 300);
    assert_eq!(amount3, 200);

    client.execute_distribution(&pool_id, &creator);

    let balance1 = get_token_balance(&env, &token_addr, &recipient1);
    let balance2 = get_token_balance(&env, &token_addr, &recipient2);
    let balance3 = get_token_balance(&env, &token_addr, &recipient3);

    assert_eq!(balance1, 500);
    assert_eq!(balance2, 300);
    assert_eq!(balance3, 200);
}

#[test]
fn test_weighted_equal_weights() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Weighted, &creator);

    // Equal weights should behave like equal split
    client.add_recipient(&pool_id, &recipient1, &1u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &1u32, &creator);

    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);

    assert_eq!(amount1, 500);
    assert_eq!(amount2, 500);
}

#[test]
#[should_panic(expected = "InvalidShare")]
fn test_weighted_zero_weight_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Weighted, &creator);

    // Try to add recipient with zero weight
    client.add_recipient(&pool_id, &recipient1, &0u32, &creator);
}

// ============ Authorization and Permission Tests ============

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_add_recipient_non_creator_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Non-creator tries to add recipient
    client.add_recipient(&pool_id, &recipient1, &50u32, &non_creator);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_execute_non_creator_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Non-creator tries to execute
    client.execute_distribution(&pool_id, &non_creator);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_cancel_non_creator_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Non-creator tries to cancel
    client.cancel_distribution(&pool_id, &non_creator);
}

// ============ Duplicate Recipient Tests ============

#[test]
#[should_panic(expected = "DuplicateRecipient")]
fn test_add_duplicate_recipient_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Add recipient
    client.add_recipient(&pool_id, &recipient1, &50u32, &creator);

    // Try to add same recipient again
    client.add_recipient(&pool_id, &recipient1, &50u32, &creator);
}

// ============ Pool Status Tests ============

#[test]
#[should_panic(expected = "PoolNotPending")]
fn test_add_recipient_after_execution_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Execute distribution
    client.execute_distribution(&pool_id, &creator);

    // Try to add recipient after execution
    client.add_recipient(&pool_id, &recipient2, &50u32, &creator);
}

#[test]
#[should_panic(expected = "PoolNotPending")]
fn test_execute_already_executed_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Execute once
    client.execute_distribution(&pool_id, &creator);

    // Try to execute again
    client.execute_distribution(&pool_id, &creator);
}

// ============ Cancellation Tests ============

#[test]
fn test_cancel_pool_success() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Cancel pool
    let result = client.cancel_distribution(&pool_id, &creator);
    assert_eq!(result, true);

    // Check status
    let status = client.get_pool_status(&pool_id);
    assert_eq!(status, DistributionStatus::Cancelled);
}

#[test]
#[should_panic(expected = "PoolNotPending")]
fn test_cancel_after_execution_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Execute
    client.execute_distribution(&pool_id, &creator);

    // Try to cancel after execution
    client.cancel_distribution(&pool_id, &creator);
}

// ============ Batch Distribution Tests ============

#[test]
fn test_batch_distribute_success() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Mint enough tokens for both pools
    mint_tokens(&env, &token_addr, &contract_id, 2000);

    // Create two pools
    let pool_id_1 =
        client.create_payment_pool(&500i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id_1, &recipient1, &100u32, &creator);

    let pool_id_2 =
        client.create_payment_pool(&500i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id_2, &recipient2, &100u32, &creator);

    // Batch distribute
    let mut pool_ids = Vec::new(&env);
    pool_ids.push_back(pool_id_1);
    pool_ids.push_back(pool_id_2);

    let results = client.batch_distribute(&pool_ids, &creator);
    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap(), true);
    assert_eq!(results.get(1).unwrap(), true);

    // Check balances
    let balance1 = get_token_balance(&env, &token_addr, &recipient1);
    let balance2 = get_token_balance(&env, &token_addr, &recipient2);

    assert_eq!(balance1, 500);
    assert_eq!(balance2, 500);
}

#[test]
fn test_batch_distribute_partial_failure() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Only mint enough tokens for one pool
    mint_tokens(&env, &token_addr, &contract_id, 500);

    // Create two pools
    let pool_id_1 =
        client.create_payment_pool(&500i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id_1, &recipient1, &100u32, &creator);

    let pool_id_2 =
        client.create_payment_pool(&500i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id_2, &recipient1, &100u32, &creator);

    // Batch distribute (second should fail due to insufficient balance)
    let mut pool_ids = Vec::new(&env);
    pool_ids.push_back(pool_id_1);
    pool_ids.push_back(pool_id_2);

    let results = client.batch_distribute(&pool_ids, &creator);
    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap(), true); // First succeeds
    assert_eq!(results.get(1).unwrap(), false); // Second fails
}

// ============ Edge Case Tests ============

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_create_pool_zero_amount_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Try to create pool with zero amount
    client.create_payment_pool(&0i128, &token, &DistributionRule::Percentage, &creator);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_create_pool_negative_amount_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Try to create pool with negative amount
    client.create_payment_pool(&-100i128, &token, &DistributionRule::Percentage, &creator);
}

#[test]
#[should_panic(expected = "NoRecipients")]
fn test_execute_no_recipients_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    mint_tokens(&env, &token_addr, &contract_id, 1000);

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);

    // Try to execute without any recipients
    client.execute_distribution(&pool_id, &creator);
}

#[test]
#[should_panic(expected = "InsufficientBalance")]
fn test_execute_insufficient_balance_fails() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Don't mint tokens to contract

    let pool_id =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    client.add_recipient(&pool_id, &recipient1, &100u32, &creator);

    // Try to execute without sufficient balance
    client.execute_distribution(&pool_id, &creator);
}

// ============ Precision and Arithmetic Tests ============

#[test]
fn test_percentage_rounding() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let recipient3 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Use amount that doesn't divide evenly by 3
    let pool_id =
        client.create_payment_pool(&100i128, &token, &DistributionRule::Percentage, &creator);

    // 33%, 33%, 34% = 100%
    client.add_recipient(&pool_id, &recipient1, &33u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &33u32, &creator);
    client.add_recipient(&pool_id, &recipient3, &34u32, &creator);

    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);
    let amount3 = client.get_recipient_amount(&pool_id, &recipient3);

    assert_eq!(amount1, 33);
    assert_eq!(amount2, 33);
    assert_eq!(amount3, 34);
}

#[test]
fn test_large_amounts() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Use large amount
    let large_amount = 1_000_000_000_000i128; // 1 trillion
    let pool_id = client.create_payment_pool(
        &large_amount,
        &token,
        &DistributionRule::Percentage,
        &creator,
    );

    client.add_recipient(&pool_id, &recipient1, &60u32, &creator);
    client.add_recipient(&pool_id, &recipient2, &40u32, &creator);

    let amount1 = client.get_recipient_amount(&pool_id, &recipient1);
    let amount2 = client.get_recipient_amount(&pool_id, &recipient2);

    assert_eq!(amount1, 600_000_000_000);
    assert_eq!(amount2, 400_000_000_000);
}

// ============ Integration Tests ============

#[test]
fn test_multiple_pools_same_creator() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);
    let token = Some(create_mock_token(&env, &creator));

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Create multiple pools
    let pool_id_1 =
        client.create_payment_pool(&1000i128, &token, &DistributionRule::Percentage, &creator);
    let pool_id_2 =
        client.create_payment_pool(&2000i128, &token, &DistributionRule::EqualSplit, &creator);
    let pool_id_3 =
        client.create_payment_pool(&3000i128, &token, &DistributionRule::Weighted, &creator);

    // Add recipients to each
    client.add_recipient(&pool_id_1, &recipient1, &100u32, &creator);

    client.add_recipient(&pool_id_2, &recipient1, &1u32, &creator);
    client.add_recipient(&pool_id_2, &recipient2, &1u32, &creator);

    client.add_recipient(&pool_id_3, &recipient1, &2u32, &creator);
    client.add_recipient(&pool_id_3, &recipient2, &1u32, &creator);

    // Verify pool statuses
    assert_eq!(
        client.get_pool_status(&pool_id_1),
        DistributionStatus::Pending
    );
    assert_eq!(
        client.get_pool_status(&pool_id_2),
        DistributionStatus::Pending
    );
    assert_eq!(
        client.get_pool_status(&pool_id_3),
        DistributionStatus::Pending
    );

    // Verify amounts
    assert_eq!(client.get_recipient_amount(&pool_id_1, &recipient1), 1000);
    assert_eq!(client.get_recipient_amount(&pool_id_2, &recipient1), 1000);
    assert_eq!(client.get_recipient_amount(&pool_id_3, &recipient1), 2000);
}

#[test]
fn test_full_payment_lifecycle() {
    let env = setup_env();
    let creator = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let contributor3 = Address::generate(&env);
    let token_addr = create_mock_token(&env, &creator);
    let token = Some(token_addr.clone());

    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Simulate a completed bounty with 3 contributors splitting reward
    mint_tokens(&env, &token_addr, &contract_id, 10000);

    // Create payment pool with weighted distribution based on contribution
    let pool_id =
        client.create_payment_pool(&10000i128, &token, &DistributionRule::Weighted, &creator);

    // Add contributors with weights based on their contribution
    client.add_recipient(&pool_id, &contributor1, &5u32, &creator); // 50% contribution
    client.add_recipient(&pool_id, &contributor2, &3u32, &creator); // 30% contribution
    client.add_recipient(&pool_id, &contributor3, &2u32, &creator); // 20% contribution

    // Validate before execution
    let is_valid = client.validate_distribution(&pool_id);
    assert_eq!(is_valid, true);

    // Check status
    assert_eq!(
        client.get_pool_status(&pool_id),
        DistributionStatus::Pending
    );

    // Execute distribution
    let result = client.execute_distribution(&pool_id, &creator);
    assert_eq!(result, true);

    // Verify status changed
    assert_eq!(
        client.get_pool_status(&pool_id),
        DistributionStatus::Executed
    );

    // Verify final balances
    let balance1 = get_token_balance(&env, &token_addr, &contributor1);
    let balance2 = get_token_balance(&env, &token_addr, &contributor2);
    let balance3 = get_token_balance(&env, &token_addr, &contributor3);

    assert_eq!(balance1, 5000); // 50%
    assert_eq!(balance2, 3000); // 30%
    assert_eq!(balance3, 2000); // 20%
}
