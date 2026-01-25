use soroban_sdk::{Address, Env, String, Vec};

use crate::guild::membership::has_permission;
use crate::guild::types::Role;
use crate::milestone::storage::{
    append_milestone_to_project, get_milestone, get_next_milestone_id, get_next_project_id,
    get_project, get_project_milestone_ids, store_milestone, store_project,
};
use crate::milestone::types::{
    Milestone, MilestoneAddedEvent, MilestoneInput, MilestonePaymentReleasedEvent,
    MilestoneRejectedEvent, MilestoneStatus, MilestoneStatusChangedEvent, MilestoneSubmittedEvent,
    Project, ProjectCreatedEvent, ProjectStatus, ProjectStatusChangedEvent,
};
use crate::treasury::execute_milestone_payment;

fn assert_project_active(project: &Project, env: &Env) {
    if project.status != ProjectStatus::Active {
        panic!("project is not active");
    }

    // Basic safety: budget sanity
    if project.allocated_amount > project.total_amount {
        panic!("project over-allocated");
    }
}

fn ensure_not_expired(env: &Env, milestone: &mut Milestone) {
    let now = env.ledger().timestamp();
    if now > milestone.deadline && milestone.status != MilestoneStatus::Approved {
        milestone.status = MilestoneStatus::Expired;
        store_milestone(env, milestone);
        panic!("milestone expired");
    }
}

pub fn create_project(
    env: &Env,
    guild_id: u64,
    contributor: Address,
    milestones: Vec<MilestoneInput>,
    total_amount: i128,
    treasury_id: u64,
    token: Option<Address>,
    is_sequential: bool,
) -> u64 {
    contributor.require_auth();

    if total_amount <= 0 {
        panic!("total_amount must be positive");
    }
    if milestones.is_empty() {
        panic!("at least one milestone required");
    }

    let now = env.ledger().timestamp();

    // Validate milestones and compute allocation
    let mut allocated: i128 = 0;
    for input in milestones.iter() {
        if input.payment_amount <= 0 {
            panic!("milestone payment must be positive");
        }
        if input.title.len() == 0 || input.title.len() > 256 {
            panic!("milestone title length invalid");
        }
        if input.description.len() > 1024 {
            panic!("milestone description too long");
        }
        if input.deadline <= now {
            panic!("milestone deadline must be in the future");
        }
        allocated = allocated
            .checked_add(input.payment_amount)
            .expect("overflow");
    }

    if allocated > total_amount {
        panic!("allocated milestone budget exceeds project total");
    }

    let project_id = get_next_project_id(env);

    let project = Project {
        id: project_id,
        guild_id,
        contributor: contributor.clone(),
        treasury_id,
        token: token.clone(),
        total_amount,
        allocated_amount: allocated,
        released_amount: 0,
        is_sequential,
        created_at: now,
        status: ProjectStatus::Active,
    };

    store_project(env, &project);

    // Create milestones
    let mut order: u32 = 1;
    for input in milestones.iter() {
        let milestone_id = get_next_milestone_id(env);
        let milestone = Milestone {
            id: milestone_id,
            project_id,
            order,
            title: input.title.clone(),
            description: input.description.clone(),
            payment_amount: input.payment_amount,
            deadline: input.deadline,
            status: MilestoneStatus::Pending,
            proof_url: String::from_str(env, ""),
            created_at: now,
            submitted_at: None,
            last_updated_at: now,
            version: 0,
            is_payment_released: false,
        };
        store_milestone(env, &milestone);
        append_milestone_to_project(env, project_id, milestone_id);

        let event = MilestoneAddedEvent {
            project_id,
            milestone_id,
            title: milestone.title.clone(),
            payment_amount: milestone.payment_amount,
            deadline: milestone.deadline,
        };
        env.events().publish(("MilestoneAdded",), event);

        order += 1;
    }

    let project_event = ProjectCreatedEvent {
        project_id,
        guild_id,
        contributor,
        treasury_id,
        token,
        total_amount,
        is_sequential,
    };
    env.events().publish(("ProjectCreated",), project_event);

    project_id
}

