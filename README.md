# soroban-telemetry

High-speed event listener and storage TTL monitor daemon for Stellar Soroban smart contracts.

## Capabilities
- **Event Streaming & Deduplication**: High-speed filtering by contract ID and topic hierarchies with LRU deduplication.
- **Storage TTL Monitor**: Real-time expiration tracking across Instance, Persistent, and Temporary storage with multi-tier alerts (Warning, Critical, Imminent Eviction).
- **Statistical Gas Surge Detector**: Rolling-window Z-score and ratio anomaly detection pinpointing potential DoS attacks or unexpected computational bursts.
- **HMAC-SHA256 Webhook Dispatcher**: Cryptographically signed alerting payload delivery to Discord, Slack, PagerDuty, or custom monitoring endpoints.

## CLI Commands
```bash
# Start daemon with contract filters and webhook dispatch
soroban-telemetry start --contract CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC --webhook-url https://api.myprotocol.com/webhook

# Run test simulation of TTL alert and gas surge
soroban-telemetry test-simulate --contract CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC
```
