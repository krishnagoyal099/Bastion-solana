# BASTION PROTOCOL — CASPER → SOLANA FULL MIGRATION BLUEPRINT

## Principal Architect's Migration Document & Production Redesign Specification

---

# 1. FULL CASPER → SOLANA MAPPING TABLE

## 1.1 Core Blockchain Primitives

| Casper Concept | Solana Equivalent | Migration Complexity | Redesign Notes |
|---|---|---|---|
| Contract (WASM) | Program (BPF/SBF) | **HIGH** | Entire compilation target changes. Anchor framework recommended over native for maintainability, CPI ergonomics, and account discriminator safety. |
| ContractHash | Program ID (Pubkey) | **LOW** | Direct mapping. Program ID is the on-chain address. |
| URef (Unforgeable Reference) | PDA-derived Account | **CRITICAL** | URefs provide capability-based security with read/write access control. Solana has no equivalent. Must be reimplemented via PDA ownership + program-level access control. |
| Dictionary (key-value) | PDA Account per entry OR Account with HashMap | **CRITICAL** | Casper dictionaries are gas-efficient sparse maps. On Solana, each dictionary entry could be a separate PDA account (flexible but rent-costly) or a field in a packed account (cheap but bounded). Choice depends on access pattern. For balances: PDA per user. For nullifiers: PDA per nullifier (must check uniqueness). |
| RuntimeArgs | Instruction Data (serialized) | **MEDIUM** | Anchor provides `#[derive(AnchorSerialize)]` for typed instruction args. Manual Borsh otherwise. |
| EntryPoint | Instruction Handler | **LOW** | Anchor `#[program]` functions map 1:1 to Casper entry points. |
| NamedKey | PDA seed / Account discriminant | **HIGH** | Named keys allow dynamic named storage. Solana uses deterministic PDA derivation from seeds. Must design seed schemas carefully. |
| Contract Package (versioning) | Program Buffer + Upgrade Authority | **MEDIUM** | Solana's upgrade model is different: buffer account + upgrade authority. No built-in version tracking. Must add version field to config account. |
| Payment (caller pays) | Transaction fees + compute budget | **MEDIUM** | Casper: caller specifies payment amount. Solana: base fee + priority fee + compute budget. Must redesign gas estimation. |
| Block time ~32s | Slot time ~400ms | **LOW** (beneficial) | Faster finality, but changes UX patterns (polling intervals, timeout logic). |
| Event Standard (CES) | Solana `msg!()` / `log!()` + program log parsing | **HIGH** | Casper has structured event standard with schemas. Solana has raw log emission. Must build event parsing layer. Anchor emits discriminator-prefixed logs. |
| Transfer (native CSPR) | SOL transfer via System Program | **LOW** | Different instruction but conceptually identical. |
| Account model (global state) | Account model (owned accounts) | **CRITICAL** | Casper: single global state, any contract can read any key. Solana: accounts are owned by programs, rent-exempt, must be passed explicitly. Fundamentally different access patterns. |

## 1.2 Token System Mapping

| Casper CEP-18 | Solana SPL Token / Token-2022 | Complexity | Notes |
|---|---|---|---|
| `balances` dictionary | Associated Token Account (ATA) balance | **HIGH** | CEP-18: contract stores balances in its own dictionary. SPL: each user has a separate token account. Architecture is inverted. |
| `allowances` dictionary | SPL Token `approve` / delegate | **MEDIUM** | CEP-18 allowances are per-owner-per-spender. SPL uses `approve` with delegate + amount. Different revocation model. |
| `mint` entrypoint | Mint authority instruction | **LOW** | Direct mapping. Mint authority must be program PDA or signed. |
| `burn` entrypoint | Burn instruction | **LOW** | Direct mapping via SPL Token `burn`. |
| `transfer` entrypoint | SPL Token `transfer` | **LOW** | Via CPI to SPL Token program. |
| `transfer_from` entrypoint | Delegate + Transfer | **MEDIUM** | Must first `approve` delegate, then delegate can `transfer`. Two-step vs one-step. |
| `total_supply` URef | Mint account `supply` field | **LOW** | Read directly from Mint account. |
| `name`, `symbol`, `decimals` URefs | Mint account + Metaplex metadata | **MEDIUM** | SPL Token Mint stores `decimals`. Name/symbol require Metaplex Token Metadata. |
| Security badges (admin/minter/none) | Mint authority / Freeze authority / Multisig | **MEDIUM** | Must redesign role management. Anchor access control or custom PDA-based role accounts. |
| Events (Mint, Burn, Transfer, etc.) | Program logs + indexer | **HIGH** | No native structured events. Must emit via `msg!()` and parse with indexer. |

## 1.3 AMM System Mapping

| Casper AMM Concept | Solana Equivalent | Complexity | Notes |
|---|---|---|---|
| `reserve_a` URef | PDA vault token account (Token A) | **MEDIUM** | Vault is a PDA-owned SPL token account. Reserves = vault balance. |
| `reserve_b` URef | PDA vault token account (Token B) | **MEDIUM** | Same as above. |
| LP shares (dictionary) | LP Mint + LP token accounts | **HIGH** | CEP-18 uses dictionary for LP shares. Solana: mint an actual SPL token for LP. |
| `x * y = k` formula | Same formula in instruction handler | **LOW** | Math is portable. Must handle U256→u64/u128 precision carefully. |
| 0.3% fee (997/1000) | Same in instruction logic | **LOW** | Direct port. |
| `min_amount_out` slippage check | Same pattern via instruction arg | **LOW** | Direct port. |
| `get_reserves()` | Read vault balances | **MEDIUM** | On Casper: free read. On Solana: client-side read via RPC. Vault balance IS the reserve. |
| Swap events | Program logs | **MEDIUM** | Must design event schema. |
| First LP lockup (MIN_LIQUIDITY) | Same concept | **LOW** | Mint MIN_LIQUIDITY LP tokens to pool PDA (permanently locked). |

## 1.4 Dark Pool System Mapping

| Casper Dark Pool Concept | Solana Equivalent | Complexity | Notes |
|---|---|---|---|
| `balances` dictionary | PDA account per user | **HIGH** | Each user gets a PDA: `seeds=[b"balance", user_pubkey.as_ref()]` |
| `nullifiers` dictionary | PDA account per nullifier | **CRITICAL** | Nullifier uniqueness MUST be enforced. PDA with nullifier hash as seed ensures on-chain uniqueness (derivation collision = impossible). This is the core privacy mechanism. |
| `commitment` (Vec<u8>) | Part of Order account data | **MEDIUM** | Stored in order PDA account. |
| `proof` (Vec<u8>) | Off-chain verified, hash committed | **CRITICAL** | Full proof verification on Solana costs 200K-500K CU. Must evaluate if verification fits in compute budget. See §5. |
| `total_orders` URef | Config account field | **LOW** | Simple counter in a PDA config account. |
| `deposit_cspr` | SOL wrap + deposit instruction | **MEDIUM** | Casper: native token deposit. Solana: must wrap SOL → WSOL then deposit via CPI. |
| `submit_order` | Submit commitment instruction | **HIGH** | Same business logic but account model forces different data layout. |
| Hidden order details | Encrypted commitment only | **CRITICAL** | Solana transactions are public. Order details MUST be encrypted off-chain. Only commitment/hash goes on-chain. |

## 1.5 CLI / Infrastructure Mapping

| Casper CLI Concept | Solana Equivalent | Complexity | Notes |
|---|---|---|---|
| `casper-client` | `solana-cli` + Anchor CLI | **LOW** | Direct tool replacement. |
| PEM key files | Filesystem wallet (JSON keypair) | **LOW** | Solana uses base58-encoded keypairs. Must support both. |
| CSPR.cloud REST API | Solana RPC + Helius/Triton/SolanaFM | **MEDIUM** | Different API structure, similar capability. |
| RPC JSON-RPC | JSON-RPC 2.0 (Solana) | **LOW** | Same protocol, different methods. |
| SSE event stream | WebSocket subscriptions | **LOW** | Solana has native WebSocket for account/log subscriptions. |
| `gum` TUI | Same, or `ink` (Rust TUI) | **LOW** | UI framework is independent of chain. |
| `jq` JSON processing | Same or `jsquery` | **LOW** | Unchanged. |

---

# 2. SOLANA TARGET ARCHITECTURE

## 2.1 Program Boundary Design

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        BASTION SOLANA PROTOCOL                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐         │
│  │  bastion-pool     │  │  bastion-amm      │  │  bastion-govern  │         │
│  │  (Dark Pool)      │  │  (StorySwap AMM)  │  │  (Config/Auth)   │         │
│  │                   │  │                    │  │                   │         │
│  │  - deposit_sol    │  │  - initialize      │  │  - set_config     │         │
│  │  - deposit_token  │  │  - add_liquidity   │  │  - set_authority  │         │
│  │  - submit_commit  │  │  - remove_liquidity│  │  - pause          │         │
│  │  - reveal_order   │  │  - swap_a_to_b     │  │  - upgrade        │         │
│  │  - match_settle   │  │  - swap_b_to_a     │  │                   │         │
│  │  - withdraw       │  │  - quote_swap      │  │                   │         │
│  │  - nullify        │  │                    │  │                   │         │
│  │  - verify_proof   │  │                    │  │                   │         │
│  └────────┬─────────┘  └────────┬──────────┘  └────────┬─────────┘         │
│           │                      │                       │                   │
│           └──────────┬───────────┘                       │                   │
│                      │                                   │                   │
│              ┌───────▼────────┐                          │                   │
│              │  SPL Token      │◄─────────────────────────┘                   │
│              │  Token-2022     │                                              │
│              │  System Program │                                              │
│              │  Associated Tok │                                              │
│              └────────────────┘                                              │
│                                                                            │
│  ┌──────────────────┐  ┌──────────────────┐                                 │
│  │  Off-chain       │  │  Relayer /       │                                 │
│  │  Prover (ZK)     │  │  Match Engine    │                                 │
│  │                  │  │                  │                                 │
│  │  - Generate      │  │  - Collect       │                                 │
│  │    commitments   │  │    commitments   │                                 │
│  │  - Create ZK     │  │  - Match orders │                                 │
│  │    proofs        │  │  - Batch settle │                                 │
│  │  - Sign          │  │  - Jito bundles │                                 │
│  │    revelations   │  │                  │                                 │
│  └──────────────────┘  └──────────────────┘                                 │
│                                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 2.2 Account Graph

