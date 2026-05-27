# Bastion Protocol Threat Model

## 1. System Boundaries
- **Client (Browser/CLI)**: Holds user keypairs, signs transactions, generates ZK proofs in WASM, encrypts order details.
- **Relayer**: Holds AES decryption key. Responsible for decrypting limits, running price-time priority matching, and submitting bundles to Jito.
- **Solana Network**: Executes `bastion-pool` and `bastion-amm` programs.

## 2. Assets
- User SOL/USDC deposits held in `UserDeposit` PDA escrows.
- Protocol fee revenues.
- LP funds in `bastion-amm`.

## 3. Threat Scenarios & Mitigations

### T1: Relayer Front-Running / MEV
*Threat*: Relayer sees all decrypted orders and attempts to insert its own trades before user trades.
*Mitigation*: The matching logic operates strictly on commitments. However, the centralized relayer in Phase 1 requires trust. Future phases will introduce multi-party computation (MPC) or TEEs for the relayer.

### T2: Forged Commitments
*Threat*: Malicious user submits an order with fake parameters to manipulate the matching engine.
*Mitigation*: The `bastion-pool` requires a cryptographic proof hash linking the commitment to the encrypted details. While Option A stores the hash on-chain, the relayer completely drops mismatched proofs off-chain.

### T3: Double Cancels / Replay Attacks
*Threat*: A user cancels an order twice to drain the escrow, or replays a signed transaction.
*Mitigation*: The `bastion-pool` state uses explicit PDA status tracking (`order.status = Cancelled`). Nonces are included in the commitment hash to prevent replay of the same parameters.
