<div align="center">

<h1>Nodus Protocol — Core Engine</h1>

<p>The payment processing backbone of the Nodus Protocol ecosystem.<br/>Fast, composable, and built for the decentralized web.</p>

[![License: MIT](https://img.shields.io/badge/License-MIT-violet.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Status: Active](https://img.shields.io/badge/Status-Active-green.svg)]()

</div>

---

## What is the Core Engine?

The **Nodus Protocol Core Engine** is the settlement and routing layer that powers seamless, permissionless payments across the Nodus ecosystem. It abstracts away the complexity of multi-chain transactions so that users and developers can move value as easily as sending a message.

Whether you're building a checkout flow, a subscription service, or a cross-chain payment app, the Core Engine handles the heavy lifting — routing, validation, settlement, and confirmation.

---

## Features

- **One-click payments** — Customers initiate transfers without managing gas, bridges, or slippage manually.
- **Multi-chain routing** — Automatically selects the optimal path across supported Substrate networks (Aleph Zero, Astar, Shiden) to minimize cost and latency.
- **Instant settlement** — Transactions are confirmed and settled in seconds, not minutes.
- **Non-custodial** — The engine never holds user funds; all transfers go directly between parties.
- **Composable** — Drop the engine into any stack via a clean API and SDK.
- **Auditable** — Every payment produces an on-chain receipt, queryable at any time.

---

## How It Works

```
Customer initiates payment
        │
        ▼
 Core Engine receives request
        │
        ├─ Validates sender & recipient
        ├─ Selects optimal chain route
        ├─ Estimates & abstracts fees
        │
        ▼
 Transaction submitted on-chain
        │
        ▼
 Settlement confirmed + receipt emitted
        │
        ▼
 Merchant/recipient notified
```

---

## Getting Started

### Prerequisites

- Rust 1.80+ (2024 edition)
- Cargo
- An RPC endpoint for your target network

### Installation

```bash
git clone https://github.com/Nodus-protocol/Nodus-Protocol-Core-Engine.git
cd Nodus-Protocol-Core-Engine
cargo build
```

### Configuration

Copy the example environment file and fill in your values:

```bash
cp .env.example .env
```

| Variable | Description |
|---|---|
| `RPC_URL` | Substrate RPC endpoint for the target chain (e.g. Aleph Zero, Astar) |
| `PRIVATE_KEY` | SR25519 signing key (SS58 format) for the engine wallet |
| `SETTLEMENT_CONTRACT` | SS58 address of the deployed LiquidityPool contract |
| `NETWORK` | Target network (`mainnet`, `testnet`) |

### Running locally

```bash
cargo run
```

### Running tests

```bash
cargo test
```

---

## API Overview

### Initiate a payment

```http
POST /api/v1/pay
Content-Type: application/json

{
  "from": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "to": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
  "amount": "50.00",
  "currency": "USDC",
  "network": "aleph-zero"
}
```

**Response**

```json
{
  "status": "confirmed",
  "txHash": "0xabc123...",
  "settledAt": "2025-01-01T12:00:00Z",
  "fee": "0.001 USDC",
  "receipt": "ipfs://Qm..."
}
```

### Query a payment

```http
GET /api/v1/pay/:txHash
```

### Supported tokens

| Symbol | Network |
|---|---|
| AZERO | Aleph Zero |
| USDC | Aleph Zero (PSP22) |
| USDT | Aleph Zero (PSP22) |
| DOT | Astar, Shiden |
| ASTR | Astar |

---

## SDK

A JavaScript/TypeScript SDK is available for easy integration:

```ts
import { NodusEngine } from "@nodus/core-engine"

const engine = new NodusEngine({ network: "aleph-zero" })

const receipt = await engine.pay({
  from: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  to: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
  amount: "100",
  currency: "USDC",
})

console.log(receipt.txHash)
```

---

## Project Structure

```
Nodus-Protocol-Core-Engine/
├── src/
│   ├── engine/         # Core payment routing & settlement logic
│   ├── adapters/       # Chain-specific adapters (EVM, etc.)
│   ├── api/            # REST API handlers
│   └── utils/          # Helpers, fee estimation, validation
├── contracts/          # On-chain settlement contracts
├── tests/              # Unit and integration tests
└── docs/               # Extended documentation
```

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

1. Fork the repo
2. Create your branch: `git checkout -b feat/your-feature`
3. Commit your changes with a clear message
4. Push to your fork and open a PR against `main`

---

## Security

If you discover a vulnerability, please **do not** open a public issue. Contact the team privately at **security@nodusprotocol.io**.

---

## License

[MIT](LICENSE) © Nodus Protocol
