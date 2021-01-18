//! ckb-analyzer is an agent for collecting metrics from ckb, then writing the processed metrics
//! info InfluxDB. We can visualize these metrics on Grafana or other visualization tools.
//!
//! ckb-analyzer is still working in progress rapidly.
//!
//! # Install
//!
//! Download from [releases](https://github.com/keroro520/ckb-analyzer/releases) or
//!
//! ```shell
//! cargo install ckb-analyzer
//! ```
//!
//! # Usage
//!
//! ckb-analyzer reads several environment variables:
//!
//! * `CKB_ANALYZER_CONFIG` specify the configuration file path
//! * `CKB_RPC_USERNAME` specify the authorization username to ckb rpc service, default is `""`
//! * `CKB_RPC_PASSWORD` specify the authorization password to ckb rpc service, default is `""`
//! * `INFLUXDB_USERNAME` specify the influxdb username, default is `""`
//! * `INFLUXDB_PASSWORD` specify the influxdb password, default is `""`
//!
//! Command example:
//!
//! ```shell
//! CKB_ANALYZER_CONFIG=config/test.toml ckb-analyzer
//! ```
//!
//! # Topics and measurements
//!
//! * [ ] canonical chain
//!   - [x] block committed transactions and proposed transactions
//!   - [x] transactions per seconds
//!   - [x] block time interval
//!   - [x] epoch uncles count and uncles rate
//!   - [x] epoch duration and length
//!   - [ ] epoch adjustment
//!
//! * [x] canonical chain reorganization
//!   * [x] traffic
//!   * [x] scale
//!
//! * [ ] node's canonical chain growth
//!
//! * [ ] node's uncle blocks (some may not be included in canonical chain uncles)
//!   - [ ] traffic
//!
//! * [ ] miner
//!   - [ ] the miner node of a specified block (ip or lock args)
//!
//! * [ ] transaction transition (mainly focus the transaction traffic and state transition latency)
//!   - [x] pending
//!   - [x] pending too long
//!   - [x] propose
//!   - [x] propose too long
//!   - [x] commit
//!   - [x] remove (with reason, reject, conflict, and so forth)
//!   - [ ] reorganize
//!
//! * [ ] node's tx-pool state
//!   - [ ] pending/proposed pool size/cycles
//!
//! * [x] processed cost (via ckb internal metrics service)
//!   - [x] verify block
//!   - [x] verify transaction
//!
//! * [x] transaction and block propagation across the network
//!   - [x] the number of connected peers
//!   - [x] propagation elapsed
//!   - [x] high latency propagation
//!
//! * [x] logged events
//!   - [x] error and warning events
//!   - [x] sufficient events, recognize via regex patterning; better structure these logs
//!
//! # Debug suites
//!
//! * [ ] persist recent transactions (debug suite)
//!
//! * [ ] reproduce context
//!
//! # Monitoring alerts
//!
//! * [ ] datasource issues
//!   - [ ] no update for a long time
//!
//! * [ ] miner issues
//!   - [ ] chain does not grow up for too long
//!   - [ ] node receives too many uncle blocks
//!
//! * [ ] chain growth issues
//!   - [ ] block time interval is shorter/longer then threshold
//!   - [ ] a big epoch adjustment
//!
//! * [ ] transaction transition issues
//!   - [ ] too many transactions at a certain state
//!
//! * [ ] logged issues
//!
//! # Dashboards
//!
//! Please reference our Grafana dashboard files at [`dashboards`](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards)
//!
//! # FAQ
//!
//! * ckb itself exposes metrics. Then why create ckb-analyzer?
//!
//!   Some metrics are not convenient to expose from ckb, like historical chain metrics and complex
//!   analyzing tasks. With ckb-analyzer, we can display historical chain information by extracting
//!   the historical blocks and do some complexity tasks outside ckb, which prevent adding too much
//!   complexity into ckb.
//!
//! * Why use InfluxDB?
//!
//!   Pushing metrics actively via HTTP to InfluxDB is much useful!

use crate::config::{Config, Topic};
use crate::topic::{
    CanonicalChainState, NetworkPropagation, NetworkTopology, PatternLogs, Reorganization,
    TxTransition,
};
use crate::util::select_last_block_number_in_influxdb;
use crossbeam::channel::bounded;
use influxdb::Client as Influx;
use std::env::var;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

mod config;
mod dashboard;
mod measurement;
mod subscribe;
mod topic;
mod util;

fn main() {
    let (async_handle, _stop_handler) = ckb_async_runtime::new_global_runtime();
    async_handle.block_on(run(async_handle.clone()));
}