```
Global Config PDA (seeds: ["config"])
├── program_authority: Pubkey
├── paused: bool
├── total_orders: u64
├── total_volume: u64
├── amm_program: Pubkey
├── treasury: Pubkey
├── fee_bps: u16
└── version: u32

User Deposit PDA (seeds: ["deposit", user_pubkey])
├── owner: Pubkey
├── sol_amount: u64
├── token_amount: u64
├── pending_orders: u8
└── bump: u8

Order Commitment PDA (seeds: ["order", commitment_hash])
├── commitment: [u8; 32]
├── nullifier_hash: [u8; 32]
├── submitter: Pubkey
├── amount_hash: [u8; 32]        // hash of actual amount
├── side_hash: [u8; 32]          // hash of actual side
├── price_hash: [u8; 32]         // hash of actual price
├── status: OrderStatus          // Committed/Revealed/Matched/Settled/Cancelled
├── created_at: i64
├── proof_valid: bool
└── bump: u8

Nullifier PDA (seeds: ["nullifier", nullifier_hash])
├── used: bool
├── order_commitment: [u8; 32]
├── used_at: i64
└── bump: u8

AMM Pool PDA (seeds: ["pool", mint_a, mint_b])
├── mint_a: Pubkey
├── mint_b: Pubkey
├── vault_a: Pubkey              // ATA owned by pool PDA
├── vault_b: Pubkey              // ATA owned by pool PDA
├── lp_mint: Pubkey
├── fee_bps: u16
├── min_liquidity: u64
├── total_lp_supply: u64
└── bump: u8

LP Position PDA (seeds: ["lp", pool, owner])
├── pool: Pubkey
├── owner: Pubkey
├── lp_shares: u64
├── deposited_at: i64
└── bump: u8

Escrow Vault PDA (seeds: ["escrow", order_commitment])
├── order: Pubkey
├── token_mint: Pubkey
├── amount: u64
└── bump: u8

Event Queue PDA (seeds: ["events", config])
├── head: u32
├── events: [Event; MAX_EVENTS]  // Ring buffer
└── bump: u8
```

## 2.3 Authority Hierarchy

```
Upgrade Authority (Multisig/Squads)
    │
    ▼
Program Deploy
    │
    ├── Config PDA (program_id is owner)
    │       │
    │       ├── program_authority (signer for privileged ops)
    │       └── Sets: treasury, fees, paused state
    │
    ├── Pool PDA (program_id is owner)
    │       │
    │       ├── Owns vault_a (SPL Token account)
    │       ├── Owns vault_b (SPL Token account)
    │       └── Is mint authority for lp_mint
    │
    ├── Deposit PDA (program_id is owner)
    │       └── Only user can withdraw their deposit
    │
    ├── Order PDA (program_id is owner)
    │       └── Only submitter can cancel/reveal
    │
    └── Nullifier PDA (program_id is owner)
            └── Write-once (set to used, never unset)
```

## 2.4 Data Flow — Order Submission

```
User (off-chain)                          On-chain                              Off-chain Match Engine
    │                                         │                                         │
    │  1. Generate commitment                 │                                         │
    │     commitment = hash(amount, side,     │                                         │
    │                        price, nonce)    │                                         │
    │                                         │                                         │
    │  2. Generate nullifier                  │                                         │
    │     nullifier = hash(secret, nonce)     │                                         │
    │                                         │                                         │
    │  3. Generate ZK proof                   │                                         │
    │     proof = prove(commitment,           │                                         │
    │              nullifier, valid_range)     │                                         │
    │                                         │                                         │
    │  ─── submit_commit ─────────────────►   │                                         │
    │     [commitment, nullifier,             │                                         │
    │      proof_hash, deposit_vault]         │                                         │
    │                                         │                                         │
    │                                   4. Verify nullifier unused              │
    │                                      (PDA derivation succeeds = unique)    │
    │                                         │                                         │
    │                                   5. Create Order PDA                     │
    │                                      seeds: ["order", commitment]          │
    │                                         │                                         │
    │                                   6. Create Nullifier PDA                 │
    │                                      seeds: ["nullifier", nullifier_hash]  │
    │                                         │                                         │
    │                                   7. Lock deposit in escrow               │
    │                                         │                                         │
    │                                   8. Emit event: OrderCommitted           │
    │                                      ──────────────────────────────────►   │
    │                                                                         │
    │                                                          9. Collect commitment
    │                                                              from event
    │                                                                         │
    │                                                          10. Match orders
    │                                                              (complementary
    │                                                               commitments)
    │                                                                         │
    │                                              ◄── match_settle ─────────│
    │                                              [order_a, order_b,       │
    │                                               match_proof]            │
    │                                                                         │
    │                                   11. Verify both orders valid           │
    │                                   12. Execute swap via AMM CPI           │
    │                                   13. Settle both orders                 │
    │                                   14. Release escrowed funds             │
    │                                   15. Emit: OrdersMatched                │
    │                                         │                                │
    │  ◄── settlement confirmation ───────────│                                │
```

## 2.5 Data Flow — Swap Execution

```
User (off-chain)                    On-chain
    │                                   │
    │  ─── swap_a_to_b ──────────────► │
    │     [amount_in, min_amount_out]   │
    │                                   │
    │                             1. Verify pool account
    │                             2. Read vault_a balance (reserve_a)
    │                             3. Read vault_b balance (reserve_b)
    │                             4. Calculate:
    │                                amount_in_with_fee = amount_in * 997
    │                                numerator = amount_in_with_fee * reserve_b
    │                                denominator = reserve_a * 1000 + amount_in_with_fee
    │                                amount_out = numerator / denominator
    │                             5. Check amount_out >= min_amount_out
    │                             6. CPI: transfer A from user ATA → vault_a
    │                             7. CPI: transfer B from vault_b → user ATA
    │                             8. Emit: SwapExecuted
    │                                   │
    │  ◄── amount_out ─────────────── │
```

---

# 3. ACCOUNT MODEL DESIGN

## 3.1 Config Account

```rust
// PDA seeds: ["config"]
// Size: ~200 bytes (fits in one account easily)

#[account]
pub struct BastionConfig {
    pub authority: Pubkey,           // 32  - Upgrade/multisig authority
    pub amm_program: Pubkey,         // 32  - AMM program ID
    pub treasury: Pubkey,            // 32  - Fee collection wallet
    pub token_mint: Pubkey,          // 32  - Wrapped SOL or stablecoin mint
    pub wsol_mint: Pubkey,           // 32  - WSOL mint (So11111111111111111111111111111111111111112)
    pub paused: bool,                // 1
    pub fee_bps: u16,               // 2   - Fee in basis points
    pub min_order_size: u64,         // 8   - Minimum order in lamports
    pub max_order_size: u64,         // 8
    pub total_orders: u64,           // 8
    pub total_volume: u64,           // 8   - Cumulative volume in lamports
    pub version: u32,                // 4
    pub bump: u8,                    // 1
    pub _reserved: [u8; 32],         // 32  - Future expansion
}
// Total: ~232 bytes
```

## 3.2 User Deposit Account

```rust
// PDA seeds: ["deposit", user_pubkey.as_ref()]
// Size: ~100 bytes

#[account]
pub struct UserDeposit {
    pub owner: Pubkey,               // 32
    pub sol_deposited: u64,          // 8   - SOL deposited in lamports
    pub token_deposited: u64,        // 8   - Token deposited in smallest unit
    pub pending_orders: u8,          // 1   - Count of active orders
    pub total_orders: u32,           // 4   - Lifetime order count
    pub created_at: i64,             // 8   - Unix timestamp
    pub bump: u8,                    // 1
    pub _reserved: [u8; 16],         // 16
}
// Total: ~110 bytes
```

## 3.3 Order Commitment Account

```rust
// PDA seeds: ["order", commitment_hash.as_ref()]
// Size: ~220 bytes

#[account]
pub struct OrderCommitment {
    pub commitment: [u8; 32],        // 32  - hash(amount, side, price, nonce)
    pub nullifier_hash: [u8; 32],    // 32  - hash(secret, nonce)
    pub submitter: Pubkey,            // 32  - Who submitted (can be relayer)
    pub beneficiary: Pubkey,          // 32  - Actual trader (may differ from submitter for privacy)
    pub escrow_vault: Pubkey,         // 32  - Token account holding locked funds
    pub amount_commitment: [u8; 32],  // 32  - Pedersen commitment to amount
    pub side_commitment: [u8; 32],    // 32  - Pedersen commitment to side
    pub price_commitment: [u8; 32],   // 32  - Pedersen commitment to limit price
    pub status: OrderStatus,          // 1   - Enum
    pub deposit_mint: Pubkey,         // 32  - Which token was deposited
    pub deposit_amount: u64,          // 8   - Amount locked (revealed after match)
    pub created_at: i64,              // 8
    pub settled_at: i64,              // 8
    pub proof_hash: [u8; 32],        // 32  - Hash of the ZK proof (not stored on-chain for size)
    pub bump: u8,                     // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Committed,    // Order submitted, awaiting match
    Revealed,     // Order details revealed for matching
    Matched,      // Match found, awaiting settlement
    Settled,      // Successfully settled
    Cancelled,    // Cancelled by user
    Expired,      // Order timed out
}
// Total: ~338 bytes
```

