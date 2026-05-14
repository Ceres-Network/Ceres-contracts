#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracterror, contracttype, Address, Env, String,
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
pub struct LatestReading {
    pub geo_cell: String,
    pub reading_type: ReadingType,
    pub value: u32,
    pub timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    Config,
    LatestReading(String, ReadingType), // geo_cell, type -> latest value
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NoReadingsAvailable = 3,
}

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config { admin };
        env.storage().instance().set(&DataKey::Config, &config);

        Ok(())
    }

    /// Submit a reading
    pub fn submit_reading(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
        value: u32,
    ) -> Result<(), Error> {
        let _config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let current_time = env.ledger().timestamp();

        // Store the latest reading
        let reading = LatestReading {
            geo_cell: geo_cell.clone(),
            reading_type,
            value,
            timestamp: current_time,
        };

        let key = DataKey::LatestReading(geo_cell, reading_type);
        env.storage().persistent().set(&key, &reading);

        Ok(())
    }

    /// Get latest reading for a geo cell and reading type
    pub fn get_latest(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<LatestReading, Error> {
        let key = DataKey::LatestReading(geo_cell, reading_type);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NoReadingsAvailable)
    }
}
