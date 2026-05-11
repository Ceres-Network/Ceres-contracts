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
  coverageAmount: BigInt(5_000_0000000), // 5,000 USDC
  rainfallThreshold: 200, // 200mm
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

### Current Implementation

- Oracle nodes can submit readings without authentication
- Only the **latest reading** is stored (no history)
- No median aggregation (just returns most recent value)
- No signature verification
- No whitelist enforcement

### Intended Design

The full implementation will include:
- **Data Sources**: Weather station APIs, satellite data (CHIRPS, GPM, Sentinel-2)
- **Whitelist**: Only approved oracle nodes can submit
- **Signatures**: Cryptographic verification of readings
- **Aggregation**: Median of multiple readings to prevent manipulation
- **History**: Time-series data for seasonal analysis

## 💰 How Payouts Are Triggered

### Current Implementation

- Accepts simulated rainfall and threshold values
- Stores trigger events but **doesn't release actual payouts**
- No cross-contract calls to policy or pool contracts
- No oracle data integration

### Intended Design

The full implementation will:
1. Fetch policy details from policy contract
2. Get aggregated oracle data for farm location
3. Evaluate drought/crop stress conditions
4. Calculate payout amount (0%, 50%, or 100%)
5. Call pool contract to release funds to farmer
6. Update policy state to Triggered

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
- Storage keys use typed enums
- Collateral ratio enforcement prevents pool insolvency
- Oracle data validation

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