pub fn add_milestone(
    env: &Env,
    project_id: u64,
    title: String,
    description: String,
    amount: i128,
    deadline: u64,
    caller: Address,
) -> u64 {
    caller.require_auth();

    let mut project = get_project(env, project_id).expect("project not found");
    assert_project_active(&project, env);

    // Only guild admins can add milestones
    if !has_permission(env, project.guild_id, caller, Role::Admin) {
        panic!("caller must be guild admin");
    }

    let now = env.ledger().timestamp();
    if amount <= 0 {
        panic!("amount must be positive");
    }
    if deadline <= now {
        panic!("deadline must be in the future");
    }
    if title.len() == 0 || title.len() > 256 {
        panic!("milestone title length invalid");
    }
    if description.len() > 1024 {
        panic!("milestone description too long");
    }

    let new_allocated = project
        .allocated_amount
        .checked_add(amount)
        .expect("overflow");
    if new_allocated > project.total_amount {
        panic!("allocated milestone budget exceeds project total");
    }
    project.allocated_amount = new_allocated;
    store_project(env, &project);

    let milestone_id = get_next_milestone_id(env);
    let order = get_project_milestone_ids(env, project_id).len() as u32 + 1;

    let milestone = Milestone {
        id: milestone_id,
        project_id,
        order,
        title: title.clone(),
        description: description.clone(),
        payment_amount: amount,
        deadline,
        status: MilestoneStatus::Pending,
        proof_url: String::from_str(env, ""),
        created_at: now,
        submitted_at: None,
        last_updated_at: now,
        version: 0,
        is_payment_released: false,
    };

    store_milestone(env, &milestone);
    append_milestone_to_project(env, project_id, milestone_id);

    let event = MilestoneAddedEvent {
        project_id,
        milestone_id,
        title,
        payment_amount: amount,
        deadline,
    };
    env.events().publish(("MilestoneAdded",), event);

    milestone_id
}

pub fn start_milestone(env: &Env, milestone_id: u64, contributor: Address) -> bool {
    contributor.require_auth();

    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let project = get_project(env, milestone.project_id).expect("project not found");

    assert_project_active(&project, env);
    ensure_not_expired(env, &mut milestone);

    if contributor != project.contributor {
        panic!("only project contributor can start milestone");
    }

    if milestone.status != MilestoneStatus::Pending {
        panic!("milestone not pending");
    }

    if project.is_sequential {
        let ids = get_project_milestone_ids(env, project.id);
        for id in ids.iter() {
            let other = get_milestone(env, id).expect("milestone missing");
            if other.order + 1 == milestone.order {
                if other.status != MilestoneStatus::Approved
                    && other.status != MilestoneStatus::Expired
                {
                    panic!("previous milestone not completed");
                }
            }
        }
    }

    let old_status = milestone.status.clone();
    milestone.status = MilestoneStatus::InProgress;
    milestone.last_updated_at = env.ledger().timestamp();
    store_milestone(env, &milestone);

    let event = MilestoneStatusChangedEvent {
        project_id: project.id,
        milestone_id,
        old_status,
        new_status: milestone.status.clone(),
    };
    env.events().publish(("MilestoneStatusChanged",), event);

    true
}

