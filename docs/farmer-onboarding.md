# Farmer Onboarding Guide

This guide walks smallholder farmers through the process of registering for parametric crop insurance on Ceres Network.

## Table of Contents

- [What is Parametric Insurance?](#what-is-parametric-insurance)
- [How Ceres Network Works](#how-ceres-network-works)
- [Getting Started](#getting-started)
- [Registering a Policy](#registering-a-policy)
- [Understanding Your Policy](#understanding-your-policy)
- [Receiving Payouts](#receiving-payouts)
- [FAQ](#faq)

## What is Parametric Insurance?

Traditional crop insurance requires:
- Filing claims after crop loss
- Waiting for adjuster to visit farm
- Proving damage with documentation
- Waiting weeks or months for payout

**Parametric insurance is different:**
- No claims process
- Automatic payouts based on weather data
- Fast settlement (minutes, not months)
- Transparent pricing and conditions

### Example

**Traditional Insurance**:
```
Drought occurs → File claim → Wait for adjuster → 
Provide proof → Wait for approval → Receive payout (maybe)
Time: 2-6 months
```

**Parametric Insurance (Ceres)**:
```
Drought occurs → Oracle confirms low rainfall → 
Automatic payout to your wallet
Time: Minutes
```

## How Ceres Network Works

### 1. Register Your Farm

Provide basic information:
- Farm location (approximate)
- Crop type
- Growing season dates
- Coverage amount desired

### 2. Pay Premium

Pay a small premium in stablecoins (USDC):
- Typical cost: 5-10% of coverage
- Example: $250 premium for $5,000 coverage

### 3. Automatic Monitoring

Satellites and weather stations monitor your area:
- Rainfall measurements
- Crop health (NDVI)
- Soil moisture

### 4. Automatic Payout

If conditions trigger your policy:
- **Drought**: Rainfall below threshold → 100% payout
- **Crop Stress**: NDVI below threshold → 50% payout
- Money sent directly to your wallet

## Getting Started

### Step 1: Get a Stellar Wallet

You need a Stellar wallet to receive payouts.

**Recommended Wallets**:
- [Freighter](https://www.freighter.app/) - Browser extension
- [Lobstr](https://lobstr.co/) - Mobile app
- [Solar Wallet](https://solarwallet.io/) - Desktop & mobile

**Setup Instructions**:
1. Download wallet app
2. Create new wallet
3. **IMPORTANT**: Write down your 24-word recovery phrase
4. Store recovery phrase in a safe place
5. Never share your recovery phrase with anyone

### Step 2: Fund Your Wallet

You need:
- **XLM** (Stellar Lumens) for transaction fees (~1 XLM)
- **USDC** for premium payment

**How to Get USDC**:
- Buy from exchange (Coinbase, Binance, Kraken)
- Receive from another wallet
- Use local crypto exchange in your country

**How to Get XLM**:
- Buy from exchange
- Use Stellar testnet faucet (for testing)

### Step 3: Access Ceres dApp

Visit the Ceres Network web application:
- URL: `https://app.ceres.network` (example)
- Connect your wallet
- Grant permission to read your address

## Registering a Policy

### Step 1: Enter Farm Details

**Farm Location**:
- You don't need exact coordinates
- Approximate location is enough (within 5km)
- Your exact farm location is NOT stored on-chain
- Example: "Near Nairobi, Kenya"

**Crop Type**:
- Select from dropdown: Maize, Wheat, Rice, Sorghum, etc.
- Different crops have different thresholds

**Growing Season**:
- Start date: When you plant
- End date: When you harvest
- Example: March 1 - June 30 (90 days)

### Step 2: Choose Coverage

**Coverage Amount**:
- How much money you want to receive if triggered
- Consider: seed cost, fertilizer, labor, lost income
- Example: $5,000 covers typical 2-hectare maize farm

**Premium**:
- Calculated automatically based on risk
- Typical: 5-10% of coverage amount
- Example: $250-$500 premium for $5,000 coverage

### Step 3: Set Trigger Conditions

**Rainfall Threshold**:
- Minimum rainfall needed for your crop
- If season rainfall < threshold → 100% payout
- Recommended thresholds:
  - Maize: 200mm
  - Wheat: 180mm
  - Sorghum: 220mm
  - Rice: 300mm

**NDVI Baseline**:
- Expected crop health level
- If NDVI < 70% of baseline → 50% payout
- Typical baseline: 0.65-0.75 (6500-7500 scaled)

### Step 4: Review and Confirm

**Review Screen Shows**:
- Farm location (geohash)
- Crop type
- Season dates
- Coverage amount
- Premium cost
- Trigger conditions

**Confirm Transaction**:
1. Check all details are correct
2. Click "Register Policy"
3. Approve transaction in wallet
4. Wait for confirmation (~5 seconds)
5. Receive policy ID

**Transaction Costs**:
- Premium payment (e.g., $250)
- Network fee (~0.01 XLM, less than $0.01)

## Understanding Your Policy

### Policy Dashboard

After registration, view your policy:

**Policy Details**:
```
Policy ID: #12345
Status: Active
Farm Location: 9q5ct (San Francisco area)
Crop: Maize
Season: March 1 - June 30, 2024
Coverage: $5,000 USDC
Premium Paid: $250 USDC
```

**Trigger Conditions**:
```
Drought Trigger:
  Rainfall < 200mm → 100% payout ($5,000)

Crop Stress Trigger:
  NDVI < 4900 (70% of 7000) → 50% payout ($2,500)
```

**Current Data**:
```
Season Rainfall: 145mm (updated daily)
Latest NDVI: 7200 (updated weekly)
Days Remaining: 45
```

### Monitoring Your Policy

**Data Updates**:
- Rainfall: Updated daily
- NDVI: Updated weekly (when clouds permit)
- Soil Moisture: Updated every 2-3 days

**Notifications** (if enabled):
- Low rainfall alert
- NDVI drop alert
- Payout triggered
- Season ending soon

### Policy States

**Active**:
- Policy is monitoring conditions
- Can be triggered if thresholds met
- Season is ongoing

**Triggered**:
- Payout has been released
- Policy is complete
- Check wallet for funds

**Expired**:
- Season ended without trigger
- No payout (good news - crops survived!)
- Can register new policy for next season

## Receiving Payouts

### Automatic Trigger

Payouts are triggered automatically when:
1. Oracle data confirms trigger condition
2. Anyone calls "evaluate policy" function
3. Smart contract releases funds
4. USDC sent to your wallet

**No action required from you!**

### Checking for Payout

**In dApp**:
- Policy status changes to "Triggered"
- Payout amount shown
- Transaction hash displayed

**In Wallet**:
- Check USDC balance
- Should see incoming transaction
- Amount matches policy coverage

### Payout Timing

**How Fast?**:
- Evaluation can happen anytime during season
- Once triggered: payout in ~5 seconds
- Funds immediately available in wallet

**Who Triggers?**:
- Anyone can call evaluate function
- Ceres bot checks policies daily
- You can trigger your own policy
- Community members can trigger

### Using Your Payout

**Immediate Access**:
- Funds are in your wallet
- Use for any purpose
- No restrictions

**Suggested Uses**:
- Buy seeds for replanting
- Pay for irrigation
- Cover living expenses
- Save for next season

**Converting to Local Currency**:
- Use local crypto exchange
- Peer-to-peer trading
- Mobile money integration (coming soon)

## FAQ

### General Questions

**Q: Do I need to know about blockchain or crypto?**

A: No! The dApp guides you through everything. You just need a wallet (like a bank account) and some USDC (like digital dollars).

**Q: Is my farm location private?**

A: Yes! Only an approximate location (within 5km) is stored. Your exact farm coordinates are never recorded.

**Q: What if I don't have a smartphone?**

A: You can register using any device with internet. After registration, you don't need to check daily - payouts are automatic.

**Q: Can I register multiple farms?**

A: Yes! Register a separate policy for each farm or field.

### Coverage Questions

**Q: How much coverage should I get?**

A: Consider:
- Input costs (seeds, fertilizer, labor)
- Expected harvest value
- Living expenses during season
- Typical: 50-80% of expected harvest value

**Q: Can I cancel my policy?**

A: No, policies cannot be canceled once registered. Premium is non-refundable.

**Q: What if my crops fail but conditions don't trigger?**

A: Parametric insurance only pays for specific conditions (drought, crop stress). It doesn't cover all risks (pests, disease, theft).

### Payout Questions

**Q: What if I disagree with the oracle data?**

A: Oracle data comes from satellites and weather stations, not human judgment. Multiple sources are used and median is calculated to ensure accuracy.

**Q: Can I get partial payout?**

A: Yes! Crop stress trigger pays 50% of coverage. Drought trigger pays 100%.

**Q: What if it rains after payout?**

A: Payout is based on season-total rainfall. Once triggered, payout is final.

**Q: How do I know the payout is fair?**

A: All data is public on-chain. You can verify:
- Oracle readings
- Trigger conditions
- Payout calculation

### Technical Questions

**Q: What if the oracle fails?**

A: Multiple oracle nodes submit data. Median is used, so single failure doesn't affect result.

**Q: What if the smart contract has a bug?**

A: Contracts are audited and tested. However, smart contracts are experimental technology. Only insure what you can afford to lose.

**Q: What blockchain is this on?**

A: Stellar blockchain with Soroban smart contracts. Fast, low-cost, and environmentally friendly.

**Q: What are the fees?**

A: Network fees are ~$0.01 per transaction. Premium is 5-10% of coverage.

### Support Questions

**Q: Who do I contact for help?**

A: 
- Email: support@ceres.network
- Telegram: @CeresSupport
- WhatsApp: +XXX-XXX-XXXX
- Local agent: [Find agent](https://ceres.network/agents)

**Q: Is there a tutorial video?**

A: Yes! Visit [ceres.network/tutorials](https://ceres.network/tutorials) for videos in multiple languages.

**Q: Can I get help in my language?**

A: We support:
- English
- Swahili
- Hindi
- Spanish
- French
- More coming soon!

## Next Steps

1. **Set up wallet** - Download Freighter or Lobstr
2. **Get USDC** - Buy from exchange or local dealer
3. **Register policy** - Visit app.ceres.network
4. **Monitor season** - Check dashboard weekly
5. **Receive payout** - Automatic if triggered!

## Important Reminders

⚠️ **Security**:
- Never share your recovery phrase
- Never send crypto to someone claiming to be support
- Always verify you're on the official website

⚠️ **Risk**:
- Parametric insurance doesn't cover all risks
- Only insure what you can afford to lose
- Smart contracts are experimental technology

⚠️ **Responsibility**:
- You are responsible for your wallet security
- You are responsible for understanding your policy
- Read all terms before registering

---

**Welcome to Ceres Network!** 🌾

We're here to help protect your livelihood and give you peace of mind during the growing season.

For more help, visit [ceres.network/support](https://ceres.network/support)