## 3.4 Nullifier Account

```rust
// PDA seeds: ["nullifier", nullifier_hash.as_ref()]
// Size: ~80 bytes
// WRITE-ONCE: created when order is committed, never modified

#[account]
pub struct NullifierRecord {
    pub nullifier_hash: [u8; 32],    // 32  - The nullifier
    pub order_commitment: [u8; 32],   // 32  - Back-reference to order
    pub used_at: i64,                 // 8   - When it was used
    pub bump: u8,                     // 1
}
// Total: ~73 bytes
```

**Critical design decision**: Nullifiers are PDA-derived from the nullifier hash. This means:

- If someone tries to submit the same nullifier twice, the PDA derivation will produce the same address, and `create_account` will fail because the account already exists.
- This is the **exact** on-chain uniqueness guarantee we need, without requiring a linear scan of any data structure.
- Rent cost: ~0.0007 SOL per nullifier (minimal, one-time).

## 3.5 AMM Pool Account

```rust
// PDA seeds: ["pool", mint_a.as_ref(), mint_b.as_ref()]
// Size: ~200 bytes

#[account]
pub struct AmmPool {
    pub mint_a: Pubkey,               // 32
    pub mint_b: Pubkey,               // 32
    pub vault_a: Pubkey,              // 32  - SPL Token account for token A reserves
    pub vault_b: Pubkey,              // 32  - SPL Token account for token B reserves
    pub lp_mint: Pubkey,              // 32  - LP token mint
    pub fee_bps: u16,                 // 2   - Trading fee (30 = 0.3%)
    pub min_liquidity: u64,           // 8   - Permanently locked LP tokens
    pub total_swaps: u64,             // 8
    pub created_at: i64,              // 8
    pub bump: u8,                     // 1
    pub _reserved: [u8; 23],          // 23  - Expansion
}
// Total: ~202 bytes
```

## 3.6 LP Position Account

```rust
// PDA seeds: ["lp", pool.as_ref(), owner.as_ref()]
// Size: ~90 bytes

#[account]
pub struct LpPosition {
    pub pool: Pubkey,                  // 32
    pub owner: Pubkey,                 // 32
    pub lp_shares: u64,                // 8
    pub deposited_at: i64,             // 8
    pub bump: u8,                      // 1
}
// Total: ~81 bytes
```

## 3.7 Escrow Vault Account

This is a standard SPL Token Associated Token Account, owned by the PDA `["escrow", order_commitment.as_ref()]`. The PDA acts as the authority, allowing the program to CPI-transfer tokens out only when the order is settled or cancelled.

## 3.8 Account Size Summary

| Account | Size | Rent (SOL) | Count | Total Rent Impact |
|---|---|---|---|---|
| Config | 232 B | ~0.002 | 1 | Negligible |
| UserDeposit | 110 B | ~0.0015 | Per user | ~0.0015 per user |
| OrderCommitment | 338 B | ~0.0023 | Per order | ~0.0023 per order |
| NullifierRecord | 73 B | ~0.0007 | Per order | ~0.0007 per order |
| AmmPool | 202 B | ~0.0019 | Per pool | Negligible |
| LpPosition | 81 B | ~0.0008 | Per LP per pool | ~0.0008 per LP position |

**Critical observation**: Each order creates 2 accounts (OrderCommitment + NullifierRecord) = ~0.003 SOL in rent. For a dark pool processing 10,000 orders/day, that's ~30 SOL/day in rent. This is manageable but must be factored into the economics. The user who submits the order pays this rent.

---

# 4. INSTRUCTION DESIGN

## 4.1 Dark Pool Program — Instructions

### 4.1.1 `initialize`

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = BastionConfig::DISCRIMINATOR.len() + std::mem::size_of::<BastionConfig>(),
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, BastionConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Sets: authority, fee_bps=30, min_order_size, token_mint, paused=false
```

### 4.1.2 `deposit_sol`

```rust
#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = UserDeposit::DISCRIMINATOR.len() + std::mem::size_of::<UserDeposit>(),
        seeds = [b"deposit", user.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// CHECK: WSOL account handling
    pub wsol_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

// Flow:
// 1. User wraps SOL → WSOL via System Program + Token Program
// 2. Transfer WSOL to program-controlled vault
// 3. Update UserDeposit.sol_deposited += amount
// 4. Emit: DepositEvent { user, amount, mint }
```

### 4.1.3 `deposit_token`

```rust
#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(
        mut,
        seeds = [b"deposit", user.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Program-controlled vault
    #[account(mut)]
    pub program_vault: Account<'info, TokenAccount>,
    
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

// Flow:
// 1. CPI: transfer tokens from user ATA → program vault
// 2. Update UserDeposit.token_deposited += amount
```

### 4.1.4 `submit_commitment`

```rust
#[derive(Accounts)]
pub struct SubmitCommitment<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,
    
    #[account(
        mut,
        seeds = [b"deposit", beneficiary.key().as_ref()],
        bump,
        constraint = user_deposit.sol_deposited >= min_deposit @ ErrorCode::InsufficientDeposit
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    
    #[account(
        init,
        payer = submitter,
        space = OrderCommitment::DISCRIMINATOR.len() + std::mem::size_of::<OrderCommitment>(),
        seeds = [b"order", commitment.as_ref()],
        bump
    )]
    pub order: Account<'info, OrderCommitment>,
    
    #[account(
        init,
        payer = submitter,
        space = NullifierRecord::DISCRIMINATOR.len() + std::mem::size_of::<NullifierRecord>(),
        seeds = [b"nullifier", nullifier_hash.as_ref()],
        bump
    )]
    pub nullifier: Account<'info, NullifierRecord>,
    
    /// CHECK: Can be different from beneficiary for privacy
    pub beneficiary: AccountInfo<'info>,
    
    #[account(mut)]
    pub submitter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Args:
//   commitment: [u8; 32]    - hash(amount, side, price, nonce)
//   nullifier_hash: [u8; 32] - hash(secret, nonce)
//   amount_commitment: [u8; 32] - Pedersen commitment
//   side_commitment: [u8; 32]
//   price_commitment: [u8; 32]
//   proof_hash: [u8; 32]    - Hash of off-chain ZK proof (for verification later)
//   deposit_mint: Pubkey     - Which token
//   deposit_amount: u64      - How much to lock

// Flow:
// 1. Verify !config.paused
// 2. Derive order PDA from commitment → ensure account doesn't exist (uniqueness)
// 3. Derive nullifier PDA from nullifier_hash → ensure account doesn't exist (anti-replay)
// 4. Lock deposit_amount from user_deposit into escrow
// 5. Create order account with status=Committed
// 6. Create nullifier account (write-once)
// 7. Emit: OrderCommitted { commitment, nullifier_hash, deposit_mint, timestamp }
```

### 4.1.5 `reveal_order`

```rust
#[derive(Accounts)]
pub struct RevealOrder<'info> {
    #[account(
        mut,
        seeds = [b"order", order.commitment.as_ref()],
        bump,
        constraint = order.status == OrderStatus::Committed @ ErrorCode::InvalidOrderStatus
    )]
    pub order: Account<'info, OrderCommitment>,
    
    pub beneficiary: Signer<'info>,  // Only beneficiary can reveal
    
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,
}

// Args:
//   amount: u64
//   side: OrderSide  (Buy/Sell)
//   price: u64       (limit price in lamports)
//   nonce: [u8; 32]

// Flow:
// 1. Verify beneficiary signature
// 2. Verify: commitment == hash(amount, side, price, nonce)
// 3. Verify: amount within [config.min_order_size, config.max_order_size]
// 4. Update order.status = Revealed
// 5. Emit: OrderRevealed { commitment, amount, side, price }
```

**Important privacy note**: On Solana, the reveal transaction is visible to all. In a true dark pool, the order should never need to be revealed on-chain. Instead, the matching engine should verify commitments off-chain and only publish settlement results. See §5 for the full privacy architecture.

### 4.1.6 `match_settle`

```rust
#[derive(Accounts)]
pub struct MatchSettle<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,
    
    #[account(
        mut,
        seeds = [b"order", order_a.commitment.as_ref()],
        bump,
        constraint = order_a.status == OrderStatus::Committed || order_a.status == OrderStatus::Revealed
    )]
    pub order_a: Account<'info, OrderCommitment>,
    
    #[account(
        mut,
        seeds = [b"order", order_b.commitment.as_ref()],
        bump,
        constraint = order_b.status == OrderStatus::Committed || order_b.status == OrderStatus::Revealed
    )]
    pub order_b: Account<'info, OrderCommitment>,
    
    #[account(mut)]
    pub escrow_a: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub escrow_b: Account<'info, TokenAccount>,
    
    /// CHECK: Beneficiary A token account
    #[account(mut)]
    pub beneficiary_a_ata: AccountInfo<'info>,
    
    /// CHECK: Beneficiary B token account
    #[account(mut)]
    pub beneficiary_b_ata: AccountInfo<'info>,
    
    /// Optional: CPI to AMM for swap execution
    pub amm_program: Option<Program<'info, BastionAmm>>,
    pub amm_pool: Option<Account<'info, AmmPool>>,
    
    pub token_program: Program<'info, Token>,
    
    pub match_authority: Signer<'info>,  // Authorized match engine
}

