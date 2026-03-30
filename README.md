# Swifey Bonding Curve Protocol

[![License: ISC](https://img.shields.io/badge/License-ISC-blue.svg)](https://opensource.org/licenses/ISC)
[![Anchor](https://img.shields.io/badge/Anchor-0.32.1-purple.svg)](https://github.com/coral-xyz/anchor)
[![Solana](https://img.shields.io/badge/Solana-Compatible-green.svg)](https://solana.com)

A robust, Anchor-based Solana smart contract implementing a **Bancor-style bonding curve** for fair-launch token distribution. Users can permissionlessly launch tokens, trade with dynamic AMM pricing, and automatically migrate liquidity to Raydium CLMM upon completion.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Instructions](#instructions)
- [Bonding Curve Math](#bonding-curve-math)
- [Configuration](#configuration)
- [Security Considerations](#security-considerations)
- [Building & Testing](#building--testing)
- [Deployment](#deployment)

---

## Overview

This protocol enables:

1. **Fair Launch** - Anyone can create a token with no upfront liquidity requirements
2. **Dynamic Pricing** - Token price increases as demand grows (Bancor formula)
3. **Automatic Migration** - Upon reaching the SOL cap, liquidity migrates to Raydium
4. **Fee Capture** - Configurable trading fees accrue to protocol treasury

---

## Features

### Token Launchpad

- **Permissionless Creation** - Deploy tokens with name, symbol, and metadata URI
- **Fixed Supply** - 100% supply minted to curve PDA at launch
- **Mint Authority Renounced** - Supply is immutable after initialization
- **Metaplex Metadata** - Standard token metadata created automatically

### Bonding Curve AMM

- **Bancor Formula** - `Price = SOL Reserve / (Token Reserve × CRR)`
- **Virtual Reserves** - Synthetic liquidity enables low-capital launches
- **Bi-directional Trading** - Buy (SOL→Token) and Sell (Token→SOL)
- **Slippage Protection** - `min_out` parameter enforces maximum acceptable slippage

### Fee System

| Fee Type      | Description                       | Recipient     |
| ------------- | --------------------------------- | ------------- |
| Buy Fee       | % of SOL spent on purchases       | Fee Recipient |
| Sell Fee      | % of SOL returned on sales        | Fee Recipient |
| Migration Fee | % of SOL during Raydium migration | Fee Recipient |

### Migration to Raydium

- **Automatic Trigger** - When virtual SOL reserve reaches `curve_limit`
- **CLMM Pool** - Concentrated liquidity market maker on Raydium V3
- **One-Click** - Admin calls `migrate` to transfer all liquidity

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Actions                          │
├─────────────┬─────────────┬────────────────┬────────────────┤
│    Launch   │    Buy      │     Sell       │   (Admin Only) │
│   (create)  │  (swap:0)   │   (swap:1)     │   Enable/Drain │
└──────┬──────┴──────┬──────┴────────┬───────┴────────────────┘
       │             │              │
       ▼             ▼              ▼
┌─────────────────────────────────────────────────────────────┐
│                    BondingCurve PDA                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │Virtual SOL   │  │Virtual Token │  │   Real SOL       │   │
│  │Reserve       │  │Reserve       │  │   (fees held)    │   │
│  └──────────────┘  └──────────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ (when limit reached)
                    ┌─────────────────────┐
                    │  Raydium CLMM Pool  │
                    │   (migration)       │
                    └─────────────────────┘
```

---

## Quick Start

### Prerequisites

```bash
# Install Solana CLI
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.vercel.app | bash

# Install Anchor
npm install -g @coral-xyz/anchor-cli

# Verify installations
solana --version
anchor --version
```

### Build

```bash
# Standard build (without Raydium migration)
anchor build

# Build with migration feature
anchor build --features "migration"
```

### Test

```bash
# Start local validator
solana-test-validator

# In another terminal
anchor test
```

### Deploy

```bash
# Configure for devnet
solana config set --url devnet

# Create wallet (if needed)
solana-keygen new -o ~/.config/solana/id.json

# Airdrop SOL
solana airdrop 2

# Deploy
anchor deploy
```

---

## Instructions

### Admin Instructions

#### `configure` - Initialize Protocol

Sets up global config: authority, fee recipient, curve parameters, fee rates.

#### `set_params` - Update Parameters

Dynamically adjust fees and limits without program upgrade.

#### `enable_trading` - Toggle Trading

Enable/disable swaps for a specific curve. Prevents sniping at launch.

#### `withdraw_fees` - Extract SOL

Drain accumulated fees from curve PDA (rent-exempt minimum preserved).

#### `migrate` - Move to Raydium

Transfer liquidity to Raydium CLMM pool (requires `migration` feature).

### Public Instructions

#### `launch` - Create Token

```rust
launch(
    name: String,      // Token name
    symbol: String,   // Token symbol
    uri: String       // Metadata JSON URI
)
```

#### `swap` - Trade

```rust
swap(
    amount: u64,      // Input amount
    direction: u8,    // 0 = Buy, 1 = Sell
    min_out: u64,     // Minimum output (slippage)
    bump_bonding_curve: u8
)
```

---

## Bonding Curve Math

### Formula

```
Token Price (in SOL) = Virtual_SOL_Reserve / (Virtual_Token_Reserve × CRR)
```

Where:

- **CRR** (Constant Reserve Ratio) = 0.2 (20%)
- **Virtual Reserves** = Synthetic values that set initial price
- **Real Reserves** = Actual SOL/tokens held by the curve

### Buy Calculation

```rust
// Given SOL input, calculate token output
base = 1 + (SOL_in / Virtual_SOL_Reserve)
tokens_out = Virtual_Token_Reserve × (base^CRR - 1)
```

### Sell Calculation

```rust
// Given token input, calculate SOL output
base = 1 - (Tokens_in / Virtual_Token_Reserve)
SOL_out = Virtual_SOL_Reserve × (1 - base^(1/CRR))
```

### Example Progression

| SOL In Curve | Token Reserve | Price (SOL) | Marketcap |
| ------------ | ------------- | ----------- | --------- |
| 12.33        | 800,000,000   | 0.000077    | ~61,872   |
| 20.00        | 750,000,000   | 0.000133    | ~100,000  |
| 30.00        | 650,000,000   | 0.000230    | ~150,000  |
| 42.00        | 450,000,000   | 0.000467    | ~210,000  |

---

## Configuration

### Default Constants (`constants.rs`)

```rust
TOKEN_DECIMAL: u8 = 6;                              // 6 decimal places
TARGET_SOL_AMOUNT: u64 = 42_000_000_000;           // 42 SOL cap
INITIAL_SOL_RESERVE: u64 = 12_330_000_000;         // 12.33 SOL virtual
TOKEN_RESERVE_PERCENTAGE: f64 = 0.8;               // 80% of supply in curve
CRR: f64 = 0.2;                                    // Constant Reserve Ratio
LAMPORTS_PER_SOL: u64 = 1_000_000_000;
```

### Config Account Fields

| Field                 | Description               | Example                 |
| --------------------- | ------------------------- | ----------------------- |
| `authority`           | Protocol admin            | `7nxn...`               |
| `fee_recipient`       | Fee collection wallet     | `3xKp...`               |
| `curve_limit`         | SOL target for completion | `42_000_000_000`        |
| `buy_fee_percentage`  | Buy fee %                 | `1.0` (1%)              |
| `sell_fee_percentage` | Sell fee %                | `1.0` (1%)              |
| `total_token_supply`  | Max token supply          | `1_000_000_000_000_000` |

---

## Security Considerations

### Known Limitations

1. **Centralization Risk**: Single admin controls fee parameters and migration
2. **f64 Math**: Floating-point calculations may have minor rounding inconsistencies
3. **Migration Dependency**: Requires external Raydium program availability

### Access Controls

| Function         | Caller | Verification                 |
| ---------------- | ------ | ---------------------------- |
| `configure`      | Admin  | `signer == config.authority` |
| `set_params`     | Admin  | `signer == config.authority` |
| `enable_trading` | Admin  | `signer == config.authority` |
| `withdraw_fees`  | Admin  | `signer == config.authority` |
| `migrate`        | Admin  | `signer == config.authority` |
| `launch`         | Anyone | No restriction               |
| `swap`           | Anyone | Trading enabled              |

### Safety Checks

- ✅ Rent-exempt minimum preserved on withdrawals
- ✅ Checked arithmetic preventing overflows
- ✅ Slippage protection on all swaps
- ✅ Trading gate (`is_trading_enabled`)
- ✅ Curve completion prevents further swaps
- ✅ Fee bounds validation (0-100%)

---

## Building & Testing

### Build Commands

```bash
# Default (lightweight, no migration)
anchor build

# With Raydium migration
anchor build --features "migration"

# Release build
cargo build-sbf --release
```

### Test Commands

```bash
# Run all tests
anchor test

# Run with logs
anchor test -- --nocapture

# Specific test
anchor test --grep "launch"
```

### Project Structure

```
bonding_curve/
├── programs/
│   └── bonding_curve/
│       ├── src/
│       │   ├── instructions/    # Transaction handlers
│       │   ├── state/           # Account schemas
│       │   ├── utils/           # Helpers & math
│       │   ├── constants.rs     # Protocol params
│       │   ├── errors.rs        # Error codes
│       │   └── lib.rs           # Program entry
│       └── Cargo.toml
├── tests/
│   └── bonding_curve.ts        # TypeScript tests
├── Anchor.toml                 # Anchor config
└── README.md                   # This file
```

---

## Deployment

### Devnet

```bash
solana config set --url devnet
anchor deploy
```

### Mainnet

```bash
solana config set --url mainnet-beta
anchor build --features "migration" --release
anchor deploy --provider.cluster mainnet
```

**Note**: Mainnet deployment requires:

- Sufficient SOL for rent exemption
- Upgraded BPF loader
- Recommended: Program multisig upgrade authority

---

## License

ISC © 2024 Swifey Protocol

---

## Disclaimer

This software is provided as-is without warranties. Use at your own risk. Smart contracts are experimental technology - always audit before mainnet use.
