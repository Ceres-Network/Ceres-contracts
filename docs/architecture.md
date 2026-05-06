# Ceres Network Architecture

This document provides a deep dive into the architecture and design decisions of Ceres Network.

## Table of Contents

- [System Overview](#system-overview)
- [Contract Architecture](#contract-architecture)
- [Data Flow](#data-flow)
- [Storage Design](#storage-design)
- [Security Model](#security-model)
- [Economic Model](#economic-model)

## System Overview

Ceres Network is a decentralized parametric crop insurance protocol built on Stellar using Soroban smart contracts. The system consists of four core contracts that work together to provide automated insurance coverage.

### Design Principles

1. **Trustless Automation**: No human intervention required for payouts
2. **Transparent Pricing**: All parameters visible on-chain
3. **Capital Efficiency**: Shared liquidity pool serves multiple policies
4. **Oracle Resilience**: Median aggregation prevents manipulation
5. **Composability**: Modular contracts for easy upgrades

## Contract Architecture

### Pool Contract

**Purpose**: Manages liquidity provider capital and coverage allocation

**Key Functions**:
- `initialize()`: Set up pool with admin, stablecoin asset, and collateral ratio
- `deposit()`: LP deposits capital, receives proportional shares
- `withdraw()`: LP burns shares, receives proportional capital
- `lock_coverage()`: Reserve funds for active policy
- `release_payout()`: Transfer payout to farmer

**Storage**:
```rust
Config {
    admin: Address,
    stablecoin_asset: Address,
    min_collateral_ratio: u32, // Basis points (500 = 5:1)
}

PoolStats {
    total_capital: i128,
    locked_amount: i128,
    total_shares: i128,
}

ProviderPosition {
    shares: i128,
}
```

**Collateral Ratio Enforcement**:

The pool enforces a minimum 5:1 collateral ratio:

```
available_capital / locked_amount >= 5
```

This ensures the pool always has sufficient capital to cover locked policies. New policies are rejected if they would breach this ratio.

**Share Calculation**:

First deposit: 1:1 ratio (1 token = 1 share)

Subsequent deposits:
```
shares_to_mint = (deposit_amount * total_shares) / total_capital
```

This ensures fair pricing as the pool grows and earns yield.

### Policy Contract

**Purpose**: Stores policy metadata and lifecycle state

**Key Functions**:
- `initialize()`: Set up policy contract with admin and pool reference
- `register_policy()`: Create new policy, lock coverage in pool
- `get_policy()`: Retrieve policy details
- `list_policies_by_farmer()`: Get all policies for a farmer
- `expire_policy()`: Mark policy as expired after season ends
- `update_policy_state()`: Update state (called by trigger contract)

**Storage**:
```rust
Policy {
    policy_id: u64,
    farmer: Address,
    farm_geohash: String,      // Geohash precision 5 (~5km cell)
    crop_type: String,
    season_start: u64,
    season_end: u64,
    coverage_amount: i128,
    rainfall_threshold: u32,    // mm
    ndvi_baseline: u32,         // scaled by 10000
    state: PolicyState,
}

PolicyState: Active | Triggered | Expired
```

**Geohash System**:

Policies use [geohash](https://en.wikipedia.org/wiki/Geohash) for location indexing:
- Precision 5 = ~5km x 5km cell
- Example: `9q5ct` = San Francisco area
- Allows efficient spatial queries
- Oracle data indexed by same geohash

### Oracle Contract

**Purpose**: Aggregate weather data from multiple oracle nodes

**Key Functions**:
- `initialize()`: Set up oracle with admin and max reading age
- `add_oracle_node()`: Whitelist oracle node (admin only)
- `remove_oracle_node()`: Remove oracle from whitelist
- `submit_reading()`: Oracle submits signed data point
- `aggregate_readings()`: Calculate median of readings in time window
- `get_aggregated()`: Retrieve aggregated value

**Storage**:
```rust
Reading {
    oracle_node: Address,
    geo_cell: String,
    reading_type: ReadingType,
    value: u32,
    timestamp: u64,
}

ReadingType: Rainfall | NDVI | SoilMoisture

AggregatedReading {
    geo_cell: String,
    reading_type: ReadingType,
    value: u32,              // Median
    last_updated: u64,
    sample_count: u32,
}
```

**Median Aggregation**:

The oracle uses median instead of mean to resist manipulation:

```rust
fn calculate_median(values: Vec<u32>) -> u32 {
    let sorted = sort(values);
    if sorted.len() % 2 == 0 {
        (sorted[len/2 - 1] + sorted[len/2]) / 2
    } else {
        sorted[len/2]
    }
}
```

**Why median?**
- Single malicious oracle cannot skew result
- Requires majority collusion to manipulate
- More robust than mean for outliers

**Reading Validation**:
- Oracle must be whitelisted
- Reading must be < 48 hours old
- Signature must be valid (future enhancement)

### Trigger Contract

**Purpose**: Evaluate policies and trigger payouts

**Key Functions**:
- `initialize()`: Set up trigger with references to other contracts
- `evaluate_policy()`: Check oracle data against policy thresholds
- `get_trigger_event()`: Retrieve trigger details
- `is_triggered()`: Check if policy has been triggered

**Storage**:
```rust
TriggerEvent {
    policy_id: u64,
    triggered_at: u64,
    rainfall_value: u32,
    ndvi_value: u32,
    payout_amount: i128,
    trigger_reason: String,
}
```

**Evaluation Logic**:

```rust
// 1. Get policy details
let policy = policy_contract.get_policy(policy_id);

// 2. Get aggregated oracle data
let rainfall = oracle_contract.get_aggregated(
    policy.farm_geohash,
    ReadingType::Rainfall
);
let ndvi = oracle_contract.get_aggregated(
    policy.farm_geohash,
    ReadingType::NDVI
);

// 3. Evaluate conditions
if rainfall.value < policy.rainfall_threshold {
    payout = policy.coverage_amount;  // 100%
} else if ndvi.value < policy.ndvi_baseline * 0.7 {
    payout = policy.coverage_amount / 2;  // 50%
}

// 4. Release payout if triggered
if payout > 0 {
    pool_contract.release_payout(policy_id, farmer, payout);
    policy_contract.update_policy_state(policy_id, Triggered);
}
```

**Idempotency**:

The trigger contract prevents double-triggers by storing triggered policy IDs. Subsequent calls to `evaluate_policy()` for the same policy will fail.

## Data Flow

### Policy Registration Flow

```
1. Farmer calls policy_contract.register_policy()
   ↓
2. Policy contract validates parameters
   ↓
3. Policy contract calls pool_contract.lock_coverage()
   ↓
4. Pool checks collateral ratio
   ↓
5. Pool locks coverage amount
   ↓
6. Policy stored with Active state
   ↓
7. Policy ID returned to farmer
```

### Oracle Data Submission Flow

```
1. Oracle node fetches weather data
   ↓
2. Oracle signs reading with private key
   ↓
3. Oracle calls oracle_contract.submit_reading()
   ↓
4. Contract validates oracle is whitelisted
   ↓
5. Contract validates reading age < 48 hours
   ↓
6. Reading stored with timestamp
   ↓
7. Reading added to history for geo_cell
```

### Payout Trigger Flow

```
1. Anyone calls trigger_contract.evaluate_policy()
   ↓
2. Trigger fetches policy details
   ↓
3. Trigger calls oracle_contract.aggregate_readings()
   ↓
4. Oracle calculates median of recent readings
   ↓
5. Trigger compares oracle data to thresholds
   ↓
6. If triggered, trigger calls pool_contract.release_payout()
   ↓
7. Pool transfers tokens to farmer
   ↓
8. Trigger updates policy state to Triggered
   ↓
9. TriggerEvent emitted with details
```

## Storage Design

### Storage Types

Soroban provides three storage types:

1. **Instance Storage**: Contract-level data (config)
2. **Persistent Storage**: Long-lived data (policies, readings)
3. **Temporary Storage**: Short-lived data (not used in Ceres)

### Storage Keys

All storage keys use typed enums:

```rust
#[contracttype]
pub enum DataKey {
    Config,
    Policy(u64),
    Provider(Address),
}
```

**Why typed keys?**
- Type safety at compile time
- No string typos
- Better performance
- Clear data structure

### Storage Costs

Soroban charges for storage based on:
- Entry size (bytes)
- Time-to-live (TTL)

**Optimization strategies**:
- Use `u32` instead of `u64` where possible
- Pack related data into structs
- Use geohash strings instead of lat/lon pairs
- Expire old oracle readings

## Security Model

### Access Control

**Pool Contract**:
- Anyone can deposit/withdraw their own funds
- Only trigger contract can release payouts
- Admin can update config (future: governance)

**Policy Contract**:
- Farmers can register their own policies
- Only trigger contract can update policy state
- Anyone can read policy data

**Oracle Contract**:
- Only whitelisted oracles can submit readings
- Only admin can manage whitelist
- Anyone can aggregate and read data

**Trigger Contract**:
- Anyone can evaluate policies (permissionless)
- Only trigger contract can update policy state

### Attack Vectors & Mitigations

**1. Oracle Manipulation**

*Attack*: Malicious oracle submits false data to trigger payouts

*Mitigation*:
- Median aggregation requires majority collusion
- Whitelisted oracles with reputation at stake
- 48-hour reading age limit prevents stale data
- Multiple data sources per oracle node

**2. Pool Insolvency**

*Attack*: Too many policies trigger simultaneously, draining pool

*Mitigation*:
- 5:1 collateral ratio enforced
- New policies rejected if ratio would breach
- Diversification across regions and crops
- LP can withdraw unlocked capital anytime

**3. Front-Running**

*Attack*: Farmer sees oracle data, quickly registers policy before trigger

*Mitigation*:
- Policy must be registered before season starts
- Oracle data aggregated over season window
- Minimum policy duration enforced

**4. Reentrancy**

*Attack*: Malicious contract calls back during payout

*Mitigation*:
- State updated before external calls
- Soroban's execution model prevents reentrancy
- Idempotency checks prevent double-triggers

## Economic Model

### Liquidity Provider Economics

**Revenue Sources**:
- Premiums paid by farmers (future enhancement)
- Yield from stablecoin lending (future enhancement)

**Costs**:
- Payouts to farmers
- Gas fees for transactions

**Risk**:
- Correlated losses (regional disasters)
- Oracle failures
- Smart contract bugs

**Returns**:
- Target: 8-12% APY
- Varies by utilization and loss ratio

### Farmer Economics

**Costs**:
- Premium payment (future: calculated based on risk)
- Gas fees for registration

**Benefits**:
- Automatic payouts during crop failure
- No claims process
- Fast settlement (minutes, not months)
- Transparent pricing

**Example**:
- Coverage: $5,000
- Premium: $250 (5% of coverage)
- Payout: $5,000 if drought
- Break-even: 1 payout every 20 seasons

### Premium Calculation (Future)

Premiums will be calculated based on:

```
premium = coverage_amount * base_rate * risk_multiplier

risk_multiplier = f(
    historical_trigger_rate,
    crop_type,
    region,
    season_length,
    threshold_sensitivity
)
```

**Factors**:
- Historical weather data for region
- Crop vulnerability to weather events
- Policy parameters (tighter thresholds = higher premium)
- Pool utilization (higher utilization = higher premium)

## Scalability Considerations

### Current Limitations

- Oracle data stored indefinitely (storage costs)
- Linear search for policy history
- Single pool for all policies

### Future Optimizations

1. **Oracle Data Pruning**
   - Archive old readings off-chain
   - Keep only recent season data on-chain

2. **Policy Indexing**
   - Spatial index by geohash
   - Temporal index by season
   - Crop type index

3. **Multi-Pool Architecture**
   - Separate pools by region
   - Separate pools by crop type
   - Reduces correlated risk

4. **Batch Operations**
   - Batch policy evaluations
   - Batch oracle submissions
   - Reduces gas costs

## Upgrade Path

### Contract Upgrades

Soroban contracts are immutable by default. Upgrade strategies:

1. **Deploy new version**
   - Deploy new contract
   - Migrate state
   - Update references

2. **Proxy pattern**
   - Proxy contract delegates to implementation
   - Upgrade implementation, keep proxy

3. **Governance**
   - DAO controls upgrades
   - Time-locked upgrades
   - Emergency pause mechanism

### Data Migration

For major upgrades:

1. Snapshot current state
2. Deploy new contracts
3. Migrate critical data (active policies)
4. Archive old contracts (read-only)
5. Update SDK and dApp

## Monitoring & Observability

### Events

All contracts emit events for key actions:

```rust
// Pool events
env.events().publish(("deposit", provider), (amount, shares));
env.events().publish(("withdraw", provider), (shares, amount));
env.events().publish(("payout_released", farmer), (policy_id, amount));

// Policy events
env.events().publish(("policy_registered", farmer), (policy_id, coverage));
env.events().publish(("policy_expired",), policy_id);

// Oracle events
env.events().publish(("reading_submitted", oracle), (geo_cell, value));
env.events().publish(("readings_aggregated",), (geo_cell, median));

// Trigger events
env.events().publish(("payout_triggered", farmer), (policy_id, amount));
```

### Metrics to Track

- Total value locked (TVL)
- Number of active policies
- Pool utilization ratio
- Payout frequency and amounts
- Oracle submission frequency
- Gas costs per operation

### Alerting

Monitor for:
- Collateral ratio approaching minimum
- Oracle nodes missing submissions
- Unusual payout patterns
- Contract errors or reverts

---

This architecture is designed to be secure, efficient, and scalable while maintaining the core principles of decentralization and transparency.
