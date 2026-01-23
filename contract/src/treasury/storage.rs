use soroban_sdk::{symbol_short, Address, Env, Map, String, Symbol, Vec};

use crate::treasury::types::{Allowance, Budget, Transaction, Treasury};

const TREASURY_CNT_KEY: Symbol = symbol_short!("t_cnt");
const TREASURIES_KEY: Symbol = symbol_short!("treasuries");

const TX_CNT_KEY: Symbol = symbol_short!("tx_cnt");
const TRANSACTIONS_KEY: Symbol = symbol_short!("txs");
const TREASURY_TX_INDEX_KEY: Symbol = symbol_short!("t_tx_idx");

const BUDGETS_KEY: Symbol = symbol_short!("budgets");
const ALLOWANCES_KEY: Symbol = symbol_short!("allows");

pub fn initialize_treasury_storage(env: &Env) {
    let storage = env.storage().persistent();
    if !storage.has(&TREASURY_CNT_KEY) {
        storage.set(&TREASURY_CNT_KEY, &0u64);
    }
    if !storage.has(&TX_CNT_KEY) {
        storage.set(&TX_CNT_KEY, &0u64);
    }
}

pub fn get_next_treasury_id(env: &Env) -> u64 {
    let storage = env.storage().persistent();
    let current: u64 = storage.get(&TREASURY_CNT_KEY).unwrap_or(0u64);
    let next = current + 1;
    storage.set(&TREASURY_CNT_KEY, &next);
    next
}

pub fn get_next_tx_id(env: &Env) -> u64 {
    let storage = env.storage().persistent();
    let current: u64 = storage.get(&TX_CNT_KEY).unwrap_or(0u64);
    let next = current + 1;
    storage.set(&TX_CNT_KEY, &next);
    next
}

pub fn store_treasury(env: &Env, treasury: &Treasury) {
    let mut treasuries: Map<u64, Treasury> = env
        .storage()
        .persistent()
        .get(&TREASURIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    treasuries.set(treasury.id, treasury.clone());
    env.storage().persistent().set(&TREASURIES_KEY, &treasuries);
}

pub fn get_treasury(env: &Env, id: u64) -> Option<Treasury> {
    let treasuries: Map<u64, Treasury> = env
        .storage()
        .persistent()
        .get(&TREASURIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    treasuries.get(id)
}

pub fn store_transaction(env: &Env, tx: &Transaction) {
    // Store main tx map
    let mut txs: Map<u64, Transaction> = env
        .storage()
        .persistent()
        .get(&TRANSACTIONS_KEY)
        .unwrap_or_else(|| Map::new(env));

    txs.set(tx.id, tx.clone());
    env.storage().persistent().set(&TRANSACTIONS_KEY, &txs);

    // Update treasury index
    let mut index: Map<u64, Vec<u64>> = env
        .storage()
        .persistent()
        .get(&TREASURY_TX_INDEX_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut list = index.get(tx.treasury_id).unwrap_or_else(|| Vec::new(env));
    list.push_back(tx.id);
    index.set(tx.treasury_id, list);
    env.storage().persistent().set(&TREASURY_TX_INDEX_KEY, &index);
}

pub fn get_transaction(env: &Env, tx_id: u64) -> Option<Transaction> {
    let txs: Map<u64, Transaction> = env
        .storage()
        .persistent()
        .get(&TRANSACTIONS_KEY)
        .unwrap_or_else(|| Map::new(env));

    txs.get(tx_id)
}

pub fn get_treasury_transactions(env: &Env, treasury_id: u64) -> Vec<Transaction> {
    let index: Map<u64, Vec<u64>> = env
        .storage()
        .persistent()
        .get(&TREASURY_TX_INDEX_KEY)
        .unwrap_or_else(|| Map::new(env));

    let ids = index.get(treasury_id).unwrap_or_else(|| Vec::new(env));

    let txs: Map<u64, Transaction> = env
        .storage()
        .persistent()
        .get(&TRANSACTIONS_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for id in ids.iter() {
        if let Some(tx) = txs.get(id) {
            result.push_back(tx);
        }
    }
    result
}

pub fn get_budget(env: &Env, treasury_id: u64, category: &String) -> Option<Budget> {
    let budgets: Map<(u64, String), Budget> = env
        .storage()
        .persistent()
        .get(&BUDGETS_KEY)
        .unwrap_or_else(|| Map::new(env));

    budgets.get((treasury_id, category.clone()))
}

pub fn store_budget(env: &Env, budget: &Budget) {
    let mut budgets: Map<(u64, String), Budget> = env
        .storage()
        .persistent()
        .get(&BUDGETS_KEY)
        .unwrap_or_else(|| Map::new(env));

    budgets.set((budget.treasury_id, budget.category.clone()), budget.clone());
    env.storage().persistent().set(&BUDGETS_KEY, &budgets);
}

pub fn get_allowance(
    env: &Env,
    treasury_id: u64,
    admin: &Address,
    token: &Option<Address>,
) -> Option<Allowance> {
    let allowances: Map<(u64, Address, Option<Address>), Allowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));

    allowances.get((treasury_id, admin.clone(), token.clone()))
}

pub fn store_allowance(env: &Env, allowance: &Allowance) {
    let mut allowances: Map<(u64, Address, Option<Address>), Allowance> = env
        .storage()
        .persistent()
        .get(&ALLOWANCES_KEY)
        .unwrap_or_else(|| Map::new(env));

    allowances.set(
        (allowance.treasury_id, allowance.admin.clone(), allowance.token.clone()),
        allowance.clone(),
    );
    env.storage().persistent().set(&ALLOWANCES_KEY, &allowances);
}

pub fn list_budgets_for_treasury(env: &Env, treasury_id: u64) -> Vec<Budget> {
    let budgets: Map<(u64, String), Budget> = env
        .storage()
        .persistent()
        .get(&BUDGETS_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for entry in budgets.iter() {
        let ((t_id, _category), budget) = entry;
        if t_id == treasury_id {
            result.push_back(budget);
        }
    }
    result
}
