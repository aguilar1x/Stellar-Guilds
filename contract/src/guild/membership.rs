use crate::guild::storage;
use crate::guild::types::{
    Guild, GuildCreatedEvent, Member, MemberAddedEvent, MemberRemovedEvent, Role, RoleUpdatedEvent,
};
use soroban_sdk::{Address, Env, String, Symbol, Vec};

/// Create a new guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `name` - The name of the guild
/// * `description` - The description of the guild
/// * `owner` - The address of the guild owner
///
/// # Returns
/// The ID of the newly created guild
///
/// # Errors
/// Returns Result with error if:
/// - Name or description is too long
pub fn create_guild(
    env: &Env,
    name: String,
    description: String,
    owner: Address,
) -> Result<u64, String> {
    // Validate inputs
    if name.len() == 0 || name.len() > 256 {
        return Err(String::from_str(
            env,
            "Guild name must be between 1 and 256 characters",
        ));
    }
    if description.len() > 512 {
        return Err(String::from_str(
            env,
            "Guild description must be at most 512 characters",
        ));
    }

    // Get next guild ID
    let guild_id = storage::get_next_guild_id(env);

    // Get current timestamp
    let timestamp = env.ledger().timestamp();

    // Create the guild
    let guild = Guild {
        id: guild_id,
        name: name.clone(),
        description,
        owner: owner.clone(),
        created_at: timestamp,
        member_count: 1, // Owner is automatically a member
    };

    // Store the guild
    storage::store_guild(env, &guild);

    // Add owner as a member
    let owner_member = Member {
        address: owner.clone(),
        role: Role::Owner,
        joined_at: timestamp,
    };
    storage::store_member(env, guild_id, &owner_member);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "guild_created"), Symbol::new(env, "v0")),
        GuildCreatedEvent {
            guild_id,
            owner,
            name,
            created_at: timestamp,
        },
    );

    Ok(guild_id)
}

/// Add a member to a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address of the member to add
/// * `role` - The role to assign to the member
/// * `caller` - The address attempting to add the member (must have permission)
///
/// # Returns
/// Result with true if successful, error string if failed
///
/// # Errors
/// Returns Result with error if:
/// - Guild doesn't exist
/// - Member already exists
/// - Caller doesn't have permission to add members
/// - Attempting to add an owner without proper permission
pub fn add_member(
    env: &Env,
    guild_id: u64,
    address: Address,
    role: Role,
    caller: Address,
) -> Result<bool, String> {
    // Get the guild
    let guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    // Check if member already exists
    if storage::has_member(env, guild_id, &address) {
        return Err(String::from_str(env, "Member already exists in guild"));
    }

    // Get caller's role
    let caller_member = storage::get_member(env, guild_id, &caller)
        .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

    // Check permissions based on role being assigned
    match role {
        Role::Owner => {
            // Only current owner can add new owners
            if caller_member.role != Role::Owner {
                return Err(String::from_str(env, "Only owner can add new owners"));
            }
        }
        Role::Admin => {
            // Owner and Admin can add admins
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(env, "Only owner or admin can add admins"));
            }
        }
        Role::Member | Role::Contributor => {
            // Owner and Admin can add members and contributors
            if !caller_member.role.has_permission(&Role::Member) {
                return Err(String::from_str(
                    env,
                    "Insufficient permissions to add members",
                ));
            }
        }
    }

    // Create and store the member
    let timestamp = env.ledger().timestamp();
    let member = Member {
        address: address.clone(),
        role: role.clone(),
        joined_at: timestamp,
    };
    storage::store_member(env, guild_id, &member);

    // Update guild member count
    let mut updated_guild = guild;
    updated_guild.member_count += 1;
    storage::update_guild(env, &updated_guild);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "member_added"), Symbol::new(env, "v0")),
        MemberAddedEvent {
            guild_id,
            address,
            role,
            joined_at: timestamp,
        },
    );

    Ok(true)
}

