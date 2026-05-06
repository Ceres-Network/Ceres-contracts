#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, crypto::Hash, Address, BytesN, Env, String, Symbol, Vec,
};

#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum ReadingType {
    Rainfall,
    NDVI,
    SoilMoisture,
}

#[derive(Clone)]
#[contracttype]
pub struct Reading {
    pub oracle_node: Address,
    pub geo_cell: String,
    pub reading_type: ReadingType,
    pub value: u32,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AggregatedReading {
    pub geo_cell: String,
    pub reading_type: ReadingType,
    pub value: u32, // Median value
    pub last_updated: u64,
    pub sample_count: u32,
}

#[contracttype]
pub enum DataKey {
    Config,
    OracleWhitelist(Address),
    Reading(String, ReadingType, u64), // geo_cell, type, timestamp
    Aggregated(String, ReadingType),   // geo_cell, type
    ReadingHistory(String, ReadingType), // List of timestamps
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub max_reading_age: u64, // seconds
}

#[contracttype]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    OracleNotWhitelisted = 4,
    ReadingTooOld = 5,
    InvalidSignature = 6,
    NoReadingsAvailable = 7,
}

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle contract
    pub fn initialize(env: Env, admin: Address, max_reading_age: u64) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin: admin.clone(),
            max_reading_age,
        };

        env.storage().instance().set(&DataKey::Config, &config);

        // Add admin to whitelist by default
        env.storage()
            .persistent()
            .set(&DataKey::OracleWhitelist(admin.clone()), &true);

        Ok(())
    }

    /// Add an oracle node to the whitelist (admin only)
    pub fn add_oracle_node(env: Env, admin: Address, oracle_node: Address) -> Result<(), Error> {
        admin.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if config.admin != admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::OracleWhitelist(oracle_node.clone()), &true);

        env.events()
            .publish((Symbol::new(&env, "oracle_added"),), oracle_node);

        Ok(())
    }

    /// Remove an oracle node from the whitelist (admin only)
    pub fn remove_oracle_node(
        env: Env,
        admin: Address,
        oracle_node: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if config.admin != admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::OracleWhitelist(oracle_node.clone()));

        env.events()
            .publish((Symbol::new(&env, "oracle_removed"),), oracle_node);

        Ok(())
    }

    /// Submit a signed reading from an oracle node
    pub fn submit_reading(
        env: Env,
        oracle_node: Address,
        geo_cell: String,
        reading_type: ReadingType,
        value: u32,
        timestamp: u64,
        signature: BytesN<64>,
    ) -> Result<(), Error> {
        oracle_node.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Check if oracle is whitelisted
        if !env
            .storage()
            .persistent()
            .has(&DataKey::OracleWhitelist(oracle_node.clone()))
        {
            return Err(Error::OracleNotWhitelisted);
        }

        // Check reading age
        let current_time = env.ledger().timestamp();
        if current_time > timestamp && (current_time - timestamp) > config.max_reading_age {
            return Err(Error::ReadingTooOld);
        }

        // Store reading
        let reading = Reading {
            oracle_node: oracle_node.clone(),
            geo_cell: geo_cell.clone(),
            reading_type,
            value,
            timestamp,
        };

        let reading_key = DataKey::Reading(geo_cell.clone(), reading_type, timestamp);
        env.storage().persistent().set(&reading_key, &reading);

        // Add to history
        let history_key = DataKey::ReadingHistory(geo_cell.clone(), reading_type);
        let mut history: Vec<u64> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or(Vec::new(&env));
        history.push_back(timestamp);
        env.storage().persistent().set(&history_key, &history);

        env.events().publish(
            (Symbol::new(&env, "reading_submitted"), oracle_node),
            (geo_cell, reading_type, value, timestamp),
        );

        Ok(())
    }

    /// Aggregate readings for a geo cell and reading type (median of last N readings)
    pub fn aggregate_readings(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
        season_window: u64,
    ) -> Result<(), Error> {
        let history_key = DataKey::ReadingHistory(geo_cell.clone(), reading_type);
        let history: Vec<u64> = env
            .storage()
            .persistent()
            .get(&history_key)
            .ok_or(Error::NoReadingsAvailable)?;

        if history.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        let current_time = env.ledger().timestamp();
        let mut values: Vec<u32> = Vec::new(&env);

        // Collect readings within the season window
        for i in 0..history.len() {
            let timestamp = history.get(i).ok_or(Error::NoReadingsAvailable)?;
            if current_time >= timestamp && (current_time - timestamp) <= season_window {
                let reading_key = DataKey::Reading(geo_cell.clone(), reading_type, timestamp);
                if let Some(reading) = env.storage().persistent().get::<_, Reading>(&reading_key)
                {
                    values.push_back(reading.value);
                }
            }
        }

        if values.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        // Calculate median
        let median = Self::calculate_median(&env, &values);

        let aggregated = AggregatedReading {
            geo_cell: geo_cell.clone(),
            reading_type,
            value: median,
            last_updated: current_time,
            sample_count: values.len(),
        };

        let agg_key = DataKey::Aggregated(geo_cell.clone(), reading_type);
        env.storage().persistent().set(&agg_key, &aggregated);

        env.events().publish(
            (Symbol::new(&env, "readings_aggregated"),),
            (geo_cell, reading_type, median, values.len()),
        );

        Ok(())
    }

    /// Get aggregated reading for a geo cell and reading type
    pub fn get_aggregated(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<AggregatedReading, Error> {
        let agg_key = DataKey::Aggregated(geo_cell, reading_type);
        env.storage()
            .persistent()
            .get(&agg_key)
            .ok_or(Error::NoReadingsAvailable)
    }

    /// Check if an oracle node is whitelisted
    pub fn is_whitelisted(env: Env, oracle_node: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::OracleWhitelist(oracle_node))
    }

    // Helper function to calculate median
    fn calculate_median(env: &Env, values: &Vec<u32>) -> u32 {
        if values.is_empty() {
            return 0;
        }

        // Simple bubble sort for small datasets
        let mut sorted = values.clone();
        let len = sorted.len();

        for i in 0..len {
            for j in 0..(len - i - 1) {
                let a = sorted.get(j).unwrap_or(0);
                let b = sorted.get(j + 1).unwrap_or(0);
                if a > b {
                    sorted.set(j, b);
                    sorted.set(j + 1, a);
                }
            }
        }

        let mid = len / 2;
        if len % 2 == 0 {
            let a = sorted.get(mid - 1).unwrap_or(0);
            let b = sorted.get(mid).unwrap_or(0);
            (a + b) / 2
        } else {
            sorted.get(mid).unwrap_or(0)
        }
    }
}
