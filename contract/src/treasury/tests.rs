#[cfg(test)]
mod tests {
    use crate::guild::types::Role;
    use crate::treasury::types::{TransactionStatus, TransactionType};
    use crate::StellarGuildsContract;
    use crate::StellarGuildsContractClient;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, Env, String, Vec};

    fn setup_env() -> Env {
        let env = Env::default();
        env.budget().reset_unlimited();
        env
    }

    fn set_ledger_timestamp(env: &Env, timestamp: u64) {
        env.ledger().set(LedgerInfo {
            timestamp,
            protocol_version: 20,
            sequence_number: 0,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 1000000,
        });
    }

    fn register_and_init_contract(env: &Env) -> Address {
        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(env, &contract_id);
        client.initialize();
        contract_id
    }

    fn setup_guild(client: &StellarGuildsContractClient<'_>, env: &Env, owner: &Address) -> u64 {
        let name = String::from_str(env, "Test Guild");
        let description = String::from_str(env, "A test guild");
        client.create_guild(&name, &description, owner)
    }

    fn create_treasury(
        env: &Env,
        client: &StellarGuildsContractClient<'_>,
        guild_id: u64,
    ) -> (u64, Address, Address, Address) {
        let owner = Address::generate(env);
        let signer1 = Address::generate(env);
        let signer2 = Address::generate(env);

        env.mock_all_auths();

        let mut signers = Vec::new(env);
        signers.push_back(owner.clone());
        signers.push_back(signer1.clone());
        signers.push_back(signer2.clone());

        let treasury_id = client.initialize_treasury(&guild_id, &signers, &2u32);

        (treasury_id, owner, signer1, signer2)
    }

    #[test]
    fn test_treasury_initialize_and_deposit_accounting() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let guild_id = setup_guild(&client, &env, &owner);
        let (treasury_id, owner, _s1, _s2) = create_treasury(&env, &client, guild_id);

        let depositor = owner.clone();
        let amount: i128 = 500;

        let ok = client.deposit_treasury(&treasury_id, &depositor, &amount, &None);
        assert!(ok);

        let bal = client.get_treasury_balance(&treasury_id, &None);
        assert_eq!(bal, amount);

        let history = client.get_transaction_history(&treasury_id, &10u32);
        assert_eq!(history.len(), 1);
        let tx = history.get(0).unwrap();
        assert_eq!(tx.tx_type, TransactionType::Deposit);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.status, TransactionStatus::Executed);
    }

    #[test]
    fn test_multisig_withdrawal_flow() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let guild_id = setup_guild(&client, &env, &owner);
        let (treasury_id, owner, signer1, signer2) = create_treasury(&env, &client, guild_id);

        // deposit some XLM accounting
        let amount: i128 = 2000;
        client.deposit_treasury(&treasury_id, &owner, &amount, &None);

        let recipient = Address::generate(&env);

        // create withdrawal proposal
        let reason = String::from_str(&env, "payout");
        let tx_id = client.propose_withdrawal(
            &treasury_id,
            &signer1,
            &recipient,
            &1500i128,
            &None,
            &reason,
        );

        // second signer approves
        client.approve_transaction(&tx_id, &signer2);

        // executor (owner) executes
        client.execute_transaction(&tx_id, &owner);

        let bal = client.get_treasury_balance(&treasury_id, &None);
        assert_eq!(bal, 500);

        let history = client.get_transaction_history(&treasury_id, &10u32);
        assert_eq!(history.len(), 3); // 1 deposit + 1 withdraw proposal + executed state stored
    }

    #[test]
    #[should_panic(expected = "budget exceeded")]
    fn test_budget_enforcement() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let guild_id = setup_guild(&client, &env, &owner);
        let (treasury_id, owner, signer1, signer2) = create_treasury(&env, &client, guild_id);

        client.deposit_treasury(&treasury_id, &owner, &5000i128, &None);

        // set a small budget for withdrawals
        let category = String::from_str(&env, "withdrawal");
        client.set_budget(&treasury_id, &category, &1000i128, &3600u64, &owner);

        let recipient = Address::generate(&env);

        // first withdrawal within budget
        let tx1 = client.propose_withdrawal(
            &treasury_id,
            &signer1,
            &recipient,
            &800i128,
            &None,
            &String::from_str(&env, "first"),
        );
        client.approve_transaction(&tx1, &signer2);
        client.execute_transaction(&tx1, &owner);

        // second withdrawal exceeding remaining budget should panic
        let tx2 = client.propose_withdrawal(
            &treasury_id,
            &signer1,
            &recipient,
            &500i128,
            &None,
            &String::from_str(&env, "second"),
        );

        client.approve_transaction(&tx2, &signer2);

        client.execute_transaction(&tx2, &owner);
    }

    #[test]
    #[should_panic(expected = "treasury is paused")]
    fn test_emergency_pause_blocks_new_ops() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let guild_id = setup_guild(&client, &env, &owner);
        let (treasury_id, owner, signer1, _signer2) = create_treasury(&env, &client, guild_id);

        client.deposit_treasury(&treasury_id, &owner, &1000i128, &None);

        // pause
        client.emergency_pause(&treasury_id, &signer1, &true);

        let recipient = Address::generate(&env);

        let reason = String::from_str(&env, "after pause");
        client.propose_withdrawal(&treasury_id, &signer1, &recipient, &100i128, &None, &reason);
    }
}
