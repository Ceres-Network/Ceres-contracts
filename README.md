# Ceres Network

[![CI](https://github.com/ceres-network/ceres-network/actions/workflows/ci.yml/badge.svg)](https://github.com/ceres-network/ceres-network/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Decentralized parametric crop insurance for smallholder farmers, built on Stellar with Soroban smart contracts.**

Ceres Network enables farmers in emerging markets to register farm plots, pay stablecoin premiums, and receive automatic payouts when on-chain weather oracle data confirms drought, flood, or crop stress conditions — no claims process, no insurance agents.

## 🌾 Overview

Traditional crop insurance is inaccessible to smallholder farmers due to high costs, complex claims processes, and lack of infrastructure. Ceres Network solves this with:

- **Parametric triggers**: Payouts based on objective weather data (rainfall, NDVI, soil moisture)
- **Instant settlement**: Smart contracts automatically release funds when conditions are met
- **Transparent pricing**: On-chain liquidity pool with clear collateral ratios
- **No claims process**: Oracle data directly triggers payouts
- **Stablecoin denominated**: Payments in USDC for price stability

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Ceres Network                            │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────┐      ┌──────────┐      ┌──────────┐          │
│  │  Farmer  │      │    LP    │      │  Oracle  │          │
│  │          │      │ Provider │      │   Node   │          │
│  └────┬─────┘      └────┬─────┘      └────┬─────┘          │
│       │                 │                  │                 │
│       │ register        │ deposit          │ submit          │
│       │ policy          │ capital          │ readings        │
│       ▼                 ▼                  ▼                 │
│  ┌──────────────────────────────────────────────┐           │
│  │           Soroban Smart Contracts             │           │
│  ├──────────────────────────────────────────────┤           │
│  │                                               │           │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │           │
│  │  │  Policy  │  │   Pool   │  │  Oracle  │  │           │
│  │  │ Contract │◄─┤ Contract │  │ Contract │  │           │
│  │  └────┬─────┘  └────▲─────┘  └────▲─────┘  │           │
│  │       │             │              │         │           │
│  │       │             │              │         │           │
│  │       ▼             │              │         │           │
│  │  ┌──────────────────┴──────────────┘         │           │
│  │  │      Trigger Contract                     │           │
│  │  │  (Evaluates conditions & releases payout) │           │
│  │  └───────────────────────────────────────────┘           │
│  │                                               │           │
│  └──────────────────────────────────────────────┘           │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Contract Responsibilities

- **Pool Contract**: Manages liquidity provider deposits, coverage locks, and payout releases
- **Policy Contract**: Stores farmer policies with coverage terms and trigger thresholds
- **Oracle Contract**: Aggregates weather data from whitelisted oracle nodes using median calculation
- **Trigger Contract**: Evaluates policies against oracle data and triggers payouts

## 🚀 Quickstart

### Prerequisites

- Rust 1.81.0 (installed via `rust-toolchain.toml`)
- Stellar CLI ([installation guide](https://developers.stellar.org/docs/tools/developer-tools))
- Node.js 20+ (for TypeScript SDK)

### Build Contracts

```bash
# Clone repository
git clone https://github.com/ceres-network/ceres-network.git
cd ceres-network

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test
```

### Deploy to Stellar Testnet

```bash
# Deploy contracts
./scripts/deploy.sh

# Seed with test data
cd sdk/typescript
npm install
npm run build
cd ../..
node scripts/seed.ts
```

### Use TypeScript SDK

```typescript
import { CeresClient } from '@ceres-network/sdk';
import { Keypair, Networks } from '@stellar/stellar-sdk';

const client = new CeresClient({
  networkPassphrase: Networks.TESTNET,
  rpcUrl: 'https://soroban-testnet.stellar.org',
  poolContractId: 'C...',
  policyContractId: 'C...',
  oracleContractId: 'C...',
  triggerContractId: 'C...',
});

// Register a policy
const farmer = Keypair.random();
const policyId = await client.registerPolicy(farmer, {
  farmer: farmer.publicKey(),
  farmGeohash: '9q5ct',
  cropType: 'maize',
  seasonStart: BigInt(Date.now() / 1000),
  seasonEnd: BigInt(Date.now() / 1000 + 90 * 24 * 60 * 60),
  coverageAmount: BigInt(5_000_0000000), // 5,000 USDC
  rainfallThreshold: 200, // 200mm
  ndviBaseline: 7000, // 0.7
});

// Evaluate policy for payout
await client.evaluatePolicy(farmer, policyId);
```

## 📊 Contract Addresses

### Testnet

| Contract | Address |
|----------|---------|
| Pool     | `C...` (deploy to get address) |
| Policy   | `C...` (deploy to get address) |
| Oracle   | `C...` (deploy to get address) |
| Trigger  | `C...` (deploy to get address) |

### Mainnet

*Not yet deployed*

## 🛰️ How Oracles Work

### Data Sources

Oracle nodes aggregate data from:
- **Rainfall**: Weather station APIs, satellite data (CHIRPS, GPM)
- **NDVI**: Sentinel-2, Landsat satellite imagery
- **Soil Moisture**: SMAP, SMOS satellite missions

### Submission Flow

1. Oracle nodes fetch data for registered farm locations (geohash cells)
2. Nodes sign readings with their private keys
3. Readings submitted to Oracle Contract via `submit_reading()`
4. Contract validates:
   - Oracle is whitelisted
   - Reading is < 48 hours old
   - Signature is valid

### Aggregation

- Contract stores all readings with timestamps
- `aggregate_readings()` calculates **median** of readings within season window
- Median prevents single malicious oracle from manipulating payouts
- Aggregated value used by Trigger Contract for evaluation

## 💰 How Payouts Are Triggered

### Trigger Conditions

Anyone can call `evaluate_policy(policy_id)` to check if a policy should pay out:

1. **Drought (100% payout)**
   - Season rainfall < policy.rainfall_threshold
   - Example: 150mm recorded, 200mm threshold → full payout

2. **Crop Stress (50% payout)**
   - NDVI < 70% of policy.ndvi_baseline
   - Example: NDVI 0.45, baseline 0.70 → partial payout

### Example Thresholds

| Crop | Region | Rainfall Threshold | NDVI Baseline |
|------|--------|-------------------|---------------|
| Maize | East Africa | 200mm/season | 0.70 |
| Wheat | South Asia | 180mm/season | 0.65 |
| Sorghum | West Africa | 220mm/season | 0.68 |

### Payout Process

```
evaluate_policy(policy_id)
  ↓
Check oracle data for farm location
  ↓
Compare against policy thresholds
  ↓
Calculate payout amount (0%, 50%, or 100%)
  ↓
Call pool.release_payout(farmer, amount)
  ↓
Update policy state to Triggered
  ↓
Emit PayoutTriggered event
```

## 🔧 Development

### Run Tests

```bash
# All tests
cargo test

# Specific test module
cargo test pool_tests

# With output
cargo test -- --nocapture
```

### Lint & Format

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

### TypeScript SDK Development

```bash
cd sdk/typescript

# Install dependencies
npm install

# Build
npm run build

# Watch mode
npm run dev

# Type check
npm run typecheck
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Good First Issues

- Add support for flood triggers (excess rainfall)
- Implement premium calculation logic
- Add more oracle data sources
- Improve SDK documentation
- Write integration tests for edge cases

## 📚 Documentation

- [Architecture Deep Dive](docs/architecture.md)
- [Oracle Specification](docs/oracle-spec.md)
- [Farmer Onboarding Guide](docs/farmer-onboarding.md)

## 🔐 Security

- All contracts use typed errors (no raw `u32` codes)
- No `unwrap()` in production paths
- Storage keys use `Symbol` types
- Collateral ratio enforcement prevents pool insolvency
- Median aggregation resists oracle manipulation

**Security audits**: Not yet audited. Use at your own risk.

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🌐 Stellar/Soroban Compatibility

| Component | Version |
|-----------|---------|
| Soroban SDK | 21.7.0 |
| Rust Toolchain | 1.81.0 |
| Stellar SDK (JS) | 12.0.0+ |
| Network | Testnet (Futurenet compatible) |

## 🙏 Acknowledgments

Built with:
- [Stellar](https://stellar.org) - Fast, low-cost blockchain
- [Soroban](https://soroban.stellar.org) - Smart contract platform
- [Geohash](https://en.wikipedia.org/wiki/Geohash) - Spatial indexing system

---

**Ceres Network** - Bringing crop insurance to the uninsured 🌾
