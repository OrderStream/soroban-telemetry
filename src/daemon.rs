use crate::events::{EventFilter, EventStreamProcessor, SorobanEvent};
use crate::gas_surge::{GasMetrics, GasSurgeAlert, GasSurgeDetector};
use crate::ttl_monitor::{StorageKeyTtl, TtlExpiryAlert, TtlMonitor};
use crate::webhook::{WebhookConfig, WebhookDispatcher, WebhookPayload};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub events: Vec<SorobanEvent>,
    pub storage_keys: Vec<StorageKeyTtl>,
    pub gas_records: Vec<GasMetrics>,
    pub ledger_sequence: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    pub total_ledgers_processed: u32,
    pub total_events_matched: usize,
    pub total_ttl_alerts: usize,
    pub total_gas_surge_alerts: usize,
    pub total_webhooks_dispatched: usize,
}

pub struct TelemetryDaemon {
    event_processor: EventStreamProcessor,
    ttl_monitor: TtlMonitor,
    gas_detector: GasSurgeDetector,
    webhook_dispatcher: WebhookDispatcher,
    pub metrics: TelemetryMetrics,
}

impl TelemetryDaemon {
    pub fn new(
        event_filter: EventFilter,
        ttl_warning: u32,
        ttl_critical: u32,
        ttl_imminent: u32,
        gas_window: usize,
        gas_z_threshold: f64,
        webhook_config: WebhookConfig,
    ) -> Self {
        Self {
            event_processor: EventStreamProcessor::new(event_filter),
            ttl_monitor: TtlMonitor::new(ttl_warning, ttl_critical, ttl_imminent),
            gas_detector: GasSurgeDetector::new(gas_window, gas_z_threshold),
            webhook_dispatcher: WebhookDispatcher::new(webhook_config),
            metrics: TelemetryMetrics::default(),
        }
    }

    /// Process a telemetry batch for a given ledger.
    pub fn process_ledger_batch(
        &mut self,
        record: TelemetryRecord,
    ) -> (Vec<SorobanEvent>, Vec<TtlExpiryAlert>, Vec<GasSurgeAlert>, Vec<WebhookPayload>) {
        self.metrics.total_ledgers_processed += 1;

        // 1. Ingest events
        let mut matched_events = Vec::new();
        for ev in record.events {
            if let Some(matched) = self.event_processor.ingest(ev) {
                matched_events.push(matched);
                self.metrics.total_events_matched += 1;
            }
        }

        // 2. Track TTL entries & check expiries
        for key_entry in record.storage_keys {
            self.ttl_monitor.track_entry(key_entry);
        }
        let ttl_alerts = self.ttl_monitor.check_expiries(record.ledger_sequence);
        self.metrics.total_ttl_alerts += ttl_alerts.len();

        // 3. Evaluate Gas Surges
        let mut gas_alerts = Vec::new();
        for gas in record.gas_records {
            if let Some(surge) = self.gas_detector.evaluate(&gas) {
                gas_alerts.push(surge);
                self.metrics.total_gas_surge_alerts += 1;
            }
        }

        // 4. Dispatch Webhooks for all alerts
        let mut dispatches = Vec::new();

        for alert in &ttl_alerts {
            if let Ok(json) = serde_json::to_string(alert) {
                if let Ok(payload) = self.webhook_dispatcher.dispatch("TTL_EXPIRY", &json) {
                    dispatches.push(payload);
                    self.metrics.total_webhooks_dispatched += 1;
                }
            }
        }

        for surge in &gas_alerts {
            if let Ok(json) = serde_json::to_string(surge) {
                if let Ok(payload) = self.webhook_dispatcher.dispatch("GAS_SURGE", &json) {
                    dispatches.push(payload);
                    self.metrics.total_webhooks_dispatched += 1;
                }
            }
        }

        (matched_events, ttl_alerts, gas_alerts, dispatches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_full_cycle() {
        let filter = EventFilter::new().with_contract("CDLZ");
        let webhook_cfg = WebhookConfig {
            endpoint_url: "https://hooks.example.com/alerts".into(),
            hmac_secret: "my_key".into(),
            max_retries: 2,
            enabled: true,
        };

        let mut daemon = TelemetryDaemon::new(filter, 1000, 500, 100, 10, 2.5, webhook_cfg);

        // Batch 1: seed baseline gas & normal TTL
        let batch1 = TelemetryRecord {
            events: vec![SorobanEvent {
                contract_id: "CDLZ".into(),
                event_type: "contract".into(),
                topics: vec!["mint".into()],
                data_xdr_base64: "AAA=".into(),
                ledger_sequence: 100,
                tx_hash: "tx_1".into(),
                timestamp: 1700000000,
            }],
            storage_keys: vec![StorageKeyTtl {
                contract_id: "CDLZ".into(),
                key: "pool_reserve".into(),
                storage_type: "Persistent".into(),
                live_until_ledger: 50_000,
                last_checked_ledger: 100,
            }],
            gas_records: (0..6)
                .map(|i| GasMetrics {
                    contract_id: "CDLZ".into(),
                    tx_hash: format!("tx_{}", i),
                    cpu_instructions: 80_000,
                    memory_bytes: 512,
                    ledger_sequence: 100,
                    timestamp: 1700000000,
                })
                .collect(),
            ledger_sequence: 100,
        };

        let (events, ttl, gas, dispatches) = daemon.process_ledger_batch(batch1);
        assert_eq!(events.len(), 1);
        assert_eq!(ttl.len(), 0);
        assert_eq!(gas.len(), 0);
        assert_eq!(dispatches.len(), 0);

        // Batch 2: Impending TTL expiry and abnormal gas surge!
        let batch2 = TelemetryRecord {
            events: vec![],
            storage_keys: vec![StorageKeyTtl {
                contract_id: "CDLZ".into(),
                key: "critical_config".into(),
                storage_type: "Instance".into(),
                live_until_ledger: 105,
                last_checked_ledger: 101,
            }],
            gas_records: vec![GasMetrics {
                contract_id: "CDLZ".into(),
                tx_hash: "tx_exploit_attempt".into(),
                cpu_instructions: 950_000, // Massive spike!
                memory_bytes: 100_000,
                ledger_sequence: 101,
                timestamp: 1700000005,
            }],
            ledger_sequence: 101,
        };

        let (_, ttl2, gas2, dispatches2) = daemon.process_ledger_batch(batch2);
        assert_eq!(ttl2.len(), 1);
        assert_eq!(gas2.len(), 1);
        assert_eq!(dispatches2.len(), 2); // 1 TTL alert + 1 Gas Surge alert
        assert_eq!(daemon.metrics.total_webhooks_dispatched, 2);
    }
}
