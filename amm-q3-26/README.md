# AMM — Solana-Anchor

A simple two-token Automated Market Maker (AMM) built on Solana using the Anchor framework and a constant-product curve library.

This project implements the required AMM functionality:

* Pool initialization
* Liquidity deposits
* Liquidity withdrawals
* Token swaps
* Protocol fees
* Treasury accounts
* Tests covering all four instructions

# Overview

An Automated Market Maker allows users to trade tokens against liquidity held inside a pool without requiring a traditional order book.

This AMM contains two tokens:

```text
Token X
Token Y
```

# Project Structure

```text
amm-q3-26/
├── programs/
│   └── amm-q3-26/
│       └── src/
│           ├── instructions/
│           │   ├── initialise.rs
│           │   ├── deposit.rs
│           │   ├── withdraw.rs
│           │   └── swap.rs
│           ├── state/
│           │   └── config.rs
│           ├── constants.rs
│           ├── error.rs
│           └── lib.rs
├── tests/
│   └── amm-q3-26.ts
├── Anchor.toml
├── Cargo.toml
└── package.json
```

# AMM Flow

```text
Initialize
    ↓
Create Pool
    ↓
Deposit X + Y
    ↓
Receive LP Tokens
    ↓
Swap X ↔ Y
    ↓
Protocol Fee → Treasury
    ↓
Withdraw
    ↓
Burn LP Tokens
    ↓
Receive X + Y
```

# Testing

The project uses TypeScript - Anchor - Mocha for its tests. You can also use LiteSVM, but I have not used it here to keep it simple and direct.

The tests are located at:

```text
tests/amm-q3-26.ts
```

Run all tests with:

```bash
anchor test
```

# Complete AMM Lifecycle

```text
                         INITIALIZE
                              │
                              ▼
                         Create Pool
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
           Vault X         Vault Y         LP Mint
                                              │
                                              ▼
                                           DEPOSIT
                                              │
                                              ▼
                                          User gets
                                          LP tokens
                                              │
                         ┌────────────────────┴─────────────────┐
                         │                                      │
                         ▼                                      ▼
                       SWAP                                  WITHDRAW
                         │                                      │
                         ▼                                      ▼
              Protocol fee → Treasury                 Burn LP tokens
                                                                │
                                                                ▼
                                                         Receive X + Y
```

# Summary

This project implements a basic two-token AMM on Solana.

The main flow is:

```text
Initialize
    ↓
Create pool
    ↓
Deposit liquidity
    ↓
Receive LP tokens
    ↓
Swap tokens
    ↓
Protocol fee → Treasury
    ↓
Withdraw liquidity
    ↓
Burn LP tokens
    ↓
Receive Token X + Token Y
```

