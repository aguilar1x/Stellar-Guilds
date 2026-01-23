use soroban_sdk::{token::Client as TokenClient, Address, Env, String, Vec};

use crate::treasury::multisig::{add_approval, assert_signer, expire_if_needed, required_approvals_for_tx, validate_threshold, TX_EXPIRY_SECONDS};
use crate::treasury::storage::{get_allowance, get_budget, get_next_treasury_id, get_next_tx_id, get_treasury, get_treasury_transactions, store_allowance, store_budget, store_transaction, store_treasury};
use crate::treasury::types::{Allowance, Budget, DepositEvent, EmergencyPauseEvent, Transaction, TransactionApprovedEvent, TransactionExecutedEvent, TransactionStatus, TransactionType, Treasury, TreasuryInitializedEvent, WithdrawalProposedEvent};

pub fn initialize_treasury(
    env: &Env,
    guild_id: u64,
    owner: Address,
    mut signers: Vec<Address>,
    approval_threshold: u32,
    high_value_threshold: i128,
) -> u64 {
    owner.require_auth();

    // ensure owner is a signer
    if !signers.iter().any(|a| a == &owner) {
        signers.push_back(owner.clone());
    }

    // deduplicate signers
    let mut unique: Vec<Address> = Vec::new(env);
    for addr in signers.iter() {
        if !unique.iter().any(|a| a == &addr) {
            unique.push_back(addr);
        }
    }

    let signers_len = unique.len() as u32;
    validate_threshold(signers_len, approval_threshold);

    let id = get_next_treasury_id(env);

    let treasury = Treasury {
        id,
        guild_id,
        owner: owner.clone(),
        signers: unique,
        approval_threshold,
        high_value_threshold,
        balance_xlm: 0,
        token_balances: soroban_sdk::Map::new(env),
        total_deposits: 0,
        total_withdrawals: 0,
        paused: false,
    };

    store_treasury(env, &treasury);

    let event = TreasuryInitializedEvent {
        treasury_id: id,
        guild_id,
        owner,
    };
    env.events().publish((b"treasury", b"init"), event);

    id
}

pub fn deposit(
    env: &Env,
    treasury_id: u64,
    depositor: Address,
    amount: i128,
    token: Option<Address>,
) -> bool {
    depositor.require_auth();
    if amount <= 0 {
        panic!("amount must be positive");
    }

    let mut treasury = get_treasury(env, treasury_id).expect("treasury not found");
    if treasury.paused {
        panic!("treasury is paused");
    }

    match token {
        Some(ref token_addr) => {
            let client = TokenClient::new(env, token_addr);
            client.transfer(&depositor, &env.current_contract_address(), &amount);

            let mut balances = treasury.token_balances.clone();
            let current = balances.get(token_addr).unwrap_or(0i128);
            balances.set(token_addr.clone(), current + amount);
            treasury.token_balances = balances;
        }
        None => {
            // For native XLM we assume a wrapped token or external transfer; we only track accounting here.
            treasury.balance_xlm += amount;
        }
    }

    treasury.total_deposits += amount;
    store_treasury(env, &treasury);

    let tx_id = get_next_tx_id(env);
    let now = env.ledger().timestamp();
    let tx = Transaction {
        id: tx_id,
        treasury_id,
        tx_type: TransactionType::Deposit,
        amount,
        token: token.clone(),
        recipient: Some(env.current_contract_address()),
        proposer: depositor.clone(),
        approvals: Vec::new(env),
        status: TransactionStatus::Executed,
        created_at: now,
        expires_at: now,
        reason: String::from_str(env, "deposit"),
    };
    store_transaction(env, &tx);

    let event = DepositEvent {
        treasury_id,
        from: depositor,
        amount,
        token,
    };
    env.events().publish((b"treasury", b"deposit"), event);

    true
}

pub fn propose_withdrawal(
    env: &Env,
    treasury_id: u64,
    proposer: Address,
    recipient: Address,
    amount: i128,
    token: Option<Address>,
    reason: String,
) -> u64 {
    if amount <= 0 {
        panic!("amount must be positive");
    }

    let mut treasury = get_treasury(env, treasury_id).expect("treasury not found");
    if treasury.paused {
        panic!("treasury is paused");
    }

    assert_signer(env, &treasury, &proposer);

    let tx_id = get_next_tx_id(env);
    let now = env.ledger().timestamp();
    let mut approvals = Vec::new(env);
    approvals.push_back(proposer.clone());

    let tx = Transaction {
        id: tx_id,
        treasury_id,
        tx_type: TransactionType::Withdrawal,
        amount,
        token: token.clone(),
        recipient: Some(recipient.clone()),
        proposer: proposer.clone(),
        approvals,
        status: TransactionStatus::Pending,
        created_at: now,
        expires_at: now + TX_EXPIRY_SECONDS,
        reason,
    };
    store_transaction(env, &tx);

    let event = WithdrawalProposedEvent {
        treasury_id,
        tx_id,
        proposer,
        recipient,
        amount,
        token,
    };
    env.events().publish((b"treasury", b"withdraw_proposed"), event);

    tx_id
}