// Args:
//   match_proof: Vec<u8>   - ZK proof that orders are compatible
//   execution_price: u64   - Settlement price

// Flow:
// 1. Verify match_authority is authorized (config.match_authority)
// 2. Verify orders are compatible (opposite sides, overlapping price ranges)
//    - If using ZK: verify match_proof
//    - If using revealed orders: verify order_a.side != order_b.side
// 3. Execute swap via AMM CPI if needed
// 4. Transfer tokens: escrow_a → beneficiary_b_ata, escrow_b → beneficiary_a_ata
// 5. Update both orders: status = Settled, settled_at = now
// 6. Update config: total_orders += 2, total_volume += executed_volume
// 7. Emit: OrdersMatched { order_a, order_b, execution_price }
```

### 4.1.7 `cancel_order`

```rust
#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(
        mut,
        seeds = [b"order", order.commitment.as_ref()],
        bump,
        constraint = order.status == OrderStatus::Committed @ ErrorCode::InvalidOrderStatus,
        constraint = order.beneficiary == beneficiary.key() @ ErrorCode::Unauthorized
    )]
    pub order: Account<'info, OrderCommitment>,
    
    #[account(
        mut,
        seeds = [b"deposit", beneficiary.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    
    #[account(mut)]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Flow:
// 1. Verify beneficiary signature
// 2. Transfer tokens from escrow back to user_deposit
// 3. Update order.status = Cancelled
// 4. Update user_deposit: restore deposit amount
```

### 4.1.8 `withdraw`

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"deposit", user.key().as_ref()],
        bump,
        constraint = user_deposit.pending_orders == 0 @ ErrorCode::PendingOrdersExist
    )]
    pub user_deposit: Account<'info, UserDeposit>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub program_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

// Args:
//   amount: u64
//   mint: Pubkey  (SOL or token)

// Flow:
// 1. Verify user has sufficient balance in deposit
// 2. Verify no pending orders
// 3. CPI transfer from program vault → user ATA
// 4. Update deposit balance
// 5. Emit: Withdrawal { user, amount, mint }
```

## 4.2 AMM Program — Instructions

### 4.2.1 `initialize_pool`

```rust
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = AmmPool::DISCRIMINATOR.len() + std::mem::size_of::<AmmPool>(),
        seeds = [b"pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, AmmPool>,
    
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"vault_a", pool.key().as_ref()],
        bump,
        token::mint = mint_a,
        token::authority = pool
    )]
    pub vault_a: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        seeds = [b"vault_b", pool.key().as_ref()],
        bump,
        token::mint = mint_b,
        token::authority = pool
    )]
    pub vault_b: Account<'info, TokenAccount>,
    
    // LP token mint (program is mint authority)
    #[account(
        init,
        payer = authority,
        seeds = [b"lp_mint", pool.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = pool
    )]
    pub lp_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
```

### 4.2.2 `add_liquidity`

```rust
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump
    )]
    pub pool: Account<'info, AmmPool>,
    
    #[account(
        init_if_needed,
        payer = provider,
        space = LpPosition::DISCRIMINATOR.len() + std::mem::size_of::<LpPosition>(),
        seeds = [b"lp", pool.key().as_ref(), provider.key().as_ref()],
        bump
    )]
    pub lp_position: Account<'info, LpPosition>,
    
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub provider_lp_ata: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub provider_token_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub provider_token_b_ata: Account<'info, TokenAccount>,
    
    pub provider: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// Args:
//   amount_a: u64
//   amount_b: u64
//   min_lp_shares: u64  (slippage protection)

// LP share calculation:
//   If first LP: shares = sqrt(amount_a * amount_b) - MIN_LIQUIDITY
//   Else: shares = min(
//     (amount_a * total_supply) / reserve_a,
//     (amount_b * total_supply) / reserve_b
//   )
//   MIN_LIQUIDITY (1000) minted to pool PDA (permanently locked)

// Compute budget: ~50K CU (2 CPI transfers + 1 mint)
```

### 4.2.3 `swap_a_to_b` / `swap_b_to_a`

```rust
#[derive(Accounts)]
pub struct SwapAToB<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump
    )]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub trader_token_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub trader_token_b_ata: Account<'info, TokenAccount>,
    
    pub trader: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Args:
//   amount_in: u64
//   min_amount_out: u64  (slippage protection)

// Calculation (constant product with 0.3% fee):
//   reserve_a = vault_a.amount
//   reserve_b = vault_b.amount
//   amount_in_with_fee = amount_in * 997
//   numerator = amount_in_with_fee * reserve_b
//   denominator = (reserve_a * 1000) + amount_in_with_fee
//   amount_out = numerator / denominator
//   
//   require!(amount_out >= min_amount_out, SlippageExceeded);
//
// CPI calls:
//   1. transfer(trader_ata_a → vault_a, amount_in)
//   2. transfer(vault_b → trader_ata_b, amount_out)
//
// Compute budget: ~30K CU (2 CPI transfers)
```

### 4.2.4 `remove_liquidity`

```rust
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump
    )]
    pub pool: Account<'info, AmmPool>,
    
    #[account(
        mut,
        seeds = [b"lp", pool.key().as_ref(), provider.key().as_ref()],
        bump,
        constraint = lp_position.lp_shares >= shares_to_burn @ ErrorCode::InsufficientShares
    )]
    pub lp_position: Account<'info, LpPosition>,
    
    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub provider_lp_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub provider_token_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub provider_token_b_ata: Account<'info, TokenAccount>,
    
    pub provider: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Args:
//   shares_to_burn: u64
//   min_amount_a: u64  (slippage protection)
//   min_amount_b: u64

// Calculation:
//   amount_a = (shares * reserve_a) / total_supply
//   amount_b = (shares * reserve_b) / total_supply
//   require!(amount_a >= min_amount_a && amount_b >= min_amount_b)
//   
// CPI calls:
//   1. burn(provider_lp_ata, shares_to_burn)
//   2. transfer(vault_a → provider_ata_a, amount_a)
//   3. transfer(vault_b → provider_ata_b, amount_b)
```

## 4.3 Compute Budget Analysis

| Instruction | Estimated CU | Notes |
|---|---|---|
| initialize | ~15K | Account creation only |
| deposit_sol | ~20K | Wrap SOL + update account |
| deposit_token | ~15K | Single CPI transfer |
| submit_commitment | ~25K | 2 account creations + escrow |
| reveal_order | ~10K | Verification + status update |
| match_settle | ~80K | 2 order updates + 2-4 CPI transfers + optional AMM CPI |
| cancel_order | ~20K | Transfer + status update |
| withdraw | ~15K | Single CPI transfer |
| initialize_pool | ~30K | Multiple account creations |
| add_liquidity | ~50K | 2 CPI transfers + 1 mint |
| remove_liquidity | ~50K | 1 burn + 2 CPI transfers |
| swap | ~30K | 2 CPI transfers |

**Maximum compute**: match_settle at ~80K CU. Well within Solana's 1.4M CU limit. ZK proof verification would push this higher — see §5.

---

# 5. ZK ARCHITECTURE RECOMMENDATION

## 5.1 The Fundamental Problem

Solana is a **fully transparent** blockchain. Every transaction, every account, every instruction is publicly visible. There is no encrypted mempool, no private state, no native privacy.

A dark pool on Solana faces three core challenges:

1. **Transaction privacy**: Orders submitted to Solana are immediately visible to all validators and MEV bots.
2. **Compute limits**: ZK proof verification (Groth16) requires ~200K-500K compute units, consuming a significant portion of the transaction budget.
3. **State privacy**: Account data is readable by anyone via RPC queries.

## 5.2 Production Recommendation: Hybrid Architecture

**Recommendation: Off-chain proving + On-chain commitment verification + Encrypted mempool via Jito bundles**

```
┌──────────────────────────────────────────────────────────────────────┐
│                     BASTION ZK ARCHITECTURE                         │
│                                                                      │
│  ┌─────────────────┐                                                │
│  │  USER WALLET     │                                                │
│  │                  │                                                │
│  │  1. Generate     │                                                │
│  │     Pedersen     │     ┌──────────────────────────────────┐     │
│  │     commitments  │     │  BASTION RELAYER                 │     │
│  │                  │     │                                  │     │
│  │  2. Create ZK    │────►│  3. Collect encrypted orders    │     │
│  │     proof        │     │  4. Verify proof off-chain      │     │
│  │                  │     │  5. Match complementary orders   │     │
│  │  3. Encrypt      │     │  6. Build settlement tx          │     │
│  │     order for    │     │  7. Submit via Jito bundle       │     │
│  │     relayer      │     │     (MEV-protected)              │     │
│  └─────────────────┘     │                                  │     │
│                           │  ZK Prover Service:              │     │
│                           │  - circom + snarkjs (Groth16)    │     │
│                           │  - or halo2 (no trusted setup)   │     │
│                           └──────────────┬───────────────────┘     │
│                                          │                         │
│                                          ▼                         │
│                           ┌──────────────────────────────────┐     │
│                           │  SOLANA ON-CHAIN                  │     │
│                           │                                  │     │
│                           │  submit_commitment:               │     │
│                           │    - commitment hash (32 bytes)   │     │
│                           │    - nullifier hash (32 bytes)    │     │
│                           │    - deposit amount               │     │
│                           │                                  │     │
│                           │  match_settle:                    │     │
│                           │    - Two matched commitments      │     │
│                           │    - Execution price              │     │
│                           │    - Optional: verify proof       │     │
│                           │      hash (not full proof)        │     │
│                           │                                  │     │
│                           │  ZK Verification Options:         │     │
│                           │    Option A: Hash-only (cheap)    │     │
│                           │      - Verify proof_hash matches  │     │
│                           │      - Trust relayer verification │     │
│                           │      - CU: ~5K                    │     │
│                           │                                  │     │
│                           │    Option B: Full Groth16 (costly)│     │
│                           │      - Verify [π, A, B, C]       │     │
│                           │      - Verify against vk          │     │
│                           │      - CU: ~300K-500K             │     │
│                           │                                  │     │
│                           │    RECOMMENDED: Option A +        │     │
│                           │    cryptographic accountability   │     │
│                           └──────────────────────────────────┘     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

## 5.3 Detailed Architecture

### 5.3.1 Off-chain Components

**ZK Prover Service** (runs in secure enclave or TEE):
- Language: Rust
- Proof system: **halo2** (recommended over Groth16 because: no trusted setup, recursive proofs possible, better for batch verification)
- Circuit: Commitment validity circuit
  - Input: amount, side, price, nonce, secret
  - Output: commitment hash, nullifier hash
  - Proof: "I know values that hash to this commitment, and they are in valid ranges"
- Verification key: Embedded in on-chain program or managed by relayer

**Order Encryption**:
- User encrypts order details (amount, side, price) using relayer's public key
- Uses ECIES (Elliptic Curve Integrated Encryption Scheme) or HPKE (Hybrid Public Key Encryption)
- Only the relayer can decrypt

**Matching Engine**:
- Collects encrypted orders from on-chain commitments
- Decrypts and matches complementary orders
- Builds settlement transaction
- Submits via Jito bundle for MEV protection

### 5.3.2 On-chain Components

**Commitment Scheme** (on-chain):
```
commitment = PedersenCommit(amount, side, price, nonce, blinding)
nullifier  = PoseidonHash(secret, nonce)

// Stored on-chain:
OrderCommitment {
    commitment: [u8; 32],        // hash of (amount || side || price || nonce)
    nullifier_hash: [u8; 32],    // for double-spend prevention
    amount_commitment: [u8; 32],  // Pedersen commitment to amount
    side_commitment: [u8; 32],    // Pedersen commitment to side
    price_commitment: [u8; 32],   // Pedersen commitment to price
    ...
}
```

**Verification Strategy** (RECOMMENDED: Option A — Hash-based):
```
On submission:
1. User generates ZK proof off-chain
2. Proof is hashed: proof_hash = sha256(proof_bytes)
3. Only proof_hash is stored on-chain (32 bytes, ~0 compute for verification)
4. Relayer verifies full proof off-chain before accepting order
5. On settlement, relayer provides proof_hash as accountability

Why this is safe:
- Relayer is trusted for matching (inevitable in any encrypted mempool design)
- If relayer cheats, user can prove fraud by revealing the full proof
- Proof hash creates cryptographic accountability
- Drastically reduces on-chain compute
```

**Verification Strategy** (OPTIONAL: Option B — Full on-chain verification):
```
For maximum trustlessness, verify Groth16 proofs on-chain.

Using: solana-zk-token-sdk (Solana's built-in ZK primitives)
Or: Custom verification using bn254 pairing checks

CU cost: ~300K-500K per verification
This consumes 20-35% of the transaction compute budget.

Only feasible if:
- Transaction contains ONLY the verification + minimal state updates
- Settlement is batched (multiple orders in one transaction)
- Priority fees are high enough to justify compute

NOT RECOMMENDED for MVP. Consider for V2 after proving economics work.
```

### 5.3.3 Encrypted Mempool via Jito

**Problem**: Even with commitments, the submission transaction itself reveals that someone is trading.

**Solution**: Use Jito bundles.

```
User → Relayer (encrypted) → Jito Validator (bundle) → Solana

How it works:
1. User encrypts order and sends to Bastion relayer
2. Relayer verifies ZK proof off-chain
3. Relayer builds the commitment transaction
4. Relayer submits as a Jito bundle (tip to validator)
5. Bundle is included in next block without being visible to other searchers
6. MEV protection: Jito validators don't show bundles to searchers

This provides:
✓ Order submission privacy (not in public mempool)
✓ MEV protection (Jito tips ensure inclusion)
✓ Matching privacy (relayer matches off-chain)
✓ Settlement finality (on-chain execution)
```

### 5.3.4 Nullifier Implementation

```rust
// Nullifier PDA ensures uniqueness by construction
// seeds: ["nullifier", nullifier_hash.as_ref()]

// When submitting order:
let (nullifier_pda, bump) = Pubkey::find_program_address(
    &[b"nullifier", nullifier_hash.as_ref()],
    program_id
);

// If account at nullifier_pda already exists → transaction fails
// This is the anti-double-spend guarantee
// No need to scan any list; PDA derivation IS the uniqueness check

// This is MORE efficient than Casper's dictionary approach because:
// 1. O(1) uniqueness check (PDA derivation)
// 2. No dictionary iteration
// 3. Rent model ensures accounts aren't garbage collected
// 4. PDA bump provides additional integrity guarantee
```

### 5.3.5 Privacy Level Comparison

| Feature | Casper Implementation | Solana Option A (Hash) | Solana Option B (Full ZK) |
|---|---|---|---|
| Order amount hidden on-chain | ✅ (commitment) | ✅ (commitment) | ✅ (commitment) |
| Order side hidden on-chain | ✅ (commitment) | ✅ (commitment) | ✅ (commitment) |
| Order price hidden on-chain | ✅ (commitment) | ✅ (commitment) | ✅ (commitment) |
| Mempool privacy | ❌ (public) | ✅ (Jito bundles) | ✅ (Jito bundles) |
| On-chain proof verification | ❌ (simulation only) | ❌ (hash only) | ✅ (full verify) |
| Anti-front-running | ⚠️ (encrypted mempool claim) | ✅ (Jito + commitment) | ✅ (Jito + commitment) |
| Trust model | Trust contract | Trust relayer | Trust math |
| Compute cost | N/A (simulated) | ~5K CU | ~400K CU |

**Final recommendation for V1**: Option A with Jito bundles. Full on-chain verification can be added in V2 when economics are proven.

---

# 6. SECURITY REDESIGN

## 6.1 Solana-Specific Attack Surfaces

### 6.1.1 PDA Validation

**Risk**: Attacker passes a non-PDA account that looks like a valid data structure.

**Mitigation**: Anchor's `#[account(...)]` constraints automatically verify:
- Account is owned by the correct program
- Account discriminator matches
- PDA seeds and bump are valid
- Signer constraints are enforced

```rust
// ALWAYS use seeds + bump in account validation
#[account(
    seeds = [b"order", commitment.as_ref()],
    bump = order.bump,  // Store bump in account, verify on every instruction
    constraint = order.status == OrderStatus::Committed
)]
pub order: Account<'info, OrderCommitment>,
```

### 6.1.2 Signer Spoofing

**Risk**: Attacker creates a transaction where they appear as the authority.

**Mitigation**: Anchor's `Signer<'info>` type ensures the account has signed the transaction. Use `constraint` for additional checks:

```rust
#[account(
    constraint = order.beneficiary.key() == beneficiary.key() @ ErrorCode::Unauthorized
)]
pub beneficiary: Signer<'info>,
```

### 6.1.3 CPI Re-entrancy

**Risk**: Called program calls back into our program with malicious data.

**Mitigation**: Solana's runtime prevents true re-entrancy (a program cannot be called recursively in the same transaction as of Solana v1.14+). However, cross-program invocation can still cause issues.

**Best practices**:
- Validate all accounts passed to CPI
- Use `CpiContext::new_with_signer` for PDA signing
- Never trust data from CPI return values — read from accounts after CPI

### 6.1.4 Replay Protection

**Risk**: Same transaction submitted twice.

**Mitigation**: Solana uses recent blockhashes for replay protection (transactions expire after ~150 blocks, ~60 seconds). This is different from Casper's approach.

**Additional protection for orders**:
- Nullifier PDA ensures the same nullifier cannot be used twice
- Order PDA ensures the same commitment cannot be submitted twice
- Both are enforced by PDA derivation collision

### 6.1.5 Front-Running on Solana

**Reality**: Solana has significant MEV activity, primarily through Jito.

**Attack vectors**:
1. **Leader schedule**: Validators know the upcoming leader and can pre-submit transactions.
2. **Jito bundles**: Searchers can front-run by including their transaction before yours in a bundle.
3. **Priority fees**: Higher-fee transactions get processed first.

**Mitigations for Bastion**:
1. **Commitment-based orders**: Amount/side/price are hidden. Front-runners only see a commitment hash.
2. **Jito bundles for submission**: Use Jito to prevent front-running of order submissions.
3. **Relayer-submitted settlements**: The match engine submits settlement transactions via Jito bundles with tips, ensuring MEV-protected execution.
4. **Batch settlement**: Multiple order pairs settled in one transaction, making individual order front-running impossible.
5. **Encrypted mempool**: Orders are never in the public mempool.

### 6.1.6 Account Exhaustion / Rent

**Risk**: Attacker creates many accounts to exhaust a user's SOL.

**Mitigation**:
- Users must sign all transactions that create accounts in their name
- Rent is always paid by the signer (`payer = authority` or `payer = user`)
- Program validates that accounts are properly sized

### 6.1.7 Token Account Drain

**Risk**: Attacker tricks user into approving a malicious delegate.

**Mitigation**:
- Never request blanket `approve` (infinite allowance)
- Use specific amounts for `approve` + `transfer_from`
- Use CPI for atomic operations (approve + transfer in same transaction)
- Consider Token-2022's `close_account` authority to reclaim rent

### 6.1.8 Compute Exhaustion

**Risk**: Instruction consumes all compute units, causing transaction to fail.

**Mitigation**:
- Use `ComputeBudget` instructions to set explicit limits
- Profile all instructions on local validator
- Set `compute_unit_limit` appropriately
- Use `compute_unit_price` for priority

## 6.2 Security Checklist

| Item | Casper | Solana | Status |
|---|---|---|---|
| Integer overflow | U256 native | u64/u128 checked math | Must use `checked_*` operations |
| Re-entrancy | Casper VM protection | Solana runtime protection | ✅ Mitigated by runtime |
| Access control | Security badges | Signer checks + PDA ownership | Must implement in Anchor |
| Slippage protection | `min_amount_out` | Same pattern | ✅ Direct port |
| Double-spend | Nullifier dictionary | Nullifier PDA | ✅ Stronger (PDA enforces uniqueness) |
| Front-running | Limited | Jito bundles + commitments | ✅ Better than Casper |
| MEV extraction | High (public mempool) | Mitigated (Jito + commitments) | ✅ Significant improvement |
| Replay | Casper handles | Blockhash-based | ✅ Different but effective |
| Pausing | Not implemented | Config.paused flag | ✅ New feature |
| Upgrade risk | Contract package versioning | Buffer + upgrade authority | Must use multisig |

---

# 7. PERFORMANCE OPTIMIZATION

## 7.1 Compute Unit Optimization

| Optimization | Impact | Implementation |
|---|---|---|
| Use u64 instead of U256 | **MAJOR** | Solana's SPL Token uses u64. AMM math can use u128 for intermediate calculations. Eliminates U256 overhead entirely. |
| Pre-compute PDA addresses off-chain | **MODERATE** | Client computes PDA addresses and passes them in. Avoids on-chain `find_program_address`. |
| Minimize CPI depth | **MODERATE** | Each CPI adds ~5K CU overhead. Limit to 2-3 levels. |
| Use `zero_copy` for large accounts | **MODERATE** | For accounts > 10KB, use `zero_copy` to avoid deserialization overhead. |
| Batch operations | **MAJOR** | Process multiple orders in a single transaction (match_settle with N pairs). |
| Avoid unnecessary account loads | **MODERATE** | Only load accounts that are actually modified. |
| Use instruction reflection | **MINOR** | Process different operations in same instruction based on discriminator. |

## 7.2 Account Size Optimization

| Technique | Savings | Notes |
|---|---|---|
| Use enums instead of string status | 30+ bytes | `OrderStatus::Committed` (1 byte) vs "committed" (9+ bytes) |
| Use Pubkey (32 bytes) not String | Variable | No dynamic strings in accounts |
| Pack booleans into bitflags | 7 bytes per 8 bools | If multiple boolean flags needed |
| Use u32 timestamps where possible | 4 bytes | If precision allows (until 2106) |
| Reserved space for upgrades | +32-64 bytes | Avoid account resizing (expensive on Solana) |

## 7.3 Transaction Packing

```
Single transaction limits:
- 1,232 bytes of instruction data (versioned transactions: larger)
- 1,400,000 compute units
- Up to 64 accounts (versioned: 256 with ALT)
- Up to 10 instructions (no hard limit, but practical)

