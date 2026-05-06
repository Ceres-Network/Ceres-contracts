#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{create_token_contract, setup_env_with_time};

#[test]
fn test_policy_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    // Setup pool
    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000).unwrap();

    // Setup policy contract
    let policy_contract_id = env.register_contract(None, ceres_policy::PolicyContract);
    let policy_client = ceres_policy::Client::new(&env, &policy_contract_id);
    policy_client
        .initialize(&admin, &pool_contract_id)
        .unwrap();

    // Register policy
    let policy_id = policy_client
        .register_policy(
            &farmer,
            &String::from_str(&env, "9q5ct"),
            &String::from_str(&env, "maize"),
            &1000000,
            &2000000,
            &10_000,
            &200,
            &7000,
        )
        .unwrap();

    assert_eq!(policy_id, 1);

    let policy = policy_client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.farmer, farmer);
    assert_eq!(policy.coverage_amount, 10_000);
    assert_eq!(policy.state, ceres_policy::PolicyState::Active);
}

#[test]
fn test_policy_list_by_farmer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000).unwrap();

    let policy_contract_id = env.register_contract(None, ceres_policy::PolicyContract);
    let policy_client = ceres_policy::Client::new(&env, &policy_contract_id);
    policy_client
        .initialize(&admin, &pool_contract_id)
        .unwrap();

    // Register multiple policies
    policy_client
        .register_policy(
            &farmer,
            &String::from_str(&env, "9q5ct"),
            &String::from_str(&env, "maize"),
            &1000000,
            &2000000,
            &5_000,
            &200,
            &7000,
        )
        .unwrap();

    policy_client
        .register_policy(
            &farmer,
            &String::from_str(&env, "9q5cu"),
            &String::from_str(&env, "wheat"),
            &1000000,
            &2000000,
            &5_000,
            &180,
            &6500,
        )
        .unwrap();

    let policies = policy_client.list_policies_by_farmer(&farmer);
    assert_eq!(policies.len(), 2);
}

#[test]
fn test_policy_expiration() {
    let env = setup_env_with_time(2500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, ceres_pool::PoolContract);
    let pool_client = ceres_pool::Client::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000).unwrap();

    let policy_contract_id = env.register_contract(None, ceres_policy::PolicyContract);
    let policy_client = ceres_policy::Client::new(&env, &policy_contract_id);
    policy_client
        .initialize(&admin, &pool_contract_id)
        .unwrap();

    let policy_id = policy_client
        .register_policy(
            &farmer,
            &String::from_str(&env, "9q5ct"),
            &String::from_str(&env, "maize"),
            &1000000,
            &2000000,
            &10_000,
            &200,
            &7000,
        )
        .unwrap();

    // Expire policy after season end
    policy_client.expire_policy(&policy_id).unwrap();

    let policy = policy_client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.state, ceres_policy::PolicyState::Expired);
}
