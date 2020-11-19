use crossbeam::channel::bounded;
use influxdb::Client;
use lazy_static::lazy_static;
use std::env::var;

mod app_config;
mod chain;
mod config;
mod get_version;
mod network;
mod topology;

pub use config::{init_config, ChainConfig, Config, InfluxdbConfig, NetworkConfig, TopologyConfig};

lazy_static! {
    static ref LOG_LEVEL: String = var("LOG_LEVEL").unwrap_or_else(|_| "ERROR".to_string());
    static ref CONFIG: Config = {
        let config_path = var("CKB_ANALYZER_CONFIG").unwrap_or_else(|_| {
            panic!("please specify config path via environment variable CKB_ANALYZER_CONFIG")
        });
        init_config(config_path)
    };
    static ref INFLUXDB_USERNAME: String =
        var("INFLUXDB_USERNAME").unwrap_or_else(|_| "".to_string());
    static ref INFLUXDB_PASSWORD: String =
        var("INFLUXDB_PASSWORD").unwrap_or_else(|_| "".to_string());
    static ref HOSTNAME: String = gethostname::gethostname().to_string_lossy().to_string();
}

#[tokio::main]
async fn main() {
    let client = if INFLUXDB_USERNAME.is_empty() {
        Client::new(
            CONFIG.influxdb.url.as_str(),
            CONFIG.influxdb.database.as_str(),
        )
    } else {
        Client::new(
            CONFIG.influxdb.url.as_str(),
            CONFIG.influxdb.database.as_str(),
        )
        .with_auth(INFLUXDB_USERNAME.as_str(), INFLUXDB_PASSWORD.as_str())
    };
    let (query_sender, query_receiver) = bounded(5000);

    assert!(CONFIG.network.enabled || CONFIG.chain.enabled || CONFIG.topology.enabled);
    if CONFIG.network.enabled {
        network::spawn_analyze(query_sender.clone());
    }
    if CONFIG.chain.enabled {
        chain::spawn_analyze(query_sender.clone());
    }
    if CONFIG.topology.enabled {
        topology::spawn_analyze(query_sender);
    }

    for mut query in query_receiver {
        // Attach built-in tags
        query = query.add_tag("hostname", HOSTNAME.clone());

        let write_result = client.query(&query).await;
        if let Err(err) = write_result {
            eprintln!("influxdb.query, error: {}", err);
        }
    }
}
