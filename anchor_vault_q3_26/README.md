# Solana Vault

![Project Screenshot](testpassing.png)

A **Solana** program built with **Anchor**. Each user gets their own vault where they can deposit SOL, withdraw SOL, and close the vault when they are done.

## What is this?

Think of it as a **personal piggy bank on Solana**.

You can:

- Create your vault
- Deposit SOL
- Withdraw SOL
- Close the vault and get the remaining SOL and rent back

> `1 SOL = 1,000,000,000 lamports`

## Key Concepts

| Word | Meaning |
| --- | --- |
| **Program** | Code that runs on the Solana blockchain. |
| **Account** | A place that stores data or SOL. |
| **Instruction** | An action performed by the program. |
| **Signer** | The wallet that signs the transaction. |
| **PDA** | A program-controlled address used for the vault. |
| **Bump** | A value used to derive a PDA. |
| **Rent** | SOL required to keep an account on Solana. |
| **CPI** | One Solana program calling another program. |
| **Lamports** | The smallest unit of SOL. |

## Instructions

| Instruction | What it does |
| --- | --- |
| `initialize` | Creates `vault_state` and funds the user's `vault` with rent-exempt SOL. |
| `deposit(amount)` | Transfers SOL from the user to the vault. |
| `withdraw(amount)` | Transfers SOL from the vault back to the user. |
| `close` | Sends the remaining vault SOL to the user and closes `vault_state`. |

## PDAs

Each user has two PDAs:

- `vault_state` — seeds: `["state", user]`
- `vault` — seeds: `["vault", user]`

`vault_state` stores the PDA bumps and `vault` holds the user's SOL.

Because the user's public key is part of the seeds, each user gets their own vault.

## Build and Test

```bash
anchor build
anchor test
Tests run with LiteSVM. The test covers initialize → deposit 0.5 SOL → withdraw 0.1 SOL → close.

