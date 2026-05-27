# Bastion Protocol - Audit Scope

## 1. Overview
The Bastion Protocol is migrating from a Casper-based architecture to a Solana-based architecture. This document outlines the scope for the Phase 10 external security audit.

## 2. In-Scope Components
The following components are strictly within the scope of the security audit:
- **`bastion-pool` (Solana Program)**: The dark pool escrow, ZK commitment verification, and order matching settlement.
- **`bastion-amm` (Solana Program)**: The underlying AMM for automated liquidity provisioning.
- **`zk/circuit`**: Halo2 constraint system for commitment validation (Option A implementation).

## 3. Out-of-Scope Components
The following components are provided for context but are out-of-scope for the primary smart contract audit:
- Relayer (`relayer/`)
- TypeScript SDK (`sdk/`)
- CLI Tool (`cli/`)
- Indexer (`indexer/`)

## 4. Known Issues & Assumptions
- **Centralized Relayer**: The MVP utilizes a trusted relayer for decryption and matching. The relayer cannot steal funds, but can censor or delay order execution. This is a known architectural trade-off for Phase 1.
- **Proof Verification**: "Option A" relies on hashing a client-side generated proof rather than a full on-chain verifier due to Solana compute budget constraints.
