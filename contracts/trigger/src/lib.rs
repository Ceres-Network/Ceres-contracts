#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct TriggerEvent {
    pub policy_id: u64,
    pub triggered_at: u64,
    pub rainfall_value: u32,
    pub ndvi_value: u32,
    pub payout_amount: i128,
    pub trigger_reason: String,
}

#[contracttype]
pub enum DataKey {
    Config,
    Triggered(u64), // policy_id -> TriggerEvent
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub policy_contract: Address,
    pub oracle_contract: Address,
    pub pool_contract: Address,
}

#[contracttype]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    PolicyNotFound = 3,
    PolicyNotActive = 4,
    AlreadyTriggered = 5,
    SeasonEnded = 6,
    OracleDataUnavailable = 7,
    ThresholdNotMet = 8,
    PayoutFailed = 9,
}

#[contract]
pub struct TriggerContract;

#[contractimpl]
impl TriggerContract {
    /// Initialize the trigger contract
    pub fn initialize(
        env: Env,
        admin: Address,
        policy_contract: Address,
        oracle_contract: Address,
        pool_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin,
            policy_contract,
            oracle_contract,
            pool_contract,
        };

        env.storage().instance().set(&DataKey::Config, &config);

        Ok(())
    }

    /// Evaluate a policy against oracle data and trigger payout if conditions are met
    pub fn evaluate_policy(env: Env, policy_id: u64) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Check if already triggered
        if env
            .storage()
            .persistent()
            .has(&DataKey::Triggered(policy_id))
        {
            return Err(Error::AlreadyTriggered);
        }

        // Note: Cross-contract calls will be implemented after initial build
        // For initial compilation, this is a placeholder
        let current_time = env.ledger().timestamp();
        
        // Placeholder trigger event for compilation
        let trigger_event = TriggerEvent {
            policy_id,
            triggered_at: current_time,
            rainfall_value: 0,
            ndvi_value: 0,
            payout_amount: 0,
            trigger_reason: String::from_str(&env, "placeholder"),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Triggered(policy_id), &trigger_event);

        env.events().publish(
            (Symbol::new(&env, "payout_triggered"),),
            policy_id,
        );

        Ok(())
    }

    /// Get trigger event details for a policy
    pub fn get_trigger_event(env: Env, policy_id: u64) -> Result<TriggerEvent, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Triggered(policy_id))
            .ok_or(Error::PolicyNotFound)
    }

    /// Check if a policy has been triggered
    pub fn is_triggered(env: Env, policy_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Triggered(policy_id))
    }
}

// Note: Cross-contract imports will be added after initial build
// For initial compilation, cross-contract calls are commented out