pub fn submit_milestone(env: &Env, milestone_id: u64, proof_url: String) -> bool {
    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let project = get_project(env, milestone.project_id).expect("project not found");

    assert_project_active(&project, env);
    ensure_not_expired(env, &mut milestone);

    if proof_url.len() == 0 || proof_url.len() > 1024 {
        panic!("invalid proof url");
    }

    if milestone.status != MilestoneStatus::InProgress
        && milestone.status != MilestoneStatus::Rejected
    {
        panic!("milestone not in progress or previously rejected");
    }

    let now = env.ledger().timestamp();
    let old_status = milestone.status.clone();
    milestone.status = MilestoneStatus::Submitted;
    milestone.proof_url = proof_url.clone();
    milestone.submitted_at = Some(now);
    milestone.last_updated_at = now;
    milestone.version = milestone.version.saturating_add(1);
    store_milestone(env, &milestone);

    let submit_event = MilestoneSubmittedEvent {
        project_id: project.id,
        milestone_id,
        proof_url,
        version: milestone.version,
    };
    env.events().publish(("MilestoneSubmitted",), submit_event);

    let status_event = MilestoneStatusChangedEvent {
        project_id: project.id,
        milestone_id,
        old_status,
        new_status: milestone.status.clone(),
    };
    env.events()
        .publish(("MilestoneStatusChanged",), status_event);

    true
}

pub fn approve_milestone(env: &Env, milestone_id: u64, approver: Address) -> bool {
    approver.require_auth();

    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let mut project = get_project(env, milestone.project_id).expect("project not found");

    assert_project_active(&project, env);
    ensure_not_expired(env, &mut milestone);

    if !has_permission(env, project.guild_id, approver, Role::Admin) {
        panic!("approver must be guild admin");
    }

    if milestone.status != MilestoneStatus::Submitted {
        panic!("milestone not submitted");
    }

    let old_status = milestone.status.clone();
    milestone.status = MilestoneStatus::Approved;
    milestone.last_updated_at = env.ledger().timestamp();
    store_milestone(env, &milestone);

    let status_event = MilestoneStatusChangedEvent {
        project_id: project.id,
        milestone_id,
        old_status,
        new_status: milestone.status.clone(),
    };
    env.events()
        .publish(("MilestoneStatusChanged",), status_event);

    // Automatic payment release (Option B via treasury)
    let _ = release_milestone_payment_internal(env, &mut project, &mut milestone);

    true
}

pub fn reject_milestone(env: &Env, milestone_id: u64, approver: Address, reason: String) -> bool {
    approver.require_auth();

    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let project = get_project(env, milestone.project_id).expect("project not found");

    assert_project_active(&project, env);
    ensure_not_expired(env, &mut milestone);

    if !has_permission(env, project.guild_id, approver, Role::Admin) {
        panic!("approver must be guild admin");
    }

    if milestone.status != MilestoneStatus::Submitted {
        panic!("milestone not submitted");
    }

    if reason.len() == 0 || reason.len() > 512 {
        panic!("invalid rejection reason");
    }

    let old_status = milestone.status.clone();
    milestone.status = MilestoneStatus::Rejected;
    milestone.last_updated_at = env.ledger().timestamp();
    store_milestone(env, &milestone);

    let reject_event = MilestoneRejectedEvent {
        project_id: project.id,
        milestone_id,
        reason,
    };
    env.events().publish(("MilestoneRejected",), reject_event);

    let status_event = MilestoneStatusChangedEvent {
        project_id: project.id,
        milestone_id,
        old_status,
        new_status: milestone.status.clone(),
    };
    env.events()
        .publish(("MilestoneStatusChanged",), status_event);

    true
}

pub fn get_project_progress(env: &Env, project_id: u64) -> (u32, u32, u32) {
    let ids = get_project_milestone_ids(env, project_id);
    let total = ids.len() as u32;
    if total == 0 {
        return (0, 0, 0);
    }

    let mut completed: u32 = 0;
    for id in ids.iter() {
        if let Some(m) = get_milestone(env, id) {
            if m.status == MilestoneStatus::Approved {
                completed += 1;
            }
        }
    }

    let percentage = completed.saturating_mul(100) / total;
    (completed, total, percentage)
}

pub fn get_milestone_view(env: &Env, milestone_id: u64) -> Milestone {
    get_milestone(env, milestone_id).expect("milestone not found")
}