pub fn approve_transaction(env: &Env, tx_id: u64, approver: Address) -> bool {
    approver.require_auth();

    let mut tx = crate::treasury::storage::get_transaction(env, tx_id).expect("tx not found");
    let mut treasury = get_treasury(env, tx.treasury_id).expect("treasury not found");

    let now = env.ledger().timestamp();
    expire_if_needed(&mut tx, now);
    if matches!(tx.status, TransactionStatus::Rejected | TransactionStatus::Executed | TransactionStatus::Expired) {
        panic!("transaction not approvable");
    }

    assert_signer(env, &treasury, &approver);
    add_approval(&mut tx, &approver);

    let required = required_approvals_for_tx(&treasury, &tx);
    if (tx.approvals.len() as u32) >= required {
        tx.status = TransactionStatus::Approved;
    }

    store_transaction(env, &tx);

    let event = TransactionApprovedEvent {
        treasury_id: tx.treasury_id,
        tx_id,
        approver,
    };
    env.events().publish((b"treasury", b"tx_approved"), event);

    true
}

fn enforce_budget(env: &Env, treasury_id: u64, category: &String, amount: i128) {
    if amount <= 0 {
        return;
    }
    let now = env.ledger().timestamp();
    let mut budget = get_budget(env, treasury_id, category).unwrap_or(Budget {
        treasury_id,
        category: category.clone(),
        allocated_amount: 0,
        spent_amount: 0,
        period_seconds: 0,
        period_start: now,
    });

    if budget.period_seconds > 0
        && now >= budget.period_start.saturating_add(budget.period_seconds)
    {
        budget.period_start = now;
        budget.spent_amount = 0;
    }

    if budget.allocated_amount > 0
        && budget.spent_amount + amount > budget.allocated_amount
    {
        panic!("budget exceeded");
    }

    budget.spent_amount += amount;
    store_budget(env, &budget);
}

fn enforce_allowance(env: &Env, treasury_id: u64, admin: &Address, token: &Option<Address>, amount: i128) {
    if amount <= 0 {
        return;
    }

    if let Some(mut allowance) = get_allowance(env, treasury_id, admin, token) {
        allowance.ensure_period_current(env);
        if allowance.remaining_amount < amount {
            panic!("allowance exceeded");
        }
        allowance.remaining_amount -= amount;
        store_allowance(env, &allowance);
    }
}

pub fn execute_transaction(env: &Env, tx_id: u64, executor: Address) -> bool {
    executor.require_auth();

    let mut tx = crate::treasury::storage::get_transaction(env, tx_id).expect("tx not found");
    let mut treasury = get_treasury(env, tx.treasury_id).expect("treasury not found");

    let now = env.ledger().timestamp();
    expire_if_needed(&mut tx, now);
    if matches!(tx.status, TransactionStatus::Rejected | TransactionStatus::Executed | TransactionStatus::Expired) {
        panic!("transaction not executable");
    }

    // when paused, only already-approved transactions may be executed
    if treasury.paused && !matches!(tx.status, TransactionStatus::Approved) {
        panic!("treasury is paused");
    }

    // require signer to execute
    assert_signer(env, &treasury, &executor);

    if !matches!(tx.status, TransactionStatus::Approved) {
        panic!("transaction must be approved");
    }

    match tx.tx_type {
        TransactionType::Withdrawal
        | TransactionType::BountyFunding
        | TransactionType::MilestonePayment => {
            let recipient = tx.recipient.clone().expect("recipient required");

            // budget category name from tx_type
            let category = match tx.tx_type {
                TransactionType::Withdrawal => String::from_str(env, "withdrawal"),
                TransactionType::BountyFunding => String::from_str(env, "bounty"),
                TransactionType::MilestonePayment => String::from_str(env, "milestone"),
                _ => String::from_str(env, "other"),
            };

            enforce_budget(env, tx.treasury_id, &category, tx.amount);
            enforce_allowance(env, tx.treasury_id, &executor, &tx.token, tx.amount);

            match tx.token {
                Some(ref token_addr) => {
                    let client = TokenClient::new(env, token_addr);

                    let mut balances = treasury.token_balances.clone();
                    let current = balances.get(token_addr).unwrap_or(0i128);
                    if current < tx.amount {
                        panic!("insufficient treasury balance");
                    }
                    balances.set(token_addr.clone(), current - tx.amount);
                    treasury.token_balances = balances;

                    client.transfer(&env.current_contract_address(), &recipient, &tx.amount);
                }
                None => {
                    if treasury.balance_xlm < tx.amount {
                        panic!("insufficient XLM balance");
                    }
                    treasury.balance_xlm -= tx.amount;
                }
            }

            treasury.total_withdrawals += tx.amount;
            store_treasury(env, &treasury);
        }
        TransactionType::Deposit => {
            panic!("cannot execute deposit transaction");
        }
        TransactionType::AllowanceGrant => {
            // state-only; execution path not used in this simplified version
        }
    }

    tx.status = TransactionStatus::Executed;
    store_transaction(env, &tx);

    let event = TransactionExecutedEvent {
        treasury_id: tx.treasury_id,
        tx_id,
    };
    env.events().publish((b"treasury", b"tx_executed"), event);

    true
}

