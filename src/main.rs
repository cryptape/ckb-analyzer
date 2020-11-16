use crossbeam::channel::bounded;
use influxdb::Client;
use lazy_static::lazy_static;
use std::env::var;

mod app_config;
mod get_version;
mod network;
mod on_chain;

lazy_static! {
    static ref LOG_LEVEL: String = var("LOG_LEVEL").unwrap_or("ERROR".to_string());
    static ref CKB_URL: String = var("CKB_URL").unwrap_or("http://0.0.0.0:8114".to_string());
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
    static ref INFLUXDB_URL: String = var("INFLUXDB_URL").unwrap_or("http://0.0.0.0:8086".to_string());
    static ref INFLUXDB_DATABASE: String = CKB_NETWORK.clone();
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
    on_chain::spawn_analyze(query_sender);

    for query in query_receiver {
        let write_result = client.query(&query).await;
        if LOG_LEVEL.as_str() != "ERROR" {
            println!("client.query(\"{:?}\")", query);
        }
        assert!(
            write_result.is_ok(),
            "client.query({:?}), error: {:?}",
            query,
            write_result.unwrap_err()
        );
    }
}