For settlement batching:
- Use Versioned Transactions (v0) with Address Lookup Tables (ALT)
- Pack 5-10 order pair settlements per transaction
- Use Jito bundles for MEV protection

Example batch settlement:
  Instruction 1: match_settle(order_a1, order_b1)
  Instruction 2: match_settle(order_a2, order_b2)
  Instruction 3: match_settle(order_a3, order_b3)
  ...
  Compute: ~80K × N pairs
  Max pairs per tx: ~15-17 (within 1.4M CU)
```

## 7.4 Event Emission Optimization

```rust
// Anchor event emission (uses program log)
emit!(OrderCommitted {
    commitment: order.commitment,
    nullifier: order.nullifier_hash,
    deposit_mint: order.deposit_mint,
    timestamp: Clock::get()?.unix_timestamp,
});

// Optimize: Only emit essential fields
// Indexer can derive the rest from account data

// Alternative: Ring buffer in account for recent events
// (avoids log parsing entirely, but costs rent)
```

## 7.5 RPC Load Optimization

```
1. Use dedicated RPC endpoints (Helius, Triton, QuickNode)
2. Cache account data locally
3. Use WebSocket subscriptions for real-time updates
4. Batch RPC calls with `getMultipleAccounts`
5. Use `getProgramAccounts` with filters for indexer
6. Prefer `getTokenAccountsByOwner` over `getProgramAccounts` for user data
```

---

# 8. FRONTEND + SDK REWRITE

## 8.1 TypeScript SDK Architecture

```
bastion-sdk/
├── src/
│   ├── index.ts                  # Main exports
│   ├── types/
│   │   ├── accounts.ts           # Account type definitions (mirrors Anchor IDL)
│   │   ├── instructions.ts       # Instruction type definitions
│   │   ├── events.ts             # Event type definitions
│   │   └── constants.ts          # Program IDs, PDA seeds, etc.
│   ├── client/
│   │   ├── BastionClient.ts      # Main client class
│   │   ├── DarkPoolClient.ts     # Dark pool operations
│   │   ├── AmmClient.ts          # AMM operations
│   │   └── IdentityClient.ts     # Identity management
│   ├── crypto/
│   │   ├── commitments.ts        # Pedersen commitment generation
│   │   ├── nullifiers.ts         # Nullifier generation
│   │   ├── encryption.ts         # Order encryption (HPKE)
│   │   └── proof.ts              # ZK proof generation (WASM)
│   ├── relayer/
│   │   ├── RelayerClient.ts      # Relayer communication
│   │   └── JitoBundler.ts        # Jito bundle submission
│   ├── rpc/
│   │   ├── SolanaRpc.ts          # RPC wrapper with retry
│   │   ├── WebSocketManager.ts   # WebSocket subscriptions
│   │   └── IndexerClient.ts      # Event indexer communication
│   └── utils/
│       ├── pda.ts                 # PDA derivation helpers
│       ├── transaction.ts         # Transaction building utilities
│       └── conversion.ts          # Unit conversion (lamports, etc.)
├── tests/
│   ├── unit/
│   └── integration/
├── package.json
└── tsconfig.json
```

## 8.2 Core SDK API

```typescript
// ─── Client Initialization ───
import { BastionClient } from '@bastion/sdk';

