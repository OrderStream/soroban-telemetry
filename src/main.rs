use clap::{Parser, Subcommand};
use soroban_telemetry::{
    EventFilter, GasMetrics, SorobanEvent, StorageKeyTtl, TelemetryDaemon, TelemetryRecord,
    WebhookConfig,
};

#[derive(Parser, Debug)]
#[command(
    name = "soroban-telemetry",
    author = "OrderStream <healthdecoded77@gmail.com>",
    version = "0.1.0",
    about = "High-speed event listener and storage TTL monitor daemon for Stellar Soroban"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Starts the telemetry listener daemon with given contract filters and webhook dispatch.
    Start {
        #[arg(short, long)]
        contract: Vec<String>,

        #[arg(long, default_value = "https://127.0.0.1:9000/webhook")]
        webhook_url: String,

        #[arg(long, default_value = "orderstream_secret_key")]
        secret: String,

        #[arg(long, default_value_t = 1000)]
        ttl_warning: u32,

        #[arg(long, default_value_t = 100)]
        ttl_imminent: u32,
    },

    /// Simulates a test ledger batch to test alerting and HMAC webhook signing.
    TestSimulate {
        #[arg(short, long, default_value = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC")]
        contract: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            contract,
            webhook_url,
            secret,
            ttl_warning,
            ttl_imminent,
        } => {
            println!("🚀 Initializing Soroban Telemetry Daemon...");
            println!("🎯 Tracking contracts: {:?}", contract);
            println!("🔗 Webhook Target: {}", webhook_url);

            let mut filter = EventFilter::new();
            for c in contract {
                filter = filter.with_contract(c);
            }

            let webhook_cfg = WebhookConfig {
                endpoint_url: webhook_url,
                hmac_secret: secret,
                max_retries: 3,
                enabled: true,
            };

            let _daemon = TelemetryDaemon::new(
                filter,
                ttl_warning,
                500,
                ttl_imminent,
                50,
                2.5,
                webhook_cfg,
            );
            println!("🛡️ Telemetry daemon ready and waiting for ledger streams.");
        }

        Commands::TestSimulate { contract } => {
            println!("🧪 Running telemetry simulation for contract: {}", contract);
            let filter = EventFilter::new().with_contract(&contract);
            let webhook_cfg = WebhookConfig {
                endpoint_url: "https://mock.stellar/hook".into(),
                hmac_secret: "audit_token".into(),
                max_retries: 3,
                enabled: true,
            };

            let mut daemon = TelemetryDaemon::new(
                filter, 1000, 500, 100, 10, 2.5, webhook_cfg,
            );

            // Seed normal entries
            let seed_record = TelemetryRecord {
                events: vec![SorobanEvent {
                    contract_id: contract.clone(),
                    event_type: "contract".into(),
                    topics: vec!["init".into()],
                    data_xdr_base64: "AAAA".into(),
                    ledger_sequence: 10,
                    tx_hash: "tx_init".into(),
                    timestamp: 1700000000,
                }],
                storage_keys: vec![],
                gas_records: (0..5)
                    .map(|i| GasMetrics {
                        contract_id: contract.clone(),
                        tx_hash: format!("tx_{}", i),
                        cpu_instructions: 50_000,
                        memory_bytes: 256,
                        ledger_sequence: 10,
                        timestamp: 1700000000,
                    })
                    .collect(),
                ledger_sequence: 10,
            };
            daemon.process_ledger_batch(seed_record);

            // Anomaly batch
            let anomaly_record = TelemetryRecord {
                events: vec![],
                storage_keys: vec![StorageKeyTtl {
                    contract_id: contract.clone(),
                    key: "governance_timelock".into(),
                    storage_type: "Persistent".into(),
                    live_until_ledger: 25,
                    last_checked_ledger: 15,
                }],
                gas_records: vec![GasMetrics {
                    contract_id: contract.clone(),
                    tx_hash: "tx_attack_surge".into(),
                    cpu_instructions: 600_000,
                    memory_bytes: 50_000,
                    ledger_sequence: 15,
                    timestamp: 1700000025,
                }],
                ledger_sequence: 15,
            };

            let (ev, ttl, gas, dispatches) = daemon.process_ledger_batch(anomaly_record);
            println!("📊 Processed Simulation Results:");
            println!("  - Matched Events: {}", ev.len());
            println!("  - Impending TTL Expiries: {}", ttl.len());
            println!("  - Abnormal Gas Surges: {}", gas.len());
            println!("  - Signed Webhooks Dispatched: {}", dispatches.len());
            for d in &dispatches {
                println!("    * [{}] HMAC Sig: {}", d.alert_type, d.signature);
            }
        }
    }

    Ok(())
}
