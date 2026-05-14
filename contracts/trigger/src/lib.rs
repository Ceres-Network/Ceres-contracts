#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[derive(Clone)]
#[contracttype]
pub struct TriggerEvent {
    pub policy_id: u64,
    pub triggered_at: u64,
    pub rainfall_value: u32,
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

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyTriggered = 3,
    ThresholdNotMet = 4,
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

    /// Evaluate a policy
    pub fn evaluate_policy(
        env: Env,
        policy_id: u64,
        simulated_rainfall: u32,
        simulated_threshold: u32,
    ) -> Result<(), Error> {
        let _config: Config = env
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

        let current_time = env.ledger().timestamp();

        // Simple evaluation: if rainfall < threshold, trigger payout
        if simulated_rainfall < simulated_threshold {
            let payout_amount = 50_000_000_000_i128; // Hardcoded 5000 USDC for demo

            let trigger_event = TriggerEvent {
                policy_id,
                triggered_at: current_time,
                rainfall_value: simulated_rainfall,
                payout_amount,
                trigger_reason: String::from_str(&env, "drought_detected"),
            };

            env.storage()
                .persistent()
                .set(&DataKey::Triggered(policy_id), &trigger_event);

            Ok(())
        } else {
            Err(Error::ThresholdNotMet)
        }
    }

    /// Get trigger event details for a policy
    pub fn get_trigger_event(env: Env, policy_id: u64) -> Result<TriggerEvent, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Triggered(policy_id))
            .ok_or(Error::NotInitialized)
    }

    /// Check if a policy has been triggered
    pub fn is_triggered(env: Env, policy_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Triggered(policy_id))
    }
}