const client = new BastionClient({
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  wsUrl: 'wss://api.mainnet-beta.solana.com',
  commitment: 'confirmed',
  programId: new PublicKey('BASTi0N...'),
});

// ─── Wallet Connection ───
await client.connect(walletAdapter); // Phantom, Solflare, etc.

// ─── Dark Pool Operations ───
const darkPool = client.darkPool;

// Deposit SOL
await darkPool.depositSol({
  amount: 10_000_000_000, // 10 SOL in lamports
});

// Deposit Token
await darkPool.depositToken({
  mint: usdcMint,
  amount: 5_000_000_000, // 5000 USDC (6 decimals)
});

// Submit encrypted order
const commitment = await darkPool.submitOrder({
  side: 'buy',
  amount: 1_000_000_000, // 1 SOL
  price: 175_00, // $175.00 (in cents)
  tokenMint: usdcMint,
  // ZK proof generated automatically by SDK
});

// Cancel order
await darkPool.cancelOrder({
  commitment: commitment.hash,
});

// Withdraw
await darkPool.withdraw({
  mint: NATIVE_MINT,
  amount: 5_000_000_000,
});

// ─── AMM Operations ───
const amm = client.amm;

// Get pool info
const pool = await amm.getPool({
  mintA: NATIVE_MINT,
  mintB: usdcMint,
});

// Swap
const quote = await amm.quoteSwap({
  pool,
  inputMint: NATIVE_MINT,
  inputAmount: 1_000_000_000, // 1 SOL
  slippageBps: 50, // 0.5%
});

const tx = await amm.swap({
  pool,
  inputMint: NATIVE_MINT,
  inputAmount: 1_000_000_000,
  minOutputAmount: quote.minOutputAmount,
});

// Add Liquidity
const lpTx = await amm.addLiquidity({
  pool,
  amountA: 10_000_000_000, // 10 SOL
  amountB: 1_750_000_000_000, // 1750 USDC
  slippageBps: 100,
});

// Remove Liquidity
const removeTx = await amm.removeLiquidity({
  pool,
  lpShares: 500_000_000,
  minAmountA: 9_000_000_000,
  minAmountB: 1_500_000_000_000,
});

// ─── Real-time Events ───
client.onOrderCommitted((event) => {
  console.log('New order committed:', event.commitment);
});

client.onOrdersMatched((event) => {
  console.log('Orders matched at:', event.executionPrice);
});

client.onSwapExecuted((event) => {
  console.log('Swap:', event.amountIn, '→', event.amountOut);
});
```

## 8.3 React Integration

```typescript
// hooks/useBastion.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { BastionClient } from '@bastion/sdk';

export function useBastion() {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [client, setClient] = useState<BastionClient | null>(null);
  
  useEffect(() => {
    if (wallet.connected) {
      const c = new BastionClient({
        rpcUrl: connection.rpcEndpoint,
        wallet: wallet,
      });
      setClient(c);
    }
  }, [wallet.connected]);
  
  return client;
}

