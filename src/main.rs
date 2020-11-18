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
}

#[tokio::main]
async fn main() {
    let client = Client::new(
        CONFIG.influxdb.url.as_str(),
        CONFIG.influxdb.database.as_str(),
    );
    if let Err(err) = client.ping().await {
        eprintln!("client.ping error: {:?}", err);
        return;
    }
    let (query_sender, query_receiver) = bounded(5000);

    network::spawn_analyze(query_sender.clone());
    chain::spawn_analyze(query_sender.clone());
    topology::spawn_analyze(query_sender);

    for mut query in query_receiver {
        // Attach built-in tags
        query = query
            .add_tag("node_id", CONFIG.influxdb.node_id.clone())
            .add_tag("hostname", CONFIG.influxdb.hostname.clone());

        let write_result = client.query(&query).await;
        if let Err(err) = write_result {
            eprintln!("influxdb.query({:?}, error: {:?}", query, err);
        }
    }
}
