#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{create_token_contract, setup_env_with_time};

#[test]
fn test_trigger_drought_payout() {
    let env = setup_env_with_time(1500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    // Setup pool
    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000).unwrap();

    // Setup oracle
    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();

    // Setup policy
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
            &200, // Rainfall threshold
            &7000,
        )
        .unwrap();

    // Submit oracle readings showing drought (rainfall < 200mm)
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &150, // Below threshold
            &1400000,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::NDVI,
            &7500, // Above stress level
            &1400000,
            &signature,
        )
        .unwrap();

    // Setup trigger
    let trigger_contract_id = env.register_contract(None, ceres_trigger::TriggerContract);
    let trigger_client = ceres_trigger::Client::new(&env, &trigger_contract_id);
    trigger_client
        .initialize(&admin, &policy_contract_id, &oracle_contract_id, &pool_contract_id)
        .unwrap();

    // Evaluate policy
    trigger_client.evaluate_policy(&policy_id).unwrap();

    // Check payout was made
    let farmer_balance = token_client.balance(&farmer);
    assert_eq!(farmer_balance, 10_000); // Full payout

    // Check policy state
    let policy = policy_client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.state, ceres_policy::PolicyState::Triggered);

    // Check trigger event
    assert!(trigger_client.is_triggered(&policy_id));
}

#[test]
fn test_trigger_crop_stress_partial_payout() {
    let env = setup_env_with_time(1500000);
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
            &7000, // NDVI baseline
        )
        .unwrap();

    // Submit readings: rainfall OK, but NDVI shows stress
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &250, // Above threshold
            &1400000,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::NDVI,
            &4500, // Below 70% of 7000 (4900)
            &1400000,
            &signature,
        )
        .unwrap();

    let trigger_contract_id = env.register_contract(None, ceres_trigger::TriggerContract);
    let trigger_client = ceres_trigger::Client::new(&env, &trigger_contract_id);
    trigger_client
        .initialize(&admin, &policy_contract_id, &oracle_contract_id, &pool_contract_id)
        .unwrap();

    trigger_client.evaluate_policy(&policy_id).unwrap();

    // Check partial payout (50%)
    let farmer_balance = token_client.balance(&farmer);
    assert_eq!(farmer_balance, 5_000);
}

#[test]
fn test_trigger_double_trigger_prevention() {
    let env = setup_env_with_time(1500000);
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

    // First trigger
    trigger_client.evaluate_policy(&policy_id).unwrap();

    // Second trigger should fail
    let result = trigger_client.evaluate_policy(&policy_id);
    assert!(result.is_err());
}