/// Remove a member from a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address of the member to remove
/// * `caller` - The address attempting to remove the member
///
/// # Returns
/// Result with true if successful, error string if failed
///
/// # Errors
/// Returns Result with error if:
/// - Guild doesn't exist
/// - Member doesn't exist
/// - Caller doesn't have permission
/// - Attempting to remove the last owner
pub fn remove_member(
    env: &Env,
    guild_id: u64,
    address: Address,
    caller: Address,
) -> Result<bool, String> {
    // Get the guild
    let guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    // Check if member exists
    let member = storage::get_member(env, guild_id, &address)
        .ok_or(String::from_str(env, "Member not found"))?;

    // Check if caller is trying to remove themselves (self-removal is allowed)
    let is_self_removal = caller == address;

    // Special case: cannot remove the last owner even via self-removal
    if member.role == Role::Owner {
        let owner_count = storage::count_owners(env, guild_id);
        if owner_count <= 1 {
            return Err(String::from_str(env, "Cannot remove the last owner"));
        }
    }

    if !is_self_removal {
        // Get caller's role
        let caller_member = storage::get_member(env, guild_id, &caller)
            .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

        // Determine permission requirements based on member's role
        match member.role {
            Role::Owner => {
                // Only owners can remove owners
                if caller_member.role != Role::Owner {
                    return Err(String::from_str(env, "Only owner can remove owners"));
                }
                // Prevent removing last owner
                let owner_count = storage::count_owners(env, guild_id);
                if owner_count <= 1 {
                    return Err(String::from_str(env, "Cannot remove the last owner"));
                }
            }
            Role::Admin => {
                // Only owner and admin can remove admins
                if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                    return Err(String::from_str(
                        env,
                        "Only owner or admin can remove admins",
                    ));
                }
            }
            Role::Member | Role::Contributor => {
                // Owner and Admin can remove members and contributors
                if !caller_member.role.has_permission(&Role::Member) {
                    return Err(String::from_str(
                        env,
                        "Insufficient permissions to remove members",
                    ));
                }
            }
        }
    }

    // Remove the member
    storage::remove_member(env, guild_id, &address);

    // Update guild member count
    let mut updated_guild = guild;
    updated_guild.member_count = updated_guild.member_count.saturating_sub(1);
    storage::update_guild(env, &updated_guild);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "member_removed"), Symbol::new(env, "v0")),
        MemberRemovedEvent { guild_id, address },
    );

    Ok(true)
}

/// Update a member's role
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address of the member
/// * `new_role` - The new role to assign
/// * `caller` - The address attempting to update the role
///
/// # Returns
/// Result with true if successful, error string if failed
///
/// # Errors
/// Returns Result with error if:
/// - Guild doesn't exist
/// - Member doesn't exist
/// - Caller doesn't have permission
/// - Attempting to change the last owner's role
pub fn update_role(
    env: &Env,
    guild_id: u64,
    address: Address,
    new_role: Role,
    caller: Address,
) -> Result<bool, String> {
    // Get the guild
    let _guild =
        storage::get_guild(env, guild_id).ok_or(String::from_str(env, "Guild not found"))?;

    // Get the member
    let member = storage::get_member(env, guild_id, &address)
        .ok_or(String::from_str(env, "Member not found"))?;

    // Get caller's role
    let caller_member = storage::get_member(env, guild_id, &caller)
        .ok_or(String::from_str(env, "Caller is not a member of the guild"))?;

    // Check permissions
    match member.role {
        Role::Owner => {
            // Only owners can change owner roles
            if caller_member.role != Role::Owner {
                return Err(String::from_str(env, "Only owner can change owner role"));
            }
            // Prevent changing the last owner to another role
            if new_role != Role::Owner {
                let owner_count = storage::count_owners(env, guild_id);
                if owner_count <= 1 {
                    return Err(String::from_str(env, "Cannot demote the last owner"));
                }
            }
        }
        Role::Admin => {
            // Only owner and admin can change admin roles
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(
                    env,
                    "Only owner or admin can change admin role",
                ));
            }
        }
        Role::Member | Role::Contributor => {
            // Only Owner and Admin can change member/contributor roles
            if caller_member.role != Role::Owner && caller_member.role != Role::Admin {
                return Err(String::from_str(
                    env,
                    "Insufficient permissions to change member role",
                ));
            }
        }
    }

    let old_role = member.role.clone();

    // Update the member's role
    let updated_member = Member {
        address: address.clone(),
        role: new_role.clone(),
        joined_at: member.joined_at,
    };
    storage::store_member(env, guild_id, &updated_member);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "role_updated"), Symbol::new(env, "v0")),
        RoleUpdatedEvent {
            guild_id,
            address,
            old_role,
            new_role,
        },
    );

    Ok(true)
}

/// Get a member from a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address of the member
///
/// # Returns
/// The Member if found, error string if not
pub fn get_member(env: &Env, guild_id: u64, address: Address) -> Result<Member, String> {
    storage::get_member(env, guild_id, &address).ok_or(String::from_str(env, "Member not found"))
}

/// Get all members of a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
///
/// # Returns
/// A vector of all members in the guild
pub fn get_all_members(env: &Env, guild_id: u64) -> Vec<Member> {
    storage::get_all_members(env, guild_id)
}

/// Check if an address is a member of a guild
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address to check
///
/// # Returns
/// True if the address is a member, false otherwise
pub fn is_member(env: &Env, guild_id: u64, address: Address) -> bool {
    storage::has_member(env, guild_id, &address)
}

/// Check if a member has permission for a required role
///
/// # Arguments
/// * `env` - The contract environment
/// * `guild_id` - The ID of the guild
/// * `address` - The address of the member
/// * `required_role` - The required role level
///
/// # Returns
/// True if the member has the required permission, false otherwise
pub fn has_permission(env: &Env, guild_id: u64, address: Address, required_role: Role) -> bool {
    if let Some(member) = storage::get_member(env, guild_id, &address) {
        member.role.has_permission(&required_role)
    } else {
        false
    }
}