pub fn set_budget(
    env: &Env,
    treasury_id: u64,
    caller: Address,
    category: String,
    amount: i128,
    period_seconds: u64,
) -> bool {
    let treasury = get_treasury(env, treasury_id).expect("treasury not found");
    assert_signer(env, &treasury, &caller);

    let now = env.ledger().timestamp();
    let mut budget = get_budget(env, treasury_id, &category).unwrap_or(Budget {
        treasury_id,
        category: category.clone(),
        allocated_amount: 0,
        spent_amount: 0,
        period_seconds,
        period_start: now,
    });

    if budget.period_seconds != period_seconds {
        budget.period_seconds = period_seconds;
    }

    if now >= budget.period_start.saturating_add(budget.period_seconds) {
        budget.period_start = now;
        budget.spent_amount = 0;
    }

    budget.allocated_amount = amount;
    store_budget(env, &budget);

    let event = crate::treasury::types::BudgetUpdatedEvent {
        treasury_id,
        category,
        allocated_amount: amount,
        period_seconds,
    };
    env.events().publish((b"treasury", b"budget"), event);

    true
}

pub fn get_balance(env: &Env, treasury_id: u64, token: Option<Address>) -> i128 {
    let treasury = get_treasury(env, treasury_id).expect("treasury not found");
    match token {
        Some(token_addr) => treasury
            .token_balances
            .get(token_addr)
            .unwrap_or(0i128),
        None => treasury.balance_xlm,
    }
}

pub fn get_transaction_history(env: &Env, treasury_id: u64, limit: u32) -> Vec<Transaction> {
    let all = get_treasury_transactions(env, treasury_id);
    let len = all.len();
    let limit_usize = limit as usize;

    if len <= limit_usize {
        return all;
    }

    let start = len - limit_usize;
    let mut result = Vec::new(env);
    for (idx, tx) in all.iter().enumerate() {
        if idx >= start {
            result.push_back(tx);
        }
    }
    result
}

pub fn grant_allowance(
    env: &Env,
    treasury_id: u64,
    owner: Address,
    admin: Address,
    amount: i128,
    token: Option<Address>,
    period_seconds: u64,
) -> bool {
    let treasury = get_treasury(env, treasury_id).expect("treasury not found");

    if treasury.owner != owner {
        panic!("only owner can grant allowance");
    }
    owner.require_auth();

    if !treasury.is_signer(&admin) {
        panic!("admin must be signer");
    }

    let now = env.ledger().timestamp();
    let mut allowance = get_allowance(env, treasury_id, &admin, &token).unwrap_or(Allowance {
        treasury_id,
        admin: admin.clone(),
        token: token.clone(),
        amount_per_period: amount,
        remaining_amount: amount,
        period_seconds,
        period_start: now,
    });

    allowance.amount_per_period = amount;
    allowance.period_seconds = period_seconds;
    allowance.period_start = now;
    allowance.remaining_amount = amount;

    store_allowance(env, &allowance);

    let event = crate::treasury::types::AllowanceGrantedEvent {
        treasury_id,
        admin,
        token,
        amount_per_period: amount,
        period_seconds,
    };
    env.events().publish((b"treasury", b"allow"), event);

    true
}

pub fn emergency_pause(env: &Env, treasury_id: u64, signer: Address, paused: bool) -> bool {
    let mut treasury = get_treasury(env, treasury_id).expect("treasury not found");
    assert_signer(env, &treasury, &signer);

    treasury.paused = paused;
    store_treasury(env, &treasury);

    let event = EmergencyPauseEvent {
        treasury_id,
        paused,
    };
    env.events().publish((b"treasury", b"pause"), event);

    true
}
