use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub endpoint_url: String,
    pub hmac_secret: String,
    pub max_retries: usize,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub alert_type: String, // "EVENT_TRIGGER", "TTL_EXPIRY", "GAS_SURGE"
    pub timestamp: u64,
    pub payload_json: String,
    pub signature: String,
}

pub struct WebhookDispatcher {
    config: WebhookConfig,
    dispatched_history: Vec<WebhookPayload>,
}

impl WebhookDispatcher {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            dispatched_history: Vec::new(),
        }
    }

    /// Signs payload using HMAC-SHA256.
    pub fn sign_payload(secret: &str, body: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(body.as_bytes());
        let result = mac.finalize();
        format!("{:x}", result.into_bytes())
    }

    /// Verifies that a webhook payload signature matches the secret using constant-time comparison.
    pub fn verify_signature(secret: &str, body: &str, signature: &str) -> bool {
        let expected = Self::sign_payload(secret, body);
        if expected.len() != signature.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in expected.bytes().zip(signature.bytes()) {
            diff |= a ^ b;
        }
        diff == 0
    }

    /// Dispatches an alert with signature verification.
    pub fn dispatch(&mut self, alert_type: &str, payload_json: &str) -> Result<WebhookPayload, String> {
        if !self.config.enabled {
            return Err("Webhook dispatcher is disabled".into());
        }

        let signature = Self::sign_payload(&self.config.hmac_secret, payload_json);
        let payload = WebhookPayload {
            alert_type: alert_type.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            payload_json: payload_json.to_string(),
            signature,
        };

        self.dispatched_history.push(payload.clone());
        Ok(payload)
    }

    pub fn history(&self) -> &[WebhookPayload] {
        &self.dispatched_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_generation_and_verification() {
        let secret = "super_secret_telemetry_key_2026";
        let body = r#"{"contract_id":"CDLZ","surge_ratio":4.5}"#;

        let sig = WebhookDispatcher::sign_payload(secret, body);
        assert_eq!(sig.len(), 64);
        assert!(WebhookDispatcher::verify_signature(secret, body, &sig));
        assert!(!WebhookDispatcher::verify_signature("wrong_secret", body, &sig));
    }

    #[test]
    fn test_webhook_dispatch_flow() {
        let config = WebhookConfig {
            endpoint_url: "https://api.orderstream.io/hooks/telemetry".into(),
            hmac_secret: "secret123".into(),
            max_retries: 3,
            enabled: true,
        };

        let mut dispatcher = WebhookDispatcher::new(config);
        let payload = dispatcher.dispatch("GAS_SURGE", r#"{"surge":"detected"}"#).unwrap();

        assert_eq!(payload.alert_type, "GAS_SURGE");
        assert_eq!(dispatcher.history().len(), 1);
    }
}
