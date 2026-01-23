use soroban_sdk::{Address, Env};

use crate::treasury::types::{Transaction, TransactionStatus, TransactionType, Treasury};

pub const TX_EXPIRY_SECONDS: u64 = 60 * 60 * 24 * 7; // 7 days

pub fn validate_threshold(signers_len: u32, threshold: u32) {
    if signers_len == 0 {
        panic!("at least one signer required");
    }
    if threshold == 0 || threshold > signers_len {
        panic!("invalid threshold");
    }
    let half = (signers_len + 1) / 2; // ceil(0.5 * n)
    if threshold < half {
        panic!("threshold must be at least 50% of signers");
    }
}

pub fn assert_signer(env: &Env, treasury: &Treasury, addr: &Address) {
    addr.require_auth();
    if !treasury.is_signer(addr) {
        panic!("caller is not a signer");
    }
}

pub fn has_approved(tx: &Transaction, addr: &Address) -> bool {
    tx.approvals.iter().any(|a| a == addr)
}

pub fn add_approval(tx: &mut Transaction, addr: &Address) {
    if has_approved(tx, addr) {
        panic!("duplicate approval");
    }
    tx.approvals.push_back(addr.clone());
}

pub fn required_approvals_for_tx(treasury: &Treasury, tx: &Transaction) -> u32 {
    match tx.tx_type {
        TransactionType::Withdrawal | TransactionType::BountyFunding | TransactionType::MilestonePayment => {
            if tx.amount >= treasury.high_value_threshold {
                treasury.approval_threshold
            } else {
                // low-value operations: single signer is enough, but cannot exceed threshold
                1u32.min(treasury.approval_threshold)
            }
        }
        _ => 1,
    }
}

pub fn is_expired(tx: &Transaction, now: u64) -> bool {
    now >= tx.expires_at
}

pub fn expire_if_needed(tx: &mut Transaction, now: u64) {
    if matches!(tx.status, TransactionStatus::Pending | TransactionStatus::Approved)
        && is_expired(tx, now)
    {
        tx.status = TransactionStatus::Expired;
    }
}
