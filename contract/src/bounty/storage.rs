use crate::bounty::types::Bounty;
use soroban_sdk::{symbol_short, Env, Map, Symbol, Vec};

// Storage keys
const BOUNTIES_KEY: Symbol = symbol_short!("bounties");
const BOUNTY_CNT_KEY: Symbol = symbol_short!("b_cnt");
const GUILD_BOUNTIES_KEY: Symbol = symbol_short!("g_bnties");

/// Initialize bounty storage
pub fn initialize(env: &Env) {
    if !env.storage().persistent().has(&BOUNTY_CNT_KEY) {
        env.storage().persistent().set(&BOUNTY_CNT_KEY, &0u64);
    }
}

/// Get the next bounty ID and increment
pub fn get_next_bounty_id(env: &Env) -> u64 {
    let counter: u64 = env
        .storage()
        .persistent()
        .get(&BOUNTY_CNT_KEY)
        .unwrap_or(0u64);

    let next_id = counter + 1;
    env.storage().persistent().set(&BOUNTY_CNT_KEY, &next_id);

    next_id
}

/// Store a bounty and update the guild index
pub fn store_bounty(env: &Env, bounty: &Bounty) {
    // 1. Save to main bounties map
    let mut bounties: Map<u64, Bounty> = env
        .storage()
        .persistent()
        .get(&BOUNTIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let is_new = !bounties.contains_key(bounty.id);
    bounties.set(bounty.id, bounty.clone());
    env.storage().persistent().set(&BOUNTIES_KEY, &bounties);

    // 2. Update guild index if it's a new bounty
    if is_new {
        let mut guild_bounties: Map<u64, Vec<u64>> = env
            .storage()
            .persistent()
            .get(&GUILD_BOUNTIES_KEY)
            .unwrap_or_else(|| Map::new(env));

        let mut list = guild_bounties
            .get(bounty.guild_id)
            .unwrap_or_else(|| Vec::new(env));

        list.push_back(bounty.id);
        guild_bounties.set(bounty.guild_id, list);
        env.storage()
            .persistent()
            .set(&GUILD_BOUNTIES_KEY, &guild_bounties);
    }
}

/// Get a bounty by ID
pub fn get_bounty(env: &Env, bounty_id: u64) -> Option<Bounty> {
    let bounties: Map<u64, Bounty> = env
        .storage()
        .persistent()
        .get(&BOUNTIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    bounties.get(bounty_id)
}

/// Get all bounties for a guild
pub fn get_guild_bounties(env: &Env, guild_id: u64) -> Vec<Bounty> {
    let guild_bounties: Map<u64, Vec<u64>> = env
        .storage()
        .persistent()
        .get(&GUILD_BOUNTIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let bounty_ids = guild_bounties
        .get(guild_id)
        .unwrap_or_else(|| Vec::new(env));

    let bounties_map: Map<u64, Bounty> = env
        .storage()
        .persistent()
        .get(&BOUNTIES_KEY)
        .unwrap_or_else(|| Map::new(env));

    let mut result = Vec::new(env);
    for id in bounty_ids.iter() {
        if let Some(b) = bounties_map.get(id) {
            result.push_back(b);
        }
    }
    result
}
