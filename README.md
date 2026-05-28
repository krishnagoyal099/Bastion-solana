<div align="center">
  <img src="./assets/logo.png" alt="Bastion Logo" width="140" />
  <h1>Bastion Protocol</h1>
  <p><strong>ZK-powered dark pool trading on Solana — your trades stay invisible.</strong></p>
  <br/>
  <a href="https://bastion-solana.vercel.app"><img src="https://img.shields.io/badge/Website-bastion--solana.vercel.app-8B5CF6?style=for-the-badge" alt="Website" /></a>
  <br/><br/>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-purple.svg" alt="License: MIT" /></a>
  <a href="https://solana.com"><img src="https://img.shields.io/badge/Solana-v1.18+-00D1A0.svg" alt="Solana" /></a>
  <a href="https://www.anchor-lang.com"><img src="https://img.shields.io/badge/Anchor-0.31.1-blue.svg" alt="Anchor" /></a>
  <a href="https://github.com/krishnagoyal099/Bastion-solana"><img src="https://img.shields.io/badge/Status-Live_on_Devnet-brightgreen.svg" alt="Status" /></a>
</div>

---

## The Problem

Every time you trade on a public DEX, bots can see your transaction before it lands on-chain. They front-run you, sandwich you, and extract value from your trade. This is called **MEV (Maximal Extractable Value)** — and it costs Solana traders millions.

## How Bastion Fixes It

Bastion is a **dark pool protocol** — your orders are encrypted using **Zero-Knowledge Proofs** before they enter the mempool. Nobody can see what you're trading, how much, or in which direction. The proof guarantees the order is valid without revealing anything about it.

| Problem | Bastion's Solution |
|:---|:---|
| **Sandwich attacks** | Orders are encrypted — bots can't see your trade size or direction |
| **Front-running** | Commitment-based ordering hides intent until execution |
| **Price impact on large orders** | Dark pool matching keeps whale trades off the public orderbook |
| **Privacy** | ZK proofs verify solvency without revealing account details |

---

## Quick Start

Install and launch the interactive terminal in two commands:

```bash
# Install the Bastion CLI
curl -fsSL https://raw.githubusercontent.com/krishnagoyal099/Bastion-solana/main/install-cli.sh | bash

# Launch the trading terminal
bastion
```

You can also run commands directly:
```bash
bastion trade                    # Open quick trade
bastion deposit 5                # Deposit 5 SOL to the dark pool
bastion withdraw 2               # Withdraw 2 SOL
bastion ticker                   # Live market prices
bastion config set-network devnet
```

---

## What's Inside

### On-Chain Programs (Solana Smart Contracts)

| Program | Address (Devnet) | Purpose |
|:---|:---|:---|
| `bastion-pool` | `CbHS6twCMkYyodaEUtvonRV6HVBZnkGjekohLqXJziU5` | ZK dark pool — commitments, proofs, matching, settlement |
| `bastion-amm` | `BvFgtfCEeCcMHoN1PRHSXkdzVYTka1NsrVBTeHmnDN2D` | Constant-product AMM — fallback liquidity when dark pool has no match |

### CLI Terminal UI

A full trading terminal built with `bash` and [`gum`](https://github.com/charmbracelet/gum):

```
[>]  Quick Trade          — Submit encrypted ZK orders
[$]  Deposit / Withdraw   — Move funds in and out of the dark pool
[~]  Liquidity Center     — Add/remove AMM liquidity
[%]  Live Market Ticker   — Real-time prices with sparklines
[<]  Arbitrage Scanner    — Detect cross-venue price gaps
[W]  Whale Mode           — Iceberg orders (split large trades automatically)
[!]  MEV Attack Simulator — See sandwich attacks vs. Bastion protection
[Z]  ZK Proof Demo        — Interactive walkthrough of how ZK commitments work
```

### SDK & Tooling

- **TypeScript SDK** (`sdk/`) — Programmatic access to dark pool and AMM
- **Relayer** (`relayer/`) — Off-chain order matching and Jito MEV bundle submission
- **Indexer** (`indexer/`) — Transaction parser for on-chain activity
- **ZK Circuits** (`zk/`) — Halo2-based proof generation compiled to WASM

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     BASTION CLI (bash/gum)                   │
│  Trade | Deposit | Withdraw | Liquidity | Ticker | ZK Demo  │
└───────────────────────┬─────────────────────────────────────┘
                        │
           ┌────────────▼────────────┐
           │    TypeScript SDK       │
           │    + Relayer Service    │
           └────────────┬────────────┘
                        │
           ┌────────────▼────────────┐
           │   Solana Programs       │
           │  ┌────────────────────┐ │
           │  │  bastion-pool      │ │  ZK Dark Pool
           │  │  bastion-amm       │ │  Constant Product AMM
           │  └────────────────────┘ │
           └────────────┬────────────┘
                        │
           ┌────────────▼────────────┐
           │  Solana Network         │
           │  localnet | devnet      │
           └─────────────────────────┘
```

---

## Build from Source

### Requirements

| Tool | Version | Install |
|:---|:---|:---|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| Solana CLI | 1.18+ | [docs.solana.com](https://docs.solana.com/cli/install-solana-cli-tools) |
| Anchor | 0.31.1 | [anchor-lang.com](https://www.anchor-lang.com/docs/installation) |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| gum | 0.14+ | [github.com/charmbracelet/gum](https://github.com/charmbracelet/gum) |

### Steps

```bash
# Clone
git clone https://github.com/krishnagoyal099/Bastion-solana.git
cd Bastion-solana

# Build smart contracts
anchor build

# Start local validator + deploy
solana-test-validator --reset &
anchor deploy

# Initialize the pool
npm install
npx ts-node scripts/init-pool.ts

# Launch the CLI
./cli/bastion
```

### Switch Networks

```bash
bastion config set-network devnet     # Use Solana devnet
bastion config set-network localnet   # Use local validator
bastion config show                   # View current config
```

---

## Security

- **Encrypted orders** — All trades are hidden behind SHA-256 commitments, validated via ZK proofs
- **No replay attacks** — Nullifier system prevents double-spend
- **Trustless escrow** — Pool funds live in PDA-derived vaults with no admin drain keys
- **MEV-immune by design** — Bots cannot extract value from data they cannot see

Full details: [audit_scope.md](./audit_scope.md) · [threat_model.md](./threat_model.md)

---

## Project Structure

```
bastion/
├── cli/                    # Interactive terminal UI
│   ├── bastion             # Main entry point
│   ├── lib/                # Feature modules (trade, liquidity, zkproof, etc.)
│   └── config/             # Network and contract configuration
├── programs/               # Solana smart contracts (Anchor/Rust)
│   ├── bastion-pool/       # ZK dark pool program
│   └── bastion-amm/        # Constant product AMM program
├── sdk/                    # TypeScript SDK for programmatic access
├── relayer/                # Off-chain matching engine (Rust)
├── indexer/                # On-chain transaction indexer
├── zk/                     # Halo2 ZK circuits + WASM prover
├── scripts/                # Deployment and initialization scripts
├── landing-page/           # React/Vite website (bastion-solana.vercel.app)
├── tests/                  # Integration and anchor tests
└── install-cli.sh          # One-line installer script
```

---

## Links

| | |
|:---|:---|
| **Website** | [bastion-solana.vercel.app](https://bastion-solana.vercel.app) |
| **GitHub** | [github.com/krishnagoyal099/Bastion-solana](https://github.com/krishnagoyal099/Bastion-solana) |
| **License** | [MIT](./LICENSE) |

---

<div align="center">
  <sub>Built for the Solana ecosystem.</sub>
</div>
