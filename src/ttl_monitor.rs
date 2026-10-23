use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtlAlertLevel {
    Normal,
    Warning,           // TTL < warning_threshold
    Critical,          // TTL < critical_threshold
    ImminentEviction,  // TTL < imminent_threshold
    AlreadyEvicted,    // TTL <= 0 (Entry has expired/archived)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageKeyTtl {
    pub contract_id: String,
    pub key: String,
    pub storage_type: String, // "Instance", "Persistent", "Temporary"
    pub live_until_ledger: u32,
    pub last_checked_ledger: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlExpiryAlert {
    pub contract_id: String,
    pub key: String,
    pub storage_type: String,
    pub remaining_ledgers: u32,
    pub alert_level: TtlAlertLevel,
    pub current_ledger: u32,
    pub recommended_action: String,
}

pub struct TtlMonitor {
    tracked_entries: HashMap<String, StorageKeyTtl>,
    warning_threshold: u32,
    critical_threshold: u32,
    imminent_threshold: u32,
}

impl TtlMonitor {
    pub fn new(warning_threshold: u32, critical_threshold: u32, imminent_threshold: u32) -> Self {
        Self {
            tracked_entries: HashMap::new(),
            warning_threshold,
            critical_threshold,
            imminent_threshold,
        }
    }

    pub fn track_entry(&mut self, entry: StorageKeyTtl) {
        let unique_id = format!("{}:{}:{}", entry.contract_id, entry.storage_type, entry.key);
        self.tracked_entries.insert(unique_id, entry);
    }

    /// Evaluates all tracked storage entries against current ledger and emits alerts for entries below thresholds.
    pub fn check_expiries(&self, current_ledger: u32) -> Vec<TtlExpiryAlert> {
        let mut alerts = Vec::new();

        for entry in self.tracked_entries.values() {
            let is_expired = current_ledger >= entry.live_until_ledger;
            let remaining = entry.live_until_ledger.saturating_sub(current_ledger);

            let alert_level = if is_expired {
                TtlAlertLevel::AlreadyEvicted
            } else if remaining <= self.imminent_threshold {
                TtlAlertLevel::ImminentEviction
            } else if remaining <= self.critical_threshold {
                TtlAlertLevel::Critical
            } else if remaining <= self.warning_threshold {
                TtlAlertLevel::Warning
            } else {
                TtlAlertLevel::Normal
            };

            if alert_level != TtlAlertLevel::Normal {
                let rec = match alert_level {
                    TtlAlertLevel::AlreadyEvicted => {
                        match entry.storage_type.as_str() {
                            "Persistent" => "ARCHIVED: Persistent entry has expired on-chain. Cannot call extend_ttl (will revert). Must submit restore_footprint transaction.".into(),
                            "Temporary" => "PERMANENTLY EVICTED: Temporary storage entry expired and is unrecoverable.".into(),
                            _ => format!("EVICTED: State entry expired at ledger {}. Submit restore_footprint to restore.", entry.live_until_ledger),
                        }
                    }
                    TtlAlertLevel::ImminentEviction => {
                        format!("CRITICAL: Execute extend_ttl immediately before ledger {} to prevent state eviction/archival", entry.live_until_ledger)
                    }
                    TtlAlertLevel::Critical => {
                        format!("WARNING: Schedule extend_ttl transaction within {} ledgers", remaining)
                    }
                    TtlAlertLevel::Warning => {
                        format!("NOTICE: Storage TTL entering low reserve buffer ({} ledgers left)", remaining)
                    }
                    TtlAlertLevel::Normal => String::new(),
                };

                alerts.push(TtlExpiryAlert {
                    contract_id: entry.contract_id.clone(),
                    key: entry.key.clone(),
                    storage_type: entry.storage_type.clone(),
                    remaining_ledgers: remaining,
                    alert_level,
                    current_ledger,
                    recommended_action: rec,
                });
            }
        }

        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_expiry_alerts() {
        let mut monitor = TtlMonitor::new(1000, 500, 100);

        monitor.track_entry(StorageKeyTtl {
            contract_id: "CDLZ".into(),
            key: "admin_config".into(),
            storage_type: "Instance".into(),
            live_until_ledger: 1050,
            last_checked_ledger: 1000,
        });

        // Ledger 1000: remaining = 50 -> ImminentEviction
        let alerts = monitor.check_expiries(1000);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_level, TtlAlertLevel::ImminentEviction);
        assert_eq!(alerts[0].remaining_ledgers, 50);

        // Ledger 900: remaining = 150 -> Critical
        let alerts_900 = monitor.check_expiries(900);
        assert_eq!(alerts_900[0].alert_level, TtlAlertLevel::Critical);

        // Ledger 200: remaining = 850 -> Warning
        let alerts_200 = monitor.check_expiries(200);
        assert_eq!(alerts_200[0].alert_level, TtlAlertLevel::Warning);

        // Ledger 0: remaining = 1050 -> Normal (no alert)
        let alerts_0 = monitor.check_expiries(0);
        assert!(alerts_0.is_empty());

        // Ledger 1050+: already expired -> AlreadyEvicted
        let alerts_1055 = monitor.check_expiries(1055);
        assert_eq!(alerts_1055.len(), 1);
        assert_eq!(alerts_1055[0].alert_level, TtlAlertLevel::AlreadyEvicted);
        assert!(alerts_1055[0].recommended_action.contains("restore_footprint"));
    }
}