pub fn release_milestone_payment(env: &Env, milestone_id: u64) -> bool {
    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let mut project = get_project(env, milestone.project_id).expect("project not found");

    assert_project_active(&project, env);

    release_milestone_payment_internal(env, &mut project, &mut milestone)
}

fn release_milestone_payment_internal(
    env: &Env,
    project: &mut Project,
    milestone: &mut Milestone,
) -> bool {
    if milestone.status != MilestoneStatus::Approved {
        panic!("milestone not approved");
    }
    if milestone.is_payment_released {
        panic!("milestone payment already released");
    }

    let new_released = project
        .released_amount
        .checked_add(milestone.payment_amount)
        .expect("overflow");
    if new_released > project.total_amount {
        panic!("project budget exceeded");
    }

    // Execute payment via treasury helper (Option B)
    let token = project.token.clone();
    let amount = milestone.payment_amount;

    execute_milestone_payment(
        env,
        project.treasury_id,
        token.clone(),
        project.contributor.clone(),
        amount,
    );

    project.released_amount = new_released;
    milestone.is_payment_released = true;
    milestone.last_updated_at = env.ledger().timestamp();

    store_project(env, project);
    store_milestone(env, milestone);

    let event = MilestonePaymentReleasedEvent {
        project_id: project.id,
        milestone_id: milestone.id,
        treasury_id: project.treasury_id,
        amount,
        token,
        recipient: project.contributor.clone(),
    };
    env.events().publish(("MilestonePaymentReleased",), event);

    // If all milestones are completed, mark project as completed
    let ids = get_project_milestone_ids(env, project.id);
    let mut all_done = true;
    for id in ids.iter() {
        if let Some(m) = get_milestone(env, id) {
            if !m.is_payment_released && m.status != MilestoneStatus::Expired {
                all_done = false;
                break;
            }
        }
    }

    if all_done && project.status != ProjectStatus::Completed {
        let old_status = project.status.clone();
        project.status = ProjectStatus::Completed;
        store_project(env, project);

        let pe = ProjectStatusChangedEvent {
            project_id: project.id,
            old_status,
            new_status: ProjectStatus::Completed,
        };
        env.events().publish(("ProjectStatusChanged",), pe);
    }

    true
}

pub fn extend_milestone_deadline(
    env: &Env,
    milestone_id: u64,
    new_deadline: u64,
    caller: Address,
) -> bool {
    caller.require_auth();

    let mut milestone = get_milestone(env, milestone_id).expect("milestone not found");
    let mut project = get_project(env, milestone.project_id).expect("project not found");

    if !has_permission(env, project.guild_id, caller, Role::Admin) {
        panic!("caller must be guild admin");
    }

    let now = env.ledger().timestamp();
    if new_deadline <= now || new_deadline <= milestone.deadline {
        panic!("new deadline must be in the future and after current deadline");
    }

    milestone.deadline = new_deadline;
    milestone.last_updated_at = now;
    if milestone.status == MilestoneStatus::Expired {
        milestone.status = MilestoneStatus::Pending;
    }
    store_milestone(env, &milestone);

    // project status unchanged; event could be added if needed
    let _ = &mut project; // silence unused for now

    true
}

pub fn cancel_project(env: &Env, project_id: u64, caller: Address) -> bool {
    caller.require_auth();

    let mut project = get_project(env, project_id).expect("project not found");

    if !has_permission(env, project.guild_id, caller, Role::Admin) {
        panic!("caller must be guild admin");
    }

    if project.status == ProjectStatus::Cancelled {
        return true;
    }

    let old_status = project.status.clone();
    project.status = ProjectStatus::Cancelled;
    store_project(env, &project);

    let event = ProjectStatusChangedEvent {
        project_id: project.id,
        old_status,
        new_status: ProjectStatus::Cancelled,
    };
    env.events().publish(("ProjectStatusChanged",), event);

    true
}
