# Escrow

A Solana on-chain escrow program built with [Anchor](https://www.anchor-lang.com/) for trustless token-for-token swaps. A **maker** locks one SPL token in a program-controlled vault and specifies how much of a second token they want in return. A **taker** can accept the offer by paying the requested amount, or the **maker** can cancel and reclaim their deposit.

This project is part of the [Turbin3](https://turbin3.com/) builder curriculum (Week 3) and demonstrates core Solana concepts: PDAs, associated token accounts, cross-program invocations (CPIs), and account validation.

**Program ID:** `G6AmazQLVY6h9qnHFKAjNy8xtCzEJ1xogJWT2s6agERi`

---

## How It Works

The escrow implements a simple two-token swap with three instructions:

```
Maker                          Program                         Taker
  │                               │                               │
  │──── make ────────────────────►│  Creates Escrow + Vault PDA   │
  │     (deposit mint_a)          │  Locks tokens in vault         │
  │                               │                               │
  │                               │◄──── take ────────────────────│
  │                               │  Taker pays mint_b to maker   │
  │                               │  Taker receives mint_a        │
  │                               │  Escrow + vault closed        │
  │                               │                               │
  │──── refund ──────────────────►│  Returns mint_a to maker      │
  │     (maker only)              │  Escrow + vault closed        │
```

| Instruction | Who calls it | What happens |
|-------------|--------------|--------------|
| `make` | Maker | Creates an `Escrow` account and token vault, deposits `mint_a` into the vault |
| `take` | Taker | Pays `mint_b` to the maker, receives `mint_a` from the vault, closes accounts |
| `refund` | Maker | Returns all vaulted `mint_a` to the maker, closes accounts |

### Example

1. Alice (maker) wants to trade **100 USDC** (`mint_a`) for **50 BONK** (`mint_b`).
2. Alice calls `make` with `deposit = 100`, `receive = 50`, and a unique `seed`.
3. Bob (taker) calls `take` — he sends 50 BONK to Alice and receives 100 USDC from the vault.
4. Alternatively, if no one accepts, Alice calls `refund` to get her 100 USDC back.

---

## On-Chain Accounts

### Escrow state

The `Escrow` account stores the terms of the offer:

| Field | Type | Description |
|-------|------|-------------|
| `seed` | `u64` | User-chosen identifier so a maker can open multiple escrows |
| `maker` | `Pubkey` | Wallet that created the escrow |
| `mint_a` | `Pubkey` | Token the maker deposits (what the taker receives) |
| `mint_b` | `Pubkey` | Token the maker wants in return (what the taker pays) |
| `receive` | `u64` | Amount of `mint_b` the taker must pay |
| `bump` | `u8` | PDA bump seed for the escrow account |

### PDAs

Two program-derived addresses are created per escrow:

**Escrow PDA** — stores the offer metadata and acts as the vault authority.

```
seeds = ["escrow", maker_pubkey, seed.to_le_bytes()]
```

**Vault** — an associated token account owned by the escrow PDA, holding the deposited `mint_a`.

```
seeds = [escrow_pda, token_program_id, mint_a]
program = Associated Token Program
```

The escrow PDA signs token transfers out of the vault via CPI with signer seeds.

---

## Instructions

### `make(seed, receive, deposit)`

Creates a new escrow and funds the vault.

**Accounts:** maker (signer), mint_a, mint_b, maker_ata_a, escrow (init), vault (init), token_program, associated_token_program, system_program

**Logic:**
1. Initialize the `Escrow` account with the offer terms.
2. CPI `transfer_checked` — move `deposit` of `mint_a` from the maker's ATA into the vault.

### `take()`

Accepts an open escrow offer.

**Accounts:** taker (signer), maker, mint_a, mint_b, taker_ata_a, taker_ata_b, maker_ata_b, escrow, vault, token_program, associated_token_program, system_program

**Logic:**
1. CPI `transfer_checked` — taker sends `receive` of `mint_b` to the maker.
2. CPI `transfer_checked` — escrow PDA sends `mint_a` from the vault to the taker.
3. CPI `close_account` — close the vault; rent returned to the maker.
4. Close the `Escrow` account (Anchor `close = maker` constraint); rent returned to the maker.

The taker pays for creating any ATAs that do not yet exist (`init_if_needed`).

### `refund(seed)`

Cancels an escrow and returns the deposit to the maker.

**Accounts:** maker (signer), mint_a, maker_ata_a, escrow, vault, token_program, system_program

**Logic:**
1. CPI `transfer_checked` — return the full vault balance of `mint_a` to the maker.
2. CPI `close_account` — close the vault.
3. Close the `Escrow` account.

Only the original maker can call this instruction.

---

## Project Structure

```
escrow/
├── Anchor.toml                 # Anchor workspace config (cluster, program ID, scripts)
├── Cargo.toml                  # Rust workspace root
├── rust-toolchain.toml         # Pinned Rust 1.89.0
├── package.json                # JS tooling (Prettier, Anchor TS types)
├── migrations/
│   └── deploy.ts               # Anchor deploy hook (placeholder)
├── programs/
│   └── escrow/
│       ├── Cargo.toml          # Program crate + LiteSVM dev-dependencies
│       ├── src/
│       │   ├── lib.rs          # Program entrypoint and instruction dispatch
│       │   ├── state.rs        # Escrow account struct
│       │   ├── constants.rs    # PDA seed prefix ("escrow")
│       │   ├── error.rs        # Custom error codes
│       │   └── instructions/
│       │       ├── make.rs     # Create escrow + deposit
│       │       ├── take.rs     # Accept offer + settle
│       │       └── refund.rs   # Cancel + return deposit
│       └── tests/
│           └── test_make.rs    # LiteSVM integration test (make + refund)
└── target/
    ├── deploy/escrow.so        # Compiled program binary (after build)
    ├── idl/escrow.json         # Generated IDL
    └── types/escrow.ts         # Generated TypeScript client types
```

The `app/` directory is reserved for a client frontend but is currently empty.

---

## Prerequisites

| Tool | Version (tested) |
|------|------------------|
| [Rust](https://rustup.rs/) | 1.89.0 (see `rust-toolchain.toml`) |
| [Solana CLI](https://docs.anza.xyz/cli/install) | 3.1.x |
| [Anchor CLI](https://www.anchor-lang.com/docs/installation) | 1.0.2 |
| [Yarn](https://yarnpkg.com/) | 1.x |

Install Anchor 1.0.2 with avm if needed:

```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 1.0.2
avm use 1.0.2
```

---

## Build

Compile the program and generate the IDL:

```bash
anchor build
```

This produces:
- `target/deploy/escrow.so` — deployable program binary
- `target/idl/escrow.json` — interface definition
- `target/types/escrow.ts` — TypeScript types for clients

---

## Test

Tests run in-process with [LiteSVM](https://github.com/LiteSVM/litesvm) — no local validator required.

```bash
anchor test
# equivalent to:
cargo test
```

The integration test in `programs/escrow/tests/test_make.rs` covers the full **make → refund** flow:

1. Spin up a LiteSVM instance and load the compiled program.
2. Create two mints and mint tokens to the maker.
3. Call `make` — verify vault balance and escrow state.
4. Call `refund` — verify escrow and vault accounts are closed.

> **Note:** The `take` instruction is implemented but does not yet have an integration test.

---

## Deploy

1. Configure your target cluster and wallet in `Anchor.toml`:

```toml
[provider]
cluster = "devnet"          # or "localnet", "mainnet-beta"
wallet = "~/.config/solana/id.json"
```

2. Build and deploy:

```bash
anchor deploy
```

3. Optionally run the migration script (currently a no-op placeholder):

```bash
anchor migrate
```

For local development with a validator:

```bash
solana-test-validator          # in one terminal
anchor deploy                  # in another
```

---

## Key Concepts Demonstrated

- **PDAs** — Escrow state and vault authority are derived addresses, letting the program sign token transfers without a private key.
- **Associated Token Accounts (ATAs)** — Standard SPL token account layout for vault and user wallets.
- **CPIs** — Token transfers and account closes are delegated to the SPL Token program via cross-program invocations.
- **Account constraints** — Anchor `has_one`, `seeds`, `bump`, and `close` attributes enforce that only valid parties can interact with an escrow.
- **Token-2022 compatibility** — Uses `token_interface` and `InterfaceAccount` so the program works with both legacy SPL Token and Token-2022 mints.
- **LiteSVM testing** — Fast, in-process tests without spinning up `solana-test-validator`.

---

## Dependencies

**On-chain (Rust):**

| Crate | Version | Purpose |
|-------|---------|---------|
| `anchor-lang` | 1.0.2 | Program framework, account macros, CPI helpers |
| `anchor-spl` | 1.0.2 | SPL Token / ATA program bindings |

**Test (Rust dev-dependencies):**

| Crate | Version | Purpose |
|-------|---------|---------|
| `litesvm` | 0.12.0 | In-process Solana VM for tests |
| `litesvm-token` | 0.12.0 | Token helpers for LiteSVM |
| `solana-keypair`, `solana-transaction`, etc. | 3.x | Transaction building in tests |

**Client tooling (Node):**

| Package | Purpose |
|---------|---------|
| `@anchor-lang/core` | TypeScript client SDK |
| `typescript`, `prettier` | Linting and formatting |

---

## License

ISC
