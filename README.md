# Solana Bonding Curve Protocol

A robust, Anchor-based Solana smart contract that implements a continuous token bonding curve. It allows users to permissionlessly launch tokens, trade them with dynamic AMM-style pricing, and automatically migrate liquidity to a Raydium Concentrated Liquidity Market Maker (CLMM) pool once a target SOL threshold is met.

## Features & Mechanics

### 1. Token Launchpad
- **Permissionless Launch:** Anyone can deploy a new curve by providing `name`, `symbol`, and `uri`.
- **Integrated Metadata:** Automatically creates standard Metaplex Token Metadata for the new SPL token.
- **Fixed Supply:** The entire token supply is minted to the curve's Token Account upon initialization, and the mint authority is permanently renounced.

### 2. Bancor Curve Pricing Math
The protocol utilizes the proven **Bancor Pricing Formula** for dynamic price discovery:
- **Price calculation:** `Reserve SOL / (Reserve Tokens * Constant Reserve Ratio (CRR))`
- Prices rise automatically as more SOL enters the curve, and drop as tokens are sold back.
- Virtual Reserves vs Real Reserves: The Curve uses "synthetic/virtual liquidity" to manipulate the initial price floor to a specific configuration without requiring massive upfront SOL capital.

### 3. Core Trade Logic (`swap`)
- Supports bi-directional trading: `buy` (SOL -> Token) and `sell` (Token -> SOL).
- **Slippage Protection:** Minimum tokens out / minimum SOL out arguments are strictly enforced on every trade.
- **Trading Gate:** Swaps can be paused or initialized by the protocol authority via the `is_trading_enabled` flag.

### 4. Fee Ecosystem
- Highly configurable fee architecture globally managed by an admin.
- Takes dynamic percentages on **buys**, **sells**, and during the final **Raydium Migration**.
- **Authority Withdrawal:** The protocol manager can call `withdraw_fees` at any time to extract accrued SOL to the designated `fee_recipient` wallet, leaving strict rent-exemption checks intact to prevent bricking the PDA.

### 5. Automated Raydium Migration
Once a bonding curve reaches its hard `TARGET_SOL_AMOUNT` limit (e.g., 42 SOL), it flips to an `is_completed` state where trading via the curve ceases. 
- The `migrate` instruction ports the remaining tokens and all real accumulated SOL over to **Raydium AMM V3**.
- A standard Raydium liquidity pool is initialized on the fly.
- Note: This functionality is gated using the Cargo `migration` feature flag to keep baseline builds light and clean without Raydium dependencies.

---

## Instructions Overview

| Instruction | Actor | Description |
|---|---|---|
| `configure` | Admin | One-time initialization of the global configuration (fees, target limits, protocol manager). |
| `set_params` | Admin | Tweak dynamic parameters (buy/sell fees) without needing to upgrade the program. |
| `enable_trading` | Admin | Activate or pause trading on a specific token's curve PDA. |
| `withdraw_fees` | Admin | Drain excess accrued SOL from the PDA into the global fee recipient wallet. |
| `launch` | Anyone | Create a new token, set up metadata, and establish its bonding curve constraints. |
| `swap` | Anyone | Buy or sell tokens. Automatically triggers the `CurveCompleted` phase if threshold hits. |
| `migrate` | Admin | Moves a finalized curve's liquidity into a Raydium DEX pool. |

---

## Technical Specifications & Constants

The default constraints are encoded in `constants.rs`:
- **Target Cap:** 42 SOL (`42_000_000_000` lamports)
- **Token Decimals:** `6`
- **Constant Reserve Ratio (CRR):** `1.314` (Drives the steepness of the curve)
- **Token Allocation Weight:** `0.80` (80% of supply actively traded on the curve)

## Building the Protocol

The program utilizes Solana's `anchor-lang`.

**Default Protocol Build (Without Raydium Migration feature):**
```bash
anchor build
```

**Build with Raydium CLMM Migration enabled:**
```bash
cargo build-sbf --features "migration"
```

## Protocol Protection & Auditing
- Standard math operations utilize Rust's checked math via `checked_add`, `checked_mul` to absolutely prevent overflows.
- Authority constraints natively govern param setting and migrations.
- Complete bounds checking limits fees strictly between `0-100%`.
