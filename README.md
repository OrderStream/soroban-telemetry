# soroban-telemetry 📡🛡️

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![OrderStream Polar](https://img.shields.io/badge/Polar-OrderStream-blue.svg)](https://polar.sh/OrderStream)
[![Stellar XLM/USDC](https://img.shields.io/badge/Stellar-USDC%2FXLM-black.svg)](#direct-web3-settlement-wallets)
[![Base USDC](https://img.shields.io/badge/Base-USDC-0052FF.svg)](#direct-web3-settlement-wallets)

**High-speed event listener and storage TTL monitor daemon for Stellar Soroban smart contracts.**

---

## 🏛️ Executive Capabilities

- **Real-Time Storage TTL Sentinel**: Real-time expiration tracking across Instance, Persistent, and Temporary storage with multi-tier alerts (Warning, Critical, Imminent Eviction) before ledger state archival.
- **Statistical Gas Surge Detector**: Rolling-window Z-score and ratio anomaly detection pinpointing potential DoS attacks or unexpected computational bursts.
- **Event Streaming & Deduplication**: High-speed filtering by contract ID and topic hierarchies with LRU deduplication.
- **HMAC-SHA256 Webhook Dispatcher**: Cryptographically signed alerting payload delivery to Discord, Slack, PagerDuty, or custom monitoring endpoints.

---

## 💳 Enterprise Pricing & Protocol Retainers

| Tier | Price | Scope | Features |
| :--- | :--- | :--- | :--- |
| **Open Source / Free** | **$0** | Single Node | Standalone daemon CLI & simulation runner |
| **24/7 Telemetry Sentinel** | **$199 / mo** | 5 Contracts | Hosted cloud keeper, automated ledger polling, Discord/Slack alarms |
| **Protocol Retainer Tier** | **$1,499 / mo** | Up to 25 Contracts | Dedicated keeper infrastructure, auto-extend TTL bumps, gas surge alerts |
| **Enterprise Protocol Platform**| **$4,999 / mo** | Unlimited Contracts | 24/7 high-availability keeper nodes, 99.99% SLA, custom PagerDuty escalation |

---

## ⚡ Direct Web3 Settlement Wallets

OrderStream accepts direct crypto payments with **0% intermediary platform cuts**:

* **Stellar Protocol (USDC / XLM)**:  
  `GABFQIK63R2NETJM7T673EAMZN4RJLLGP3OFUEJU5ZVTGWUKULZJNL6`  
  *(Required Memo: `400566831`)*

* **EVM Base Network (USDC / ETH)**:  
  `0x5de2b62434754bfd39ef350a3a867ed3eced5074`

* **Credit Card & Invoicing**:  
  Subscribe online via [Polar.sh/OrderStream](https://polar.sh/OrderStream).

---

## 🚀 CLI Commands

```bash
# Build binary
cargo build --release

# Start daemon with contract filters and webhook dispatch
./target/release/soroban-telemetry start \
  --contract CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --webhook-url https://hub.orderstream.dev/api/telemetry/ingest

# Run test simulation of TTL alert and gas surge
./target/release/soroban-telemetry test-simulate \
  --contract CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```

---

## 🛡️ GitHub Action Integration

Verify storage TTL limits and alert pipelines in CI:

```yaml
- name: Soroban Telemetry Sentinel Check
  uses: OrderStream/soroban-telemetry@main
  with:
    contract_id: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
```
