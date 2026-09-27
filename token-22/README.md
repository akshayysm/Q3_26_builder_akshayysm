# Token-22 — Remittance USD & Confidential Transfers

![Test passing](./t22testpassing.png)

An Anchor program on Solana implementing **Remittance USD (RUSD)** using the SPL **Token-2022** standard. The program provides protocol-level transfer fees, frozen-by-default token accounts for KYC gating, on-chain metadata, and an optional confidential-transfer mint.

---

## Overview

The program provides two RUSD mint configurations:

1. **Standard Mint** — 1% transfer fee, frozen-by-default accounts, mint close authority, and on-chain metadata.
2. **Confidential Mint** — standard features plus confidential balances, confidential transfer fees, and a permanent delegate.

The program is a thin orchestration layer over Token-2022 and performs most token operations through CPI.

---

## Program Details

* **Program ID:** `3kjBCVZPCqW6e6JLFCBkTWPocDH1TD6GgB6XEiLw1qVc`
* **Framework:** Anchor / Rust
* **Token Program:** SPL Token-2022
* **Decimals:** `6`
* **Transfer Fee:** `100 bps` (1%)
* **Maximum Fee:** `u64::MAX`

---

## Token Specifications

### Standard Mint

| Extension             | Purpose                                           |
| --------------------- | ------------------------------------------------- |
| `TransferFeeConfig`   | Applies a 1% transfer fee                         |
| `MintCloseAuthority`  | Allows the configured authority to close the mint |
| `DefaultAccountState` | New token accounts start as `Frozen`              |
| `MetadataPointer`     | Points metadata to the mint                       |
| `TokenMetadata`       | Stores token name, symbol, and URI                |

### Confidential Mint

The confidential mint includes the standard extensions plus:

| Extension                       | Purpose                                               |
| ------------------------------- | ----------------------------------------------------- |
| `PermanentDelegate`             | Configures a permanent delegate                       |
| `ConfidentialTransferMint`      | Enables encrypted balances and confidential transfers |
| `ConfidentialTransferFeeConfig` | Enables encrypted fee withholding                     |

---

## RUSD Configuration

| Parameter    | Value                           |
| ------------ | ------------------------------- |
| Name         | `Remittance USD`                |
| Symbol       | `RUSD`                          |
| Decimals     | `6`                             |
| Transfer Fee | `100 bps (1%)`                  |
| Maximum Fee  | `u64::MAX`                      |

Example:

```text
Transfer:       10,000
Fee (1%):          100
Recipient:        9,900
Withheld Fee:       100
```

---

## Instructions

### `initialize`

Creates the standard RUSD mint with:

* Transfer fee
* Frozen default account state
* Mint close authority
* Metadata pointer
* Token metadata

### `transfer_with_fee`

Transfers RUSD using Token-2022 `transfer_checked_with_fee`.

**Parameters:**

```text
amount: u64
decimals: u8
```

The transfer fee is calculated from the Token-2022 configuration for the current epoch.

### `unfreeze_kyc_account`

Thaws an individual frozen token account.

New token accounts remain frozen by default; this instruction only thaws the selected account.

> KYC verification itself is performed off-chain. This instruction only provides the on-chain thaw operation.

### `initialize_confidential_mint`

Creates the confidential RUSD mint.

**Parameters:**

```text
withdraw_withheld_authority_elgamal_pubkey: [u8; 32]
```

### `deposit_confidential`

Moves public tokens into the account's encrypted pending balance.

**Parameters:**

```text
amount: u64
decimals: u8
```

### `apply_pending_balance`

Moves pending confidential credits into the available confidential balance.

**Parameters:**

```text
expected_pending_balance_credit_counter: u64
new_decryptable_available_balance: [u8; 36]
```

---

## Public Token Flow

```text
              initialize
                  │
                  ▼
             RUSD Mint
                  │
                  ▼
        Create Token Account
                  │
                  ▼
               Frozen
                  │
          KYC verification
                  │
                  ▼
       unfreeze_kyc_account
                  │
                  ▼
              Thawed
                  │
                  ▼
        transfer_with_fee
                  │
                  ▼
          1% fee withheld
```

The mint's default state remains `Frozen`, so newly created accounts still require the same process.

---

## Confidential Token Flow