// hooks/usePool.ts
export function usePool(mintA: PublicKey, mintB: PublicKey) {
  const client = useBastion();
  const [pool, setPool] = useState<PoolInfo | null>(null);
  const [reserves, setReserves] = useState<ReserveInfo | null>(null);
  
  useEffect(() => {
    if (!client) return;
    
    const fetchPool = async () => {
      const poolInfo = await client.amm.getPool({ mintA, mintB });
      setPool(poolInfo);
      setReserves({
        reserveA: poolInfo.vaultABalance,
        reserveB: poolInfo.vaultBBalance,
      });
    };
    
    fetchPool();
    
    // Subscribe to pool changes
    const sub = client.onPoolUpdate({ mintA, mintB }, (update) => {
      setReserves(update.reserves);
    });
    
    return () => sub.unsubscribe();
  }, [client, mintA, mintB]);
  
  return { pool, reserves };
}

// hooks/useTicker.ts
export function useTicker(mintA: PublicKey, mintB: PublicKey) {
  const client = useBastion();
  const [price, setPrice] = useState<number>(0);
  const [history, setHistory] = useState<number[]>([]);
  
  useEffect(() => {
    if (!client) return;
    
    const interval = setInterval(async () => {
      const pool = await client.amm.getPool({ mintA, mintB });
      const newPrice = pool.vaultBBalance / pool.vaultABalance;
      setPrice(newPrice);
      setHistory(prev => [...prev.slice(-59), newPrice]);
    }, 2000);
    
    return () => clearInterval(interval);
  }, [client]);
  
  return { price, history };
}
```

## 8.4 CLI Replacement

Replace Bash CLI with Rust CLI (for consistency with on-chain code) or TypeScript CLI (for web compatibility).

**Recommendation**: TypeScript CLI (using `@bastion/sdk` + `commander` + `ink` for TUI)

```typescript
// cli/src/commands/trade.ts
import { Command } from 'commander';
import { BastionClient } from '@bastion/sdk';
import { promptAmount, promptSide, confirmOrder } from '../ui/prompts';

export const tradeCommand = new Command('trade')
  .description('Execute a trade on Bastion Dark Pool')
  .option('-s, --side <side>', 'Buy or Sell')
  .option('-a, --amount <amount>', 'Amount in SOL')
  .action(async (options) => {
    const client = new BastionClient({ /* config */ });
    await client.connect(/* keypair or wallet */);
    
    const side = options.side || await promptSide();
    const amount = options.amount || await promptAmount();
    
    await confirmOrder({ side, amount });
    
    const result = await client.darkPool.submitOrder({
      side: side.toLowerCase(),
      amount: solToLamports(amount),
      tokenMint: USDC_MINT,
    });
    
    console.log(`Order committed: ${result.commitment}`);
    console.log(`Status: ${result.status}`);
  });
```

---

# 9. DEPLOYMENT PIPELINE

## 9.1 Local Development

```bash
# 1. Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# 2. Install Anchor
avm install latest
avm use latest

# 3. Start local validator
solana-test-validator --reset --quiet

# 4. Build programs
anchor build

# 5. Deploy to local
anchor deploy

# 6. Run tests
anchor test --skip-local-validator
```

## 9.2 Anchor.toml

```toml
[features]
resolution = true
skip-lint = false

[programs.localnet]
bastion_pool = "BASTi0N11111111111111111111111111111111111111"
bastion_amm = "BASTAMM1111111111111111111111111111111111111"

[programs.devnet]
bastion_pool = "BASTi0Ndevnet1111111111111111111111111111111"
bastion_amm = "BASTAMMdevnet1111111111111111111111111111111"

[programs.mainnet]
bastion_pool = "BASTi0N11111111111111111111111111111111111111"
bastion_amm = "BASTAMM1111111111111111111111111111111111111"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"

[test]
startup_wait = 5000
```

## 9.3 CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: Bastion CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Solana
        run: sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
        
      - name: Install Anchor
        run: cargo install --git https://github.com/coral-xyz/anchor avm
        
      - name: Build Programs
        run: anchor build
        
      - name: Run Tests
        run: anchor test
        
      - name: Security Audit
        run: cargo audit
        
      - name: Lint
        run: cargo clippy --all-targets -- -D warnings

  deploy-devnet:
    needs: build
    if: github.ref == 'refs/heads/develop'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to Devnet
        run: |
          solana config set --url devnet
          anchor deploy --provider.cluster devnet
        env:
          ANCHOR_WALLET: ${{ secrets.DEVNET_DEPLOY_KEY }}

  deploy-mainnet:
    needs: build
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to Mainnet (via Squads multisig)
        run: |
          # Build program buffer
          solana program write-buffer target/deploy/bastion_pool.so
          
          # Submit to Squads multisig for approval
          # (requires multisig threshold approval before upgrade)
```

## 9.4 Verifiable Builds

```bash
# Build with deterministic flag
anchor build -- --config .cargo/config-deterministic.toml

# Verify on-chain program matches source
sbv2 verify <PROGRAM_ID> https://github.com/bastion-protocol/bastion-solana

# Or use Solana Verify
solana-verify verify-from-repo \
  --program-id <PROGRAM_ID> \
  --url https://api.mainnet-beta.solana.com \
  https://github.com/bastion-protocol/bastion-solana
```

## 9.5 Upgrade Authority Management

```
Upgrade authority flow:
1. Initial deploy: Authority = deployer keypair
2. Transfer to Squads multisig (3-of-5 threshold)
3. All upgrades require multisig approval
4. Buffer account created → Multisig vote → Execute upgrade
5. Emergency pause: Config.paused = true (single instruction, no upgrade needed)
```

---

# 10. FINAL MIGRATION ROADMAP

## 10.1 Phase Overview

| Phase | Duration | Description | Risk |
|---|---|---|---|
| **Phase 0** | 2 weeks | Architecture design + specification | LOW |
| **Phase 1** | 4 weeks | Core AMM program (Anchor) | LOW |
| **Phase 2** | 4 weeks | Dark pool commitment program | MEDIUM |
| **Phase 3** | 3 weeks | ZK proof integration (off-chain) | HIGH |
| **Phase 4** | 3 weeks | Relayer + matching engine | HIGH |
| **Phase 5** | 3 weeks | TypeScript SDK | LOW |
| **Phase 6** | 2 weeks | CLI + TUI rewrite | LOW |
| **Phase 7** | 2 weeks | Indexer + event pipeline | MEDIUM |
| **Phase 8** | 3 weeks | Integration testing | MEDIUM |
| **Phase 9** | 2 weeks | Security audit preparation | LOW |
| **Phase 10** | 4 weeks | External security audit | HIGH |
| **Phase 11** | 2 weeks | Testnet deployment + testing | MEDIUM |
| **Phase 12** | 1 week | Mainnet deployment | HIGH |
| **Total** | ~35 weeks | | |

## 10.2 Phase Details

### Phase 0: Architecture Design (Week 1-2)
- Finalize account schemas
- Design PDA seed schemas
- Define instruction set
- Define event schema
- Create Anchor IDL draft
- Design ZK circuit (specification)
- Design relayer protocol
- Security threat model

**Deliverables**: Architecture specification document, IDL draft, threat model

### Phase 1: AMM Program (Week 3-6)
- Implement `bastion-amm` Anchor program
- Initialize pool
- Add/remove liquidity with LP token minting
- Swap A→B and B→A
- Slippage protection
- Fee collection
- Event emission
- Unit tests + integration tests
- Compute profiling

**Deliverables**: Working AMM on local validator with 100% test coverage

### Phase 2: Dark Pool Commitment Program (Week 7-10)
- Implement `bastion-pool` Anchor program
- Initialize config
- Deposit SOL/token
- Submit commitment (with nullifier PDA)
- Cancel order
- Withdraw
- Match settle (basic version without ZK verification)
- Event emission
- Tests

**Deliverables**: Working dark pool (trust-based matching) on local validator

### Phase 3: ZK Proof Integration (Week 11-13)
- Implement halo2 circuits for:
  - Commitment validity
  - Amount range proof
  - Side commitment
  - Price range proof
- Build WASM prover for browser/Node.js
- Integrate with SDK
- Off-chain proof generation
- On-chain proof hash storage

**Deliverables**: Working ZK proof generation + verification pipeline

### Phase 4: Relayer + Matching Engine (Week 14-16)
- Build relayer service (Rust or TypeScript)
- Order collection from on-chain events
- Decryption of encrypted order details
- Matching algorithm (price-time priority)
- Jito bundle submission
- Relayer API (REST + WebSocket)

**Deliverables**: Working relayer with Jito integration

### Phase 5: TypeScript SDK (Week 17-19)
- Core client implementation
- Wallet adapter integration
- PDA derivation helpers
- Transaction building
- Event subscription
- ZK proof generation (WASM)
- Order encryption
- Full API documentation

**Deliverables**: Published `@bastion/sdk` npm package

### Phase 6: CLI + TUI (Week 20-21)
- TypeScript CLI with `commander`
- Interactive TUI with `ink` or React-based terminal UI
- Commands: trade, deposit, withdraw, liquidity, ticker, identity, history
- Wallet management
- Real-time event streaming

**Deliverables**: `npx @bastion/cli` or `bastion` npm global package

### Phase 7: Indexer + Event Pipeline (Week 22-23)
- Helius webhook integration or custom Geyser plugin
- Event parsing and storage
- REST API for event queries
- WebSocket API for real-time updates
- Historical data API

**Deliverables**: Event indexer API

### Phase 8: Integration Testing (Week 24-26)
- End-to-end tests on local validator
- Multi-user scenario tests
- MEV resistance tests (with Jito simulation)
- Load testing (transaction throughput)
- Edge case testing
- Cross-program interaction tests

**Deliverables**: Test report, bug fixes

