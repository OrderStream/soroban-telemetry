use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorobanEvent {
    pub contract_id: String,
    pub event_type: String, // "contract", "system", "diagnostic"
    pub topics: Vec<String>,
    pub data_xdr_base64: String,
    pub ledger_sequence: u32,
    pub tx_hash: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EventFilter {
    pub contract_ids: HashSet<String>,
    pub topic_filters: Vec<String>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            contract_ids: HashSet::new(),
            topic_filters: Vec::new(),
        }
    }

    pub fn with_contract(mut self, contract_id: impl Into<String>) -> Self {
        self.contract_ids.insert(contract_id.into());
        self
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic_filters.push(topic.into());
        self
    }

    pub fn matches(&self, event: &SorobanEvent) -> bool {
        if !self.contract_ids.is_empty() && !self.contract_ids.contains(&event.contract_id) {
            return false;
        }

        if !self.topic_filters.is_empty() {
            let matched_any = self.topic_filters.iter().any(|filter| {
                event.topics.iter().any(|t| t == filter || t.starts_with(filter))
            });
            if !matched_any {
                return false;
            }
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventStreamProcessor {
    filter: EventFilter,
    processed_hashes: HashSet<String>,
    max_history: usize,
    history_queue: Vec<String>,
}

impl EventStreamProcessor {
    pub fn new(filter: EventFilter) -> Self {
        Self {
            filter,
            processed_hashes: HashSet::new(),
            max_history: 10_000,
            history_queue: Vec::new(),
        }
    }

    /// Ingests an event and returns it if it matches filters and is not a duplicate.
    pub fn ingest(&mut self, event: SorobanEvent) -> Option<SorobanEvent> {
        let unique_key = format!("{}:{}:{}", event.tx_hash, event.contract_id, event.ledger_sequence);

        if self.processed_hashes.contains(&unique_key) {
            return None; // Duplicate
        }

        if !self.filter.matches(&event) {
            return None; // Filtered out
        }

        if self.history_queue.len() >= self.max_history {
            if let Some(oldest) = self.history_queue.first().cloned() {
                self.history_queue.remove(0);
                self.processed_hashes.remove(&oldest);
            }
        }

        self.processed_hashes.insert(unique_key.clone());
        self.history_queue.push(unique_key);

        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filtering_and_deduplication() {
        let filter = EventFilter::new()
            .with_contract("CAAA")
            .with_topic("transfer");

        let mut processor = EventStreamProcessor::new(filter);

        let ev1 = SorobanEvent {
            contract_id: "CAAA".into(),
            event_type: "contract".into(),
            topics: vec!["transfer".into(), "user_1".into()],
            data_xdr_base64: "AAAA".into(),
            ledger_sequence: 100,
            tx_hash: "hash_1".into(),
            timestamp: 1700000000,
        };

        // Matching event
        assert!(processor.ingest(ev1.clone()).is_some());

        // Duplicate rejection
        assert!(processor.ingest(ev1).is_none());

        // Non-matching contract
        let ev2 = SorobanEvent {
            contract_id: "CBBB".into(),
            event_type: "contract".into(),
            topics: vec!["transfer".into()],
            data_xdr_base64: "AAAA".into(),
            ledger_sequence: 101,
            tx_hash: "hash_2".into(),
            timestamp: 1700000005,
        };
        assert!(processor.ingest(ev2).is_none());
    }
}