```text
┌──────────────────────┐
│ Public Token Balance │
└──────────┬───────────┘
           │
           │ deposit_confidential
           ▼
┌────────────────────────────┐
│ Pending Confidential       │
│ Balance                    │
└──────────┬─────────────────┘
           │
           │ apply_pending_balance
           ▼
┌────────────────────────────┐
│ Available Confidential     │
│ Balance                    │
└──────────┬─────────────────┘
           │
           │ confidential transfer
           ▼
┌────────────────────────────┐
│ Recipient Pending Balance  │
└──────────┬─────────────────┘
           │
           │ apply_pending_balance
           ▼
┌────────────────────────────┐
│ Recipient Available        │
│ Confidential Balance       │
└────────────────────────────┘
```

The current program wraps the **deposit** and **apply** operations.

The confidential transfer and withdrawal operations are exercised directly through Token-2022 in the integration tests.

---

## Confidential Example

The test lifecycle uses:

```text
Alice deposits       10,000
Alice applies        10,000 available

Alice transfers       2,500
Transfer fee             25
Bob receives           2,475 pending

Bob applies            2,475 available

Bob withdraws          1,000
Remaining              1,475
```

---

## Token-2022 Architecture

The program does not implement a replacement token standard.

```text
Client
  │
  ▼
┌──────────────────────┐
│      Token-22        │
│   Anchor Program     │
└──────────┬───────────┘
           │ CPI
           ▼
┌──────────────────────┐
│      Token-2022      │
│   SPL Token Program  │
└──────────┬───────────┘
           │
           ▼
      Mint / Accounts
```

Most token functionality remains implemented by Token-2022.

---

## Mint Allocation

Token Metadata is variable-length, so the mint cannot be sized using only the fixed extension list.

The initialization flow is:

```text
Calculate fixed extension size
            │
            ▼
Calculate metadata size
            │
            ▼
Create mint account
            │
            ▼
Initialize extensions
            │
            ▼
Initialize mint
            │
            ▼
Write Token Metadata
```

The program uses Token-2022 extension APIs to calculate and initialize the required account space.

---

## Authorities

For the current implementation, the payer is used for the main mint authorities:

* Mint authority
* Freeze authority
* Transfer-fee authority
* Withdraw-withheld authority
* Mint close authority
* Metadata update authority

The confidential mint additionally configures:

* Permanent delegate
* Confidential-transfer authority
* Confidential fee authority

Production deployments should normally separate these authorities according to the application's trust model.

---

## Project Structure

```text
token-22/
├── Anchor.toml
├── Cargo.toml
├── rust-toolchain.toml
│
├── programs/
│   └── t22/
│       └── src/
│           ├── lib.rs
│           ├── constants.rs
│           └── instructions/
│               ├── ...
│               └── ...
│
└── tests/
    ├── test_all.rs
    └── handlers/
        ├── ...
        └── ...
```

### Important Files

| File                             | Purpose                     |
| -------------------------------- | --------------------------- |
| `programs/t22/src/lib.rs`        | Program entrypoints         |
| `programs/t22/src/constants.rs`  | RUSD configuration          |
| `programs/t22/src/instructions/` | Instruction implementations |
| `tests/test_all.rs`              | LiteSVM integration tests   |
| `tests/handlers/`                | Test helpers                |
| `Anchor.toml`                    | Anchor configuration        |

---

## Building & Testing

### Build

```bash
anchor build
```

### Test

```bash
cargo test
```

or:

```bash
anchor test
```

Tests use **LiteSVM** and load:

```text
target/deploy/t22.so
```

No `solana-test-validator` is required.

---

## Test Coverage

The integration tests cover:

* Standard mint initialization
* Token-2022 extension configuration
* On-chain metadata
* Transfer fee calculation
* Frozen-by-default accounts
* KYC account thawing
* Confidential mint initialization
* Confidential deposits
* Applying pending confidential balances
* Confidential transfer with fee
* Confidential withdrawal lifecycle

---

## Notes

* `initialize` and `initialize_confidential_mint` are separate mint configurations.
* Token-2022 extensions must be configured during account creation.
* `unfreeze_kyc_account` only performs the on-chain thaw; it does not implement KYC.
* Confidential transfers require client-side cryptographic operations.
* Confidential transfer and withdrawal are currently demonstrated through Token-2022 directly in the test harness.
* The project is an implementation/demo of the Token-2022 architecture and should undergo additional security and authority review before production deployment.