### Phase 9: Security Audit Prep (Week 27-28)
- Code cleanup
- Documentation
- Audit scope document
- Known issues list
- Fuzzing setup

**Deliverables**: Audit-ready codebase

### Phase 10: External Security Audit (Week 29-32)
- Engage security firm (recommended: Trail of Bits, OtterSec, or Neodyme)
- Address findings
- Re-audit critical fixes

**Deliverables**: Audit report, fixes applied

### Phase 11: Testnet Deployment (Week 33-34)
- Deploy to Solana devnet
- Public beta testing
- Bug bounty program
- Community feedback

**Deliverables**: Live devnet deployment

### Phase 12: Mainnet Deployment (Week 35)
- Final security review
- Multisig approval
- Mainnet deployment
- Monitoring setup
- Incident response plan

**Deliverables**: Live mainnet deployment

## 10.3 Risk Assessment

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| ZK proof doesn't fit in compute budget | MEDIUM | HIGH | Use hash-based verification (Option A) |
| Jito bundle submission fails | LOW | MEDIUM | Fallback to regular transaction submission |
| PDA collision (different seeds, same address) | VERY LOW | CRITICAL | Use sufficiently long, unique seeds |
| Account rent drain | LOW | MEDIUM | Use Token-2022 for reclaimable accounts |
| Relayer goes down | MEDIUM | HIGH | Decentralized relayer network (future) |
| Anchor framework breaking changes | LOW | MEDIUM | Pin Anchor version, test upgrades carefully |
| Mainnet congestion | MEDIUM | MEDIUM | Priority fees, retry logic |
| SPL Token v0 migration needed | LOW | LOW | Design for Token-2022 from start |

---

# FINAL DELIVERABLES

## 1. Recommended Final Stack

```
ON-CHAIN:
  - Anchor 0.30+ (Solana v1.18+)
  - SPL Token (standard, not Token-2022 for V1 — simpler, broader support)
  - Token-2022 migration path for V2

OFF-CHAIN:
  - ZK Prover: halo2 + WASM (Rust)
  - Relayer: Rust (axum + tokio)
  - Matching Engine: Rust
  - Jito Bundle Integration: Rust

SDK:
  - TypeScript (Node.js + browser)
  - @solana/web3.js
  - @coral-xyz/anchor
  - WASM-pack for ZK prover

CLI/TUI:
  - TypeScript (commander + ink)

INDEXER:
  - Helius (recommended for production)
  - Custom Geyser plugin (if needed)

RPC:
  - Primary: Helius
  - Fallback: Triton, QuickNode

WALLET:
  - @solana/wallet-adapter (Phantom, Solflare, Backpack, etc.)

CI/CD:
  - GitHub Actions
  - Squads multisig for program authority
```

## 2. Recommended Repository Structure

```
bastion-solana/
├── programs/
│   ├── bastion-pool/           # Dark pool program
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Program entry
│   │       ├── state.rs        # Account structs
│   │       ├── instructions/
│   │       │   ├── mod.rs
│   │       │   ├── initialize.rs
│   │       │   ├── deposit.rs
│   │       │   ├── submit_commitment.rs
│   │       │   ├── reveal_order.rs
│   │       │   ├── match_settle.rs
│   │       │   ├── cancel_order.rs
│   │       │   └── withdraw.rs
│   │       ├── errors.rs
│   │       └── events.rs
│   ├── bastion-amm/            # AMM program
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── state.rs
│   │       ├── instructions/
│   │       │   ├── mod.rs
│   │       │   ├── initialize_pool.rs
│   │       │   ├── add_liquidity.rs
│   │       │   ├── remove_liquidity.rs
│   │       │   ├── swap.rs
│   │       │   └── quote.rs
│   │       ├── math.rs         # AMM math (constant product)
│   │       ├── errors.rs
│   │       └── events.rs
│   └── shared/                 # Shared types and utils
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── types.rs
├── zk/
│   ├── circuit/                # halo2 circuits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # Commitment validity circuit
│   └── prover-wasm/            # WASM prover for browser/Node
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── relayer/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── api.rs              # REST API
│       ├── matcher.rs          # Order matching engine
│       ├── jito.rs             # Jito bundle submission
│       ├── encryption.rs       # Order decryption
│       └── proof.rs            # Off-chain proof verification
├── sdk/
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── index.ts
│       ├── client/
│       ├── crypto/
│       ├── relayer/
│       ├── rpc/
│       └── utils/
├── cli/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── commands/
│       └── ui/
├── indexer/
│   ├── package.json
│   └── src/
│       ├── index.ts
│       ├── parsers/
│       └── storage/
├── tests/
│   ├── anchor/
│   │   ├── bastion-pool.ts
│   │   └── bastion-amm.ts
│   └── integration/
│       ├── e2e-trade.ts
│       └── e2e-liquidity.ts
├── scripts/
│   ├── deploy.ts
│   ├── airdrop.ts
│   └── init-pool.ts
├── Anchor.toml
├── Cargo.toml                  # Workspace
├── package.json                # Monorepo root
└── README.md
```

## 3. Recommended Team Structure

| Role | Count | Responsibility |
|---|---|---|
| Solana Protocol Lead | 1 | Architecture, PDA design, instruction design, Anchor programs |
| Rust Smart Contract Engineer | 2 | On-chain program implementation |
| ZK/Cryptography Engineer | 1 | Circuit design, proof generation, verification |
| Backend Engineer | 1 | Relayer, matching engine, indexer |
| Frontend/SDK Engineer | 1-2 | TypeScript SDK, CLI, wallet integration |
| Security Engineer | 1 (or audit firm) | Security review, fuzzing, formal verification |
| DevOps | 0.5 | CI/CD, deployment, monitoring |

**Minimum viable team**: 4 engineers (1 protocol lead, 2 Rust, 1 full-stack)

## 4. Recommended Audit Priorities

1. **PDA derivation and seed collision resistance** — Critical for nullifier uniqueness
2. **Order matching logic** — Must be fair, cannot be gamed
3. **CPI safety** — All cross-program invocations must validate accounts
4. **Integer overflow in AMM math** — u64 overflow in constant product calculation
5. **Relayer trust model** — What happens if relayer goes rogue?
6. **Escrow fund safety** — Can escrow be drained by anyone other than beneficiary?
7. **LP share calculation** — First LP gets MIN_LIQUIDITY permanently locked
8. **Compute budget for settlement** — Must fit within limits even with AMM CPI
9. **Priority fee handling** — Must not create DoS vector
10. **Upgrade authority** — Must be multisig controlled

## 5. Biggest Migration Risks

1. **ZK proof verification on Solana** — Compute limits may prevent full on-chain verification. Mitigated by hash-based approach (Option A), but reduces trustlessness.

2. **Privacy model is fundamentally different** — Casper's global state + URefs provide capability-based privacy that Solana simply does not have. The dark pool must be built entirely through commitments + off-chain matching + Jito bundles. This is a different (arguably weaker) privacy model.

3. **Account model inversion** — Casper stores all user balances in a contract-owned dictionary. Solana requires each user to have their own token account. This changes the deposit/withdraw flow significantly and requires more accounts per user.

4. **MEV landscape is more adversarial on Solana** — Solana has a more mature MEV ecosystem (Jito, searchers, priority fees). Dark pool protection must be robust against sophisticated adversaries.

5. **No native event standard** — Casper's CES provides structured, schema-validated events. Solana's raw logs require building a custom event parsing and indexing layer from scratch.

6. **Rent economics** — Every account costs SOL. The Casper version creates no per-user accounts (dictionaries are free). The Solana version creates 2+ accounts per user (deposit + potentially LP position). This changes the economic model.

## 6. "If Building Today" Architecture Recommendation

**Build with Anchor. Use SPL Token (not Token-2022) for V1. Use hash-based ZK verification. Use Jito bundles for MEV protection. Deploy relayer as a centralized service initially, plan for decentralization.**

The architecture should be:

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│  USER (browser wallet)                                               │
│    │                                                                 │
│    ├── Generate commitment (WASM in browser)                        │
│    ├── Encrypt order details (HPKE)                                 │
│    └── Submit to relayer                                            │
│                                                                      │
│  RELAYER (centralized, audited)                                     │
│    │                                                                 │
│    ├── Verify ZK proof off-chain                                    │
│    ├── Submit commitment on-chain (via Jito)                        │
│    ├── Match orders off-chain                                       │
│    └── Submit settlement on-chain (via Jito)                        │
│                                                                      │
│  SOLANA ON-CHAIN                                                    │
│    │                                                                 │
│    ├── bastion-pool: Commitments, nullifiers, deposits, settlement   │
│    ├── bastion-amm: Pools, swaps, liquidity                        │
│    └── SPL Token: All token operations                              │
│                                                                      │
│  INDEXER (Helius-powered)                                           │
│    │                                                                 │
│    ├── Parse program logs                                           │
│    ├── Store events in database                                     │
│    └── Serve via REST + WebSocket                                   │
│                                                                      │
│  SDK (TypeScript)                                                   │
│    │                                                                 │
│    ├── Wallet integration                                           │
│    ├── Transaction building                                         │
│    ├── ZK proof generation (WASM)                                   │
│    └── Relayer communication                                        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

This is the most pragmatic architecture that:
- Preserves dark pool functionality
- Leverages Solana's strengths (speed, composability)
- Mitigates Solana's weaknesses (transparency, compute limits)
- Is deployable within the timeline
- Can be progressively decentralized

---

**End of Migration Blueprint**

This document represents the complete architectural analysis and migration plan for the Bastion protocol from Casper to Solana. Every component has been mapped, every incompatibility identified, and every design decision justified against Solana's specific constraints and capabilities.
