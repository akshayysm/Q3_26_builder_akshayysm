# Token-2022 Remittance USD (RUSD)

![Tests](./t22testpassing.png)

An Anchor program on Solana that issues a **Token-2022** compliant remittance stablecoin (**RUSD**). 

Instead of implementing a custom token protocol, this program initializes token mints with designated Token-2022 extensions and executes Cross-Program Invocations (CPIs) into the Token-2022 standard program.

**Program ID:** `3kjBCVZPCqW6e6JLFCBkTWPocDH1TD6GgB6XEiLw1qVc`

---

## Overview

The program implements two distinct mint configurations. Because Token-2022 extension definitions are immutable after initialization, the confidential mint exists as a dedicated re-issue rather than an in-place upgrade.

| Requirement | Implementation Mechanism |
|---|---|
| Protocol revenue per transfer | 1% protocol fee via `TransferFeeConfig` |
| Default wallet freeze until KYC | `DefaultAccountState` set to **Frozen**; granular thaw via freeze authority |
| Native on-chain metadata | Integrated `TokenMetadata` + `MetadataPointer` (self-referencing mint) |
| Controlled mint decommission | `MintCloseAuthority` (callable when total supply is zero) |
| Confidential transfers & regulatory seizure | `ConfidentialTransferMint` paired with `PermanentDelegate` |

---

## Token Specifications

| Parameter | Configuration |
|---|---|
| **Name** | Remittance USD |
| **Symbol** | RUSD |
| **Decimals** | 6 |
| **Metadata URI** | `https://example.com/rusd.json` |
| **Transfer Fee** | 100 bps (1.00%) |
| **Maximum Fee** | `u64::MAX` |

> **Fee Calculation Example:**  
> Transfer `10,000` RUSD -> Destination receives `9,900` RUSD, and `100` RUSD is withheld on the destination token account.

*Authority Model:* In testing environments, the payer key holds all authorities (mint, freeze, transfer fee, close, metadata, permanent delegate, and confidential transfer). Production deployments should partition these authorities across distinct multisig or governance accounts.

---

## Mint Architectures & Flows

### 1. Public Mint (`initialize`)

Account space is dynamically calculated using `ExtensionType::try_calculate_account_len` and initialized prior to `InitializeMint` / `initialize_mint2`.

#### Enabled Extensions

* `TransferFeeConfig` — Enforces 1% transfer fee
* `DefaultAccountState` — Forces newly created token accounts into a **Frozen** state
* `MetadataPointer` — Points directly to the mint address for metadata resolution
* `TokenMetadata` — Writes name, symbol, and URI directly to mint storage
* `MintCloseAuthority` — Allows closing the mint account if total supply is zero

#### Transaction Flow

```text
initialize
    └── Mint initialized with extensions (fee, default-frozen, metadata pointer)

create_associated_token_account
    └── Account initialized in Frozen state (transfers disabled)

off-chain verification (KYC)
    └── unfreeze_kyc_account (invoked by Freeze Authority)
    └── Target account thawed (mint-level default remains Frozen)

mint_to (Token-2022 CPI)
    └── Mints public tokens to thawed account

transfer_with_fee(amount, decimals)
    ├── Reads active fee configuration via StateWithExtensions
    ├── Calculates epoch fee via calculate_epoch_fee(epoch, amount)
    └── Executes transfer_checked_with_fee CPI (withheld fee sits on recipient ATA)
```

---

### 2. Confidential Mint (`initialize_confidential_mint`)

Shares all base extensions with the public mint, adding zero-knowledge confidential transfer primitives.

#### Additional Extensions

* `PermanentDelegate` — Authorizes compliance asset seizure without owner signatures
* `ConfidentialTransferMint` — Encrypted balance tracking with manual approval gate (`auto_approve_new_accounts = false`)
* `ConfidentialTransferFeeConfig` — Encrypted fee withholdings (mandatory when transfer fees coexist with confidential transfers)

*Instruction Argument:* `withdraw_withheld_authority_elgamal_pubkey` (`[u8; 32]`)

#### Token Account Balance States

Every confidential token account tracks balances across three distinct fields:
1. **Public Balance** — Standard visible SPL balance
2. **Pending Confidential Balance** — Inbound encrypted transfers or deposits (non-spendable)
3. **Available Confidential Balance** — Decrypted, spendable confidential balance

#### Transaction Flow

```text
initialize_confidential_mint
    └── Mint created with public + confidential + delegate extensions

create_associated_token_account
    └── Initialized as Frozen

KYC Verification & Setup
    ├── unfreeze_kyc_account (Freeze Authority thaws account)
    ├── ConfigureAccount (Account owner registers ElGamal keys)
    └── ApproveAccount (Confidential authority approves account for ZK transfers)

Deposit & Settlement
    ├── mint_to (Mint public RUSD)
    ├── deposit_confidential (Public balance decreases -> Pending balance increases)
    └── apply_pending_balance (Pending balance -> Available spendable balance)

Confidential Transfer
    ├── Confidential transfer with encrypted 1% fee
    ├── Recipient pending balance increases (net of transfer fee)
    ├── Recipient executes apply_pending_balance
    └── (Optional) WithdrawConfidentialTokens converts available encrypted tokens to public
```

#### Balance Lifecycle Example

```text
Alice deposits 10,000 RUSD  -> apply_pending_balance  -> Available: 10,000 RUSD
Alice transfers 2,500 RUSD  -> Fee: 25 RUSD           -> Bob Pending: 2,475 RUSD
Bob applies pending balance                         -> Bob Available: 2,475 RUSD
Bob withdraws 1,000 RUSD    -> Public: 1,000 RUSD    -> Bob Available: 1,475 RUSD
```

---

## Program Instructions

All state deserialization across transfer paths leverages `StateWithExtensions` to prevent truncation vulnerabilities.

| Instruction | Accounts / Args | Purpose |
|---|---|---|
| `initialize` | None | Initializes public RUSD mint and metadata |
| `transfer_with_fee` | `amount: u64`, `decimals: u8` | Executes epoch-adjusted fee transfer CPI |
| `unfreeze_kyc_account` | Target token account | Thaws an account after external KYC |
| `initialize_confidential_mint` | `elgamal_pubkey: [u8; 32]` | Initializes confidential RUSD mint |
| `deposit_confidential` | `amount: u64`, `decimals: u8` | Moves tokens from public to pending confidential |
| `apply_pending_balance` | `expected_pending_balance_credit_counter: u64`, `new_decryptable_available_balance: [u8; 36]` | Promotes pending confidential to available balance |

---

## Project Structure

```text
token-22/

└── programs/t22/
    ├── src/
    │   ├── lib.rs                  # Program entrypoint and instruction routing
    │   ├── constants.rs            # Token constants (metadata, fees, decimals)
    │   └── instructions/           # Modular instruction handlers
    │       ├── apply.rs 
    │       ├── confidential_mint.rs 
    │       ├── deposit.rs 
    │       ├── increment.rs confidential_mint.rs
    │       ├── initialize.rs
    │       ├── transfer.rs
    │       └── unfreeze.rs
    └── tests/
        ├── test_all.rs             # LiteSVM test suite
        └── handlers/               # Transaction wrappers and assertion helpers
```

---

## Building & Testing

### Build Instructions

```bash
# Build the program binary
anchor build
```

### Running Tests

The test suite runs in-process via **LiteSVM**, bypassing the overhead of a full local test validator.

```bash
# Execute LiteSVM integration tests
cargo test
# or
anchor test --skip-local-validator
```
