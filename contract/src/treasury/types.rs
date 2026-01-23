use soroban_sdk::{contracttype, Address, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    BountyFunding,
    MilestonePayment,
    AllowanceGrant,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub id: u64,
    pub treasury_id: u64,
    pub tx_type: TransactionType,
    pub amount: i128,
    pub token: Option<Address>,
    pub recipient: Option<Address>,
    pub proposer: Address,
    pub approvals: Vec<Address>,
    pub status: TransactionStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Treasury {
    pub id: u64,
    pub guild_id: u64,
    pub owner: Address,
    pub signers: Vec<Address>,
    pub approval_threshold: u32,
    pub high_value_threshold: i128,
    pub balance_xlm: i128,
    pub token_balances: Map<Address, i128>,
    pub total_deposits: i128,
    pub total_withdrawals: i128,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    pub treasury_id: u64,
    pub category: String,
    pub allocated_amount: i128,
    pub spent_amount: i128,
    pub period_seconds: u64,
    pub period_start: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allowance {
    pub treasury_id: u64,
    pub admin: Address,
    pub token: Option<Address>,
    pub amount_per_period: i128,
    pub remaining_amount: i128,
    pub period_seconds: u64,
    pub period_start: u64,
}

// Events

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryInitializedEvent {
    pub treasury_id: u64,
    pub guild_id: u64,
    pub owner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositEvent {
    pub treasury_id: u64,
    pub from: Address,
    pub amount: i128,
    pub token: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithdrawalProposedEvent {
    pub treasury_id: u64,
    pub tx_id: u64,
    pub proposer: Address,
    pub recipient: Address,
    pub amount: i128,
    pub token: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionApprovedEvent {
    pub treasury_id: u64,
    pub tx_id: u64,
    pub approver: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionExecutedEvent {
    pub treasury_id: u64,
    pub tx_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetUpdatedEvent {
    pub treasury_id: u64,
    pub category: String,
    pub allocated_amount: i128,
    pub period_seconds: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceGrantedEvent {
    pub treasury_id: u64,
    pub admin: Address,
    pub token: Option<Address>,
    pub amount_per_period: i128,
    pub period_seconds: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyPauseEvent {
    pub treasury_id: u64,
    pub paused: bool,
}

impl Treasury {
    pub fn is_signer(&self, addr: &Address) -> bool {
        self.signers.iter().any(|a| a == addr)
    }
}

impl Allowance {
    pub fn ensure_period_current(&mut self, env: &Env) {
        let now = env.ledger().timestamp();
        if now >= self.period_start.saturating_add(self.period_seconds) {
            self.period_start = now;
            self.remaining_amount = self.amount_per_period;
        }
    }
}