async fn run(async_handle: ckb_async_runtime::Handle) {
    init_logger();
    log::info!("ckb-analyzer starting");
    let config = init_config();
    let influx = init_influx(&config);
    let (query_sender, query_receiver) = bounded(5000);
    let node = config.node.clone();

    for topic in config.topics.iter() {
        log::info!("Start topic {:?}", topic);
        match topic {
            Topic::CanonicalChainState => {
                let last_number =
                    select_last_block_number_in_influxdb(&influx, &config.network).await;
                let mut handler =
                    CanonicalChainState::new(&node.rpc_url(), query_sender.clone(), last_number);
                async_handle.spawn(async move { handler.run().await });
            }
            Topic::Reorganization => {
                let (handler, subscription) = Reorganization::new(
                    node.rpc_url(),
                    node.subscription_url(),
                    query_sender.clone(),
                );

                // // WARNING: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                ::std::thread::spawn(move || {
                    jsonrpc_server_utils::tokio::run(subscription.run());
                });

                // jsonrpc_server_utils::tokio::spawn(subscription.run());

                // // PROBLEM: With delaying a while, both tasks subscription and reorganization will run;
                // // But without delaying, only the task reorganization will run.
                // async_handle.spawn(async { subscription.run().await });
                // tokio::time::delay_for(::std::time::Duration::from_secs(3)).await;

                async_handle.spawn(async move { handler.run().await });
            }
            Topic::TxTransition => {
                let (handler, subscription) = TxTransition::new(
                    node.rpc_url(),
                    node.subscription_url(),
                    query_sender.clone(),
                );

                // WARNING: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                // jsonrpc_server_utils::tokio::spawn(subscription.run());

                ::std::thread::spawn(move || {
                    jsonrpc_server_utils::tokio::run(subscription.run());
                });

                async_handle.spawn(async move { handler.run().await });
            }
            Topic::NetworkPropagation => {
                // WARNING: As network service is synchronous, DON'T use async runtime.
                let mut handler = NetworkPropagation::new(
                    node.rpc_url(),
                    node.bootnodes.clone(),
                    query_sender.clone(),
                );
                let async_handle_ = async_handle.clone();
                ::std::thread::spawn(move || handler.run(async_handle_));
            }
            Topic::NetworkTopology => {
                // TODO
                let handler = NetworkTopology::new(vec![]);
                async_handle.spawn(async move { handler.run().await });
            }
            Topic::PatternLogs => {
                let mut handler = PatternLogs::new(&node.data_dir, query_sender.clone()).await;
                async_handle.spawn(async move { handler.run().await });
            }
        }
    }

    let hostname = var("HOSTNAME")
        .unwrap_or_else(|_| gethostname::gethostname().to_string_lossy().to_string());
    let rate_limiter = Arc::new(AtomicU64::new(0));
    for mut query in query_receiver {
        // Attach built-in tags
        query = query
            .add_tag("network", config.network.clone())
            .add_tag("hostname", hostname.clone());

        // Writes asynchronously
        log::info!("{:?}", query);
        while rate_limiter.load(Ordering::Relaxed) >= 100 {
            tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
        }
        let rate_limiter_ = Arc::clone(&rate_limiter);
        let influx_ = influx.clone();
        async_handle.spawn(async move {
            rate_limiter_.fetch_add(1, Ordering::Relaxed);
            if let Err(err) = influx_.query(&query).await {
                log::error!("influxdb.query, error: {}", err);
            }
            rate_limiter_.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

fn init_logger() {
    simple_logger::SimpleLogger::from_env().init().unwrap();
}

fn init_config() -> Config {
    let config_path = var("CKB_ANALYZER_CONFIG").unwrap_or_else(|_| {
        panic!("please specify config path via environment variable CKB_ANALYZER_CONFIG")
    });
    let bytes = ::std::fs::read_to_string(config_path)
        .unwrap_or_else(|err| panic!("fs::read error: {:?}", err));
    toml::from_str(&bytes).unwrap_or_else(|err| panic!("toml::from_str error: {:?}", err))
}

fn init_influx(config: &Config) -> Influx {
    let username = var("INFLUXDB_USERNAME").unwrap_or_else(|_| "".to_string());
    let password = var("INFLUXDB_PASSWORD").unwrap_or_else(|_| "".to_string());
    let without_auth = username.is_empty();
    if without_auth {
        Influx::new(&config.influxdb.url, &config.influxdb.database)
    } else {
        Influx::new(&config.influxdb.url, &config.influxdb.database).with_auth(&username, &password)
    }
}
