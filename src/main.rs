use crossbeam::channel::bounded;
use influxdb::Client;
use lazy_static::lazy_static;
use std::env::var;

mod app_config;
mod get_version;
mod network;
mod on_chain;
mod topology;

lazy_static! {
    static ref LOG_LEVEL: String = var("LOG_LEVEL").unwrap_or_else(|_| "ERROR".to_string());
    static ref CKB_URL: String = var("CKB_URL").unwrap_or_else(|_| "http://0.0.0.0:8114".to_string());
    static ref CKB_NETWORK: String = var("CKB_NETWORK").unwrap_or_else(|_|  panic!(
        "please specify network name via environment variable CKB_NETWORK, \"mainnet\" or \"testnet\""
    ));
    static ref CKB_NETWORK_IDENTIFIER: String = {
        match CKB_NETWORK.as_str() {
            "mainnet" => "/ckb/92b197aa".to_string(),
            "testnet" => "/ckb_testnet/10639e08".to_string(),
            _unknown => panic!("unknown ckb network, only support \"mainnet\" and \"testnet\"")
        }
    };
    static ref INFLUXDB_URL: String = var("INFLUXDB_URL").unwrap_or_else(|_| "http://0.0.0.0:8086".to_string());
    static ref INFLUXDB_DATABASE: String = CKB_NETWORK.clone();
    static ref NODE_ID: String = var("NODE_ID").unwrap_or_else(|_| panic!("please specify node id via environment variable NODE_ID"));
    static ref HOSTNAME: String = gethostname::gethostname().to_string_lossy().to_string();
}

#[tokio::main]
async fn main() {
    let client = Client::new(INFLUXDB_URL.as_str(), INFLUXDB_DATABASE.as_str());
    if let Err(err) = client.ping().await {
        eprintln!("client.ping error: {:?}", err);
        return;
    }
    let (query_sender, query_receiver) = bounded(5000);

    network::spawn_analyze(query_sender.clone());
    on_chain::spawn_analyze(query_sender.clone());
    topology::spawn_analyze(query_sender);

    for mut query in query_receiver {
        // Attach built-in tags
        query = query
            .add_tag("node_id", NODE_ID.clone())
            .add_tag("hostname", HOSTNAME.clone());

        let write_result = client.query(&query).await;
        if let Err(err) = write_result {
            eprintln!("influxdb.query({:?}, error: {:?}", query, err);
        }
    }
}
