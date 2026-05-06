#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::setup_env_with_time;

#[test]
fn test_oracle_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800); // 48 hours

    assert!(oracle_client.is_whitelisted(&admin));
}

#[test]
fn test_oracle_whitelist_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);

    // Add oracle node
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();
    assert!(oracle_client.is_whitelisted(&oracle_node));

    // Remove oracle node
    oracle_client
        .remove_oracle_node(&admin, &oracle_node)
        .unwrap();
    assert!(!oracle_client.is_whitelisted(&oracle_node));
}

#[test]
fn test_oracle_submit_reading() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    oracle_client
        .submit_reading(
            &oracle_node,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &250,
            &1000000,
            &signature,
        )
        .unwrap();
}

#[test]
fn test_oracle_aggregation_median() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    let oracle3 = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle1).unwrap();
    oracle_client.add_oracle_node(&admin, &oracle2).unwrap();
    oracle_client.add_oracle_node(&admin, &oracle3).unwrap();

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Submit three readings: 100, 250, 300
    oracle_client
        .submit_reading(
            &oracle1,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &100,
            &999000,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle2,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &250,
            &999500,
            &signature,
        )
        .unwrap();

    oracle_client
        .submit_reading(
            &oracle3,
            &geo_cell,
            &ceres_oracle::ReadingType::Rainfall,
            &300,
            &1000000,
            &signature,
        )
        .unwrap();

    // Aggregate
    oracle_client
        .aggregate_readings(&geo_cell, &ceres_oracle::ReadingType::Rainfall, &10000)
        .unwrap();

    let aggregated = oracle_client
        .get_aggregated(&geo_cell, &ceres_oracle::ReadingType::Rainfall)
        .unwrap();

    assert_eq!(aggregated.value, 250); // Median of [100, 250, 300]
    assert_eq!(aggregated.sample_count, 3);
}

#[test]
fn test_oracle_rejects_old_readings() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, ceres_oracle::OracleContract);
    let oracle_client = ceres_oracle::Client::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800); // 48 hours max age
    oracle_client.add_oracle_node(&admin, &oracle_node).unwrap();

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to submit reading older than 48 hours
    let result = oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &ceres_oracle::ReadingType::Rainfall,
        &250,
        &800000, // Too old
        &signature,
    );

    assert!(result.is_err());
}
