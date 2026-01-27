use soroban_sdk::{symbol_short, Address, Env, Map, Symbol};

use crate::dispute::types::{Dispute, DisputeReference, Vote};

const DISPUTES_KEY: Symbol = symbol_short!("dsp_all");
const DISPUTE_COUNTER_KEY: Symbol = symbol_short!("dsp_cnt");
const DISPUTE_VOTES_KEY: Symbol = symbol_short!("dsp_vot");
const REF_LOCKS_KEY: Symbol = symbol_short!("dsp_ref");

/// Get the next dispute ID and increment the counter.
pub fn get_next_dispute_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .persistent()
        .get(&DISPUTE_COUNTER_KEY)
        .unwrap_or(0u64);
    let next = current + 1;
    env.storage()
        .persistent()
        .set(&DISPUTE_COUNTER_KEY, &next);
    next
}

/// Persist a dispute record.
pub fn store_dispute(env: &Env, dispute: &Dispute) {
    let mut disputes: Map<u64, Dispute> = env
        .storage()
        .persistent()
        .get(&DISPUTES_KEY)
        .unwrap_or_else(|| Map::new(env));

    disputes.set(dispute.id, dispute.clone());
    env.storage().persistent().set(&DISPUTES_KEY, &disputes);
}

/// Fetch a dispute by ID.
pub fn get_dispute(env: &Env, dispute_id: u64) -> Option<Dispute> {
    let disputes: Map<u64, Dispute> = env
        .storage()
        .persistent()
        .get(&DISPUTES_KEY)
        .unwrap_or_else(|| Map::new(env));

    disputes.get(dispute_id)
}

/// Persist a vote for a dispute.
pub fn store_vote(env: &Env, vote: &Vote) {
    let mut votes_map: Map<u64, Map<Address, Vote>> = env
        .storage()
        .persistent()
        .get(&DISPUTE_VOTES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut dispute_votes = votes_map
        .get(vote.dispute_id)
        .unwrap_or_else(|| Map::new(env));

    dispute_votes.set(vote.voter.clone(), vote.clone());
    votes_map.set(vote.dispute_id, dispute_votes);

    env.storage()
        .persistent()
        .set(&DISPUTE_VOTES_KEY, &votes_map);
}

/// Fetch a vote for a dispute by voter.
pub fn get_vote(env: &Env, dispute_id: u64, voter: &Address) -> Option<Vote> {
    let votes_map: Map<u64, Map<Address, Vote>> = env
        .storage()
        .persistent()
        .get(&DISPUTE_VOTES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let dispute_votes = votes_map.get(dispute_id)?;
    dispute_votes.get(voter.clone())
}

/// Check whether a reference is locked by an active dispute.
pub fn is_reference_locked(env: &Env, reference_type: &DisputeReference, reference_id: u64) -> bool {
    let locks: Map<(DisputeReference, u64), u64> = env
        .storage()
        .persistent()
        .get(&REF_LOCKS_KEY)
        .unwrap_or_else(|| Map::new(env));

    locks.contains_key((reference_type.clone(), reference_id))
}

/// Lock a bounty/milestone reference to prevent concurrent disputes.
pub fn lock_reference(
    env: &Env,
    reference_type: &DisputeReference,
    reference_id: u64,
    dispute_id: u64,
) {
    let mut locks: Map<(DisputeReference, u64), u64> = env
        .storage()
        .persistent()
        .get(&REF_LOCKS_KEY)
        .unwrap_or_else(|| Map::new(env));

    locks.set((reference_type.clone(), reference_id), dispute_id);
    env.storage().persistent().set(&REF_LOCKS_KEY, &locks);
}

/// Unlock a bounty/milestone reference after resolution.
pub fn unlock_reference(env: &Env, reference_type: &DisputeReference, reference_id: u64) {
    let mut locks: Map<(DisputeReference, u64), u64> = env
        .storage()
        .persistent()
        .get(&REF_LOCKS_KEY)
        .unwrap_or_else(|| Map::new(env));

    if locks.contains_key((reference_type.clone(), reference_id)) {
        locks.remove((reference_type.clone(), reference_id));
        env.storage().persistent().set(&REF_LOCKS_KEY, &locks);
    }
}
