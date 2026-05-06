#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{create_token_contract, setup_env_with_time};

#[test]
fn test_full_happy_path() {
    let env = setup_env_with_time(1500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    // Create token and mint to provider
    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    // 1. Setup pool
    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);

    // Provider deposits capital
    let shares = pool_client.deposit(&provider, &100_000).unwrap();
    assert_eq!(shares, 100_000);

    // 2. Setup oracle
    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();

    // 3. Setup policy contract
    let policy_contract_id = env.register_contract(None, ceres_policy::PolicyContract);
    let policy_client = ceres_policy::Client::new(&env, &policy_contract_id);
    policy_client
        .initialize(&admin, &pool_contract_id)
        .unwrap();

    // 4. Farmer registers policy
    let geo_cell = String::from_str(&env, "9q5ct");
    let policy_id = policy_client
        .register_policy(
            &farmer,
            &geo_cell,
            &String::from_str(&env, "maize"),
            &1000000,
            &2000000,
            &10_000,
            &200,
            &7000,
        )
        .unwrap();

    assert_eq!(policy_id, 1);

    // Verify pool locked coverage
    let stats = pool_client.get_pool_stats().unwrap();
    assert_eq!(stats.locked_amount, 10_000);

    // 5. Oracle nodes submit readings showing drought
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &150, // Below 200mm threshold
            &1400000,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::NDVI,
            &7500,
            &1400000,
            &signature,
        )
        .unwrap();

    // 6. Setup trigger contract
    let trigger_contract_id = env.register_contract(None, ceres_trigger::TriggerContract);
    let trigger_client = ceres_trigger::Client::new(&env, &trigger_contract_id);
    trigger_client
        .initialize(&admin, &policy_contract_id, &oracle_contract_id, &pool_contract_id)
        .unwrap();

    // 7. Evaluate policy and trigger payout
    trigger_client.evaluate_policy(&policy_id).unwrap();

    // 8. Verify payout received
    let farmer_balance = token_client.balance(&farmer);
    assert_eq!(farmer_balance, 10_000);

    // Verify policy state updated
    let policy = policy_client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.state, ceres_policy::PolicyState::Triggered);

    // Verify pool state updated
    let final_stats = pool_client.get_pool_stats().unwrap();
    assert_eq!(final_stats.total_capital, 90_000);
    assert_eq!(final_stats.locked_amount, 0);

    // Verify trigger event recorded
    let trigger_event = trigger_client.get_trigger_event(&policy_id).unwrap();
    assert_eq!(trigger_event.payout_amount, 10_000);
    assert_eq!(trigger_event.rainfall_value, 150);
}

#[test]
fn test_expired_policy_no_payout() {
    let env = setup_env_with_time(2500000); // After season end
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000).unwrap();

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();

    let policy_contract_id = env.register_contract(None, ceres_policy::PolicyContract);
    let policy_client = ceres_policy::Client::new(&env, &policy_contract_id);
    policy_client
        .initialize(&admin, &pool_contract_id)
        .unwrap();

    let geo_cell = String::from_str(&env, "9q5ct");
    let policy_id = policy_client
        .register_policy(
            &farmer,
            &geo_cell,
            &String::from_str(&env, "maize"),
            &1000000,
            &2000000,
            &10_000,
            &200,
            &7000,
        )
        .unwrap();

    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &150,
            &1400000,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::NDVI,
            &7500,
            &1400000,
            &signature,
        )
        .unwrap();

    let trigger_contract_id = env.register_contract(None, ceres_trigger::TriggerContract);
    let trigger_client = ceres_trigger::Client::new(&env, &trigger_contract_id);
    trigger_client
        .initialize(&admin, &policy_contract_id, &oracle_contract_id, &pool_contract_id)
        .unwrap();

    // Try to evaluate after season end
    let result = trigger_client.evaluate_policy(&policy_id);
    assert!(result.is_err()); // Should fail - season ended
}
