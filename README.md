# ♜ Bastion Protocol

<div align="center">
  <p><strong>Privacy-preserving dark pool trading on Solana. Powered by Zero-Knowledge Proofs.</strong></p>
  <p>
    <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-purple.svg" alt="License: MIT" /></a>
    <a href="https://solana.com"><img src="https://img.shields.io/badge/Solana-v1.18+-00D1A0.svg" alt="Solana" /></a>
    <a href="https://www.anchor-lang.com"><img src="https://img.shields.io/badge/Anchor-0.31.1-blue.svg" alt="Anchor" /></a>
  </p>
</div>

---

## ⚡ What is Bastion?

**Bastion** is a decentralized trading protocol and CLI-first application designed to completely eliminate MEV (Miner Extractable Value) attacks such as front-running and sandwiching. 

By encrypting your orders using **ZK-SNARKs (Zero-Knowledge Proofs)** before they hit the Solana mempool, Bastion ensures your trading intent remains invisible until execution. 

| The Problem | How Bastion Solves It |
| :--- | :--- |
| 🥪 **Sandwich Attacks** | Orders are encrypted. Predatory bots cannot see your trade size or direction. |
| 🏃 **Front-running** | Commitment-based ordering means your intent is hidden until execution. |
| 📊 **Price Impact** | Dark pool matching ensures large "whale" orders don't move the public market. |
| 🔍 **Privacy** | ZK proofs verify your solvency and order validity without revealing account details. |

---

## 🚀 Quick Start

Get started trading in seconds using our interactive Terminal UI (TUI):

```bash
# 1. Install the Bastion CLI
curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/bastion/main/install-cli.sh | bash

# 2. Launch the trading terminal
bastion
```

*Note: The CLI provides an interactive menu. You can also run commands directly like `bastion trade`, `bastion deposit 5`, or `bastion ticker`.*

---

## 🛠 Core Features

- **🔐 Dark Pool Trading:** Submit orders as encrypted ZK commitments. Completely invisible to the mempool.
- **💧 AMM Fallback:** Built-in constant-product AMM ensures instant execution when dark liquidity is scarce.
- **📊 Live Market Ticker:** Real-time terminal pricing charts with sparklines and live spread detection.
- **⚔️ MEV Attack Simulator:** Educational demo showcasing sandwich attacks on public DEXs versus Bastion's protection.
- **🐳 Whale Mode (Iceberg Orders):** Automatically splits large orders into randomized chunks with time delays.
- **👥 Multi-Identity:** Hot-swap between multiple Solana keypairs instantly.

---

## 🏗 Architecture

Bastion consists of a suite of Solana programs and a powerful client-side execution engine:

```text
┌────────────────────────────────────────────────────────────┐
│                    BASTION CLI (bash/gum)                    │
│  Trade │ Deposit │ Withdraw │ Liquidity │ Ticker │ ZK Demo  │
└──────────────────────┬─────────────────────────────────────┘
                       │
          ┌────────────▼────────────┐
          │   Solana Programs       │
          │  ┌────────────────────┐ │
          │  │  bastion-pool      │ │  ← ZK Dark Pool (commitments, proofs)
          │  │  bastion-amm       │ │  ← Constant Product AMM (fallback liquidity)
          │  └────────────────────┘ │
          └────────────┬────────────┘
                       │
          ┌────────────▼────────────┐
          │  Solana Network         │
          │  localnet │ devnet      │
          └─────────────────────────┘
```

---

## 💻 Development & Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) 1.75+
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) 1.18+
- [Anchor](https://www.anchor-lang.com/docs/installation) 0.31.1
- [gum](https://github.com/charmbracelet/gum) 0.14+ (For CLI UI)

### Build & Deploy locally

```bash
# 1. Clone the repository
git clone https://github.com/krishnagoyal099/bastion.git
cd bastion

# 2. Build the Solana programs
anchor build

# 3. Start a local validator and deploy
solana-test-validator --reset &
anchor deploy

# 4. Initialize the pool
npm install
npx ts-node scripts/init-pool.ts

# 5. Run the CLI
./cli/bastion
```

### Network Configuration

Switch between networks instantly using the CLI:
```bash
bastion config set-network devnet
bastion config set-network localnet
```

---

## 🛡️ Security Guarantees

- **Mathematically Proven:** All orders are hidden behind SHA-256 commitments and validated strictly via ZK proofs.
- **No Double Spends:** A robust nullifier system prevents replay attacks.
- **Trustless Escrow:** Pool funds are locked in PDA-derived vaults. No admin keys can drain liquidity.
- **MEV-Immune:** By structurally hiding order parameters, MEV bots lack the data required to attack.

*(See our [Audit Scope](./audit_scope.md) for full security analysis)*

---

## 📂 Project Structure

- `cli/` - The interactive Terminal UI (bash + gum)
- `programs/` - Solana smart contracts (Anchor)
    - `bastion-pool/` - ZK Dark Pool logic
    - `bastion-amm/` - Constant Product AMM
- `landing-page/` - React/Vite promotional website
- `scripts/` - TypeScript deployment & initialization tools
- `zk/` - Halo2 circuits and WASM provers

---

<div align="center">
  <p>Built with 🖤 for the Solana Ecosystem.</p>
</div>
