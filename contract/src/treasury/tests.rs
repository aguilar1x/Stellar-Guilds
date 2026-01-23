#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec as SorobanVec};

    use crate::treasury::management::{
        approve_transaction,
        deposit,
        emergency_pause,
        execute_transaction,
        get_balance,
        get_transaction_history,
        grant_allowance,
        initialize_treasury,
        propose_withdrawal,
        set_budget,
    };
    use crate::treasury::storage::initialize_treasury_storage;
    use crate::treasury::types::{TransactionStatus, TransactionType};

    fn setup_env() -> Env {
        let env = Env::default();
        env.budget().reset_unlimited();
        initialize_treasury_storage(&env);
        env
    }

    fn create_treasury(env: &Env) -> (u64, Address, Address, Address) {
        let owner = Address::random(env);
        let signer1 = Address::random(env);
        let signer2 = Address::random(env);

        owner.mock_all_auths();
        signer1.mock_all_auths();
        signer2.mock_all_auths();

        let mut signers = SorobanVec::new(env);
        signers.push_back(owner.clone());
        signers.push_back(signer1.clone());
        signers.push_back(signer2.clone());

        let treasury_id = initialize_treasury(env, 1u64, owner.clone(), signers, 2, 1000);

        (treasury_id, owner, signer1, signer2)
    }

    #[test]
    fn test_treasury_initialize_and_deposit_accounting() {
        let env = setup_env();
        let (treasury_id, owner, _s1, _s2) = create_treasury(&env);

        let depositor = owner.clone();
        let amount: i128 = 500;

        let ok = deposit(&env, treasury_id, depositor.clone(), amount, None);
        assert!(ok);

        let bal = get_balance(&env, treasury_id, None);
        assert_eq!(bal, amount);

        let history = get_transaction_history(&env, treasury_id, 10);
        assert_eq!(history.len(), 1);
        let tx = history.get(0).unwrap();
        assert_eq!(tx.tx_type, TransactionType::Deposit);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.status, TransactionStatus::Executed);
    }

    #[test]
    fn test_multisig_withdrawal_flow() {
        let env = setup_env();
        let (treasury_id, owner, signer1, signer2) = create_treasury(&env);

        // deposit some XLM accounting
        let amount: i128 = 2000;
        deposit(&env, treasury_id, owner.clone(), amount, None);

        let recipient = Address::random(&env);
        recipient.mock_all_auths();

        // create signers vec to pass to proposal
        let reason = String::from_str(&env, "payout");
        let tx_id = propose_withdrawal(
            &env,
            treasury_id,
            signer1.clone(),
            recipient.clone(),
            1500,
            None,
            reason,
        );

        // second signer approves
        approve_transaction(&env, tx_id, signer2.clone());

        // executor (owner) executes
        execute_transaction(&env, tx_id, owner.clone());

        let bal = get_balance(&env, treasury_id, None);
        assert_eq!(bal, 500);

        let history = get_transaction_history(&env, treasury_id, 10);
        assert_eq!(history.len(), 3); // 1 deposit + 1 withdraw proposal + executed state stored
    }

    #[test]
    fn test_budget_enforcement() {
        let env = setup_env();
        let (treasury_id, owner, signer1, signer2) = create_treasury(&env);

        deposit(&env, treasury_id, owner.clone(), 5000, None);

        // set a small budget for withdrawals
        let category = String::from_str(&env, "withdrawal");
        set_budget(&env, treasury_id, owner.clone(), category.clone(), 1000, 3600);

        let recipient = Address::random(&env);
        recipient.mock_all_auths();

        // first withdrawal within budget
        let tx1 = propose_withdrawal(
            &env,
            treasury_id,
            signer1.clone(),
            recipient.clone(),
            800,
            None,
            String::from_str(&env, "first"),
        );
        approve_transaction(&env, tx1, signer2.clone());
        execute_transaction(&env, tx1, owner.clone());

        // second withdrawal exceeding remaining budget should panic
        let tx2 = propose_withdrawal(
            &env,
            treasury_id,
            signer1.clone(),
            recipient.clone(),
            500,
            None,
            String::from_str(&env, "second"),
        );

        signer2.mock_all_auths();

        approve_transaction(&env, tx2, signer2.clone());

        owner.mock_all_auths();

        let result = std::panic::catch_unwind(|| {
            execute_transaction(&env, tx2, owner.clone());
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_emergency_pause_blocks_new_ops() {
        let env = setup_env();
        let (treasury_id, owner, signer1, _signer2) = create_treasury(&env);

        deposit(&env, treasury_id, owner.clone(), 1000, None);

        // pause
        emergency_pause(&env, treasury_id, signer1.clone(), true);

        let recipient = Address::random(&env);
        recipient.mock_all_auths();

        let res = std::panic::catch_unwind(|| {
            let reason = String::from_str(&env, "after pause");
            propose_withdrawal(&env, treasury_id, signer1.clone(), recipient, 100, None, reason);
        });
        assert!(res.is_err());
    }
}
