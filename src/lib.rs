//! High-speed event listener and storage TTL monitor daemon for Stellar Soroban.
//!
//! Provides on-chain contract event streaming, storage TTL expiry monitoring,
//! statistical gas surge anomaly detection, and HMAC-authenticated webhook dispatch.

pub mod daemon;
pub mod events;
pub mod gas_surge;
pub mod ttl_monitor;
pub mod webhook;

pub use daemon::{TelemetryDaemon, TelemetryMetrics, TelemetryRecord};
pub use events::{EventFilter, EventStreamProcessor, SorobanEvent};
pub use gas_surge::{GasMetrics, GasSurgeAlert, GasSurgeDetector};
pub use ttl_monitor::{StorageKeyTtl, TtlAlertLevel, TtlExpiryAlert, TtlMonitor};
pub use webhook::{WebhookConfig, WebhookDispatcher, WebhookPayload};
