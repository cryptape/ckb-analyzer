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
//! * [ ] network distribution
//!   * [ ] tip distribution accross the network
//!   * [ ] version distribution accross the network
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
//! * [ ] network fork, there are nodes with different block hash on the same block number
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
    CanonicalChainState, NetworkPropagation, NetworkTopology, Reorganization, TxTransition,
};
use crate::util::get_last_updated_block_number;
use ckb_suite_rpc::Jsonrpc;
use crossbeam::channel::bounded;
use jsonrpc_server_utils::tokio as tokio01;
use std::env::var;
use std::time::{Duration, Instant};

mod config;
mod dashboard;
mod subscribe;
mod table;
mod topic;
mod util;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("ckb-analyzer-runtime")
        .build()
        .expect("ckb runtime initialized");
    let (async_handle02, _stop_handler) = ckb_async_runtime::new_global_runtime();
    runtime.block_on(run(async_handle02));
}

async fn run(async_handle02: ckb_async_runtime::Handle) {
    init_logger();
    log::info!("ckb-analyzer starting");
    let config = init_config();
    let (query_sender, query_receiver) = bounded::<String>(5000);
    let pg = create_pg(config.postgres()).await;

    for topic in config.topics.iter() {
        let topic = *topic;
        log::info!("Start topic {:?}", topic);
        match topic {
            Topic::CanonicalChainState => {
                let jsonrpc = Jsonrpc::connect(&config.rpc_url());
                let last_number = get_last_updated_block_number(&pg, &config.network).await;
                let mut handler = CanonicalChainState::new(
                    config.clone(),
                    jsonrpc,
                    query_sender.clone(),
                    last_number,
                );
                tokio::spawn(async move {
                    handler.run().await;
                    log::info!("End topic {:?}", topic);
                });
            }
            Topic::Reorganization => {
                let jsonrpc = Jsonrpc::connect(&config.rpc_url());
                let (handler, subscription) =
                    Reorganization::new(config.clone(), jsonrpc, query_sender.clone());

                ::std::thread::spawn(move || {
                    let mut runtime01 = tokio01::runtime::Builder::new().build().unwrap();
                    runtime01.block_on(subscription.run()).unwrap();
                    log::info!("Runtime for subscription on topic {:?} exit", topic);
                });

                tokio::spawn(async move {
                    handler.run().await;
                    log::info!("End topic {:?}", topic);
                });
            }
            Topic::TxTransition => {
                let jsonrpc = Jsonrpc::connect(&config.rpc_url());
                let (handler, subscription) =
                    TxTransition::new(config.clone(), jsonrpc, query_sender.clone());

                ::std::thread::spawn(move || {
                    let mut runtime01 = tokio01::runtime::Builder::new().build().unwrap();
                    runtime01.block_on(subscription.run()).unwrap();
                    log::info!("Runtime for subscription on topic {:?} exit", topic);
                });

                tokio::spawn(async move {
                    handler.run().await;
                    log::info!("End topic {:?}", topic);
                });
            }
            Topic::NetworkPropagation => {
                let jsonrpc = Jsonrpc::connect(&config.rpc_url());
                let mut handler = NetworkPropagation::new(
                    config.clone(),
                    jsonrpc,
                    query_sender.clone(),
                    async_handle02.clone(),
                );
                ::std::thread::spawn(move || handler.run());
            }
            Topic::NetworkTopology => {
                // TODO NetworkTopology
                let handler = NetworkTopology::new(vec![]);
                tokio::spawn(async move {
                    handler.run().await;
                    log::info!("End topic {:?}", topic);
                });
            }
        }
    }

    let mut batch: Vec<String> = Vec::new();
    let mut last_batch_instant: Instant = Instant::now();
    loop {
        handle_message(&pg, &query_receiver, &mut batch, &mut last_batch_instant).await;
    }
}

async fn handle_message(
    pg: &tokio_postgres::Client,
    query_receiver: &crossbeam::channel::Receiver<String>,
    batch: &mut Vec<String>,
    last_batch_instant: &mut Instant,
) {
    let max_batch_size: usize = 100;
    let max_batch_timeout = Duration::from_secs(60);
    match query_receiver.try_recv() {
        Ok(query) => {
            // TODO Attach built-in tags, hostname
            batch.push(query);
            if batch.len() >= max_batch_size || last_batch_instant.elapsed() >= max_batch_timeout {
                let batch_query: String = batch.join(";");
                pg.batch_execute(&batch_query).await.unwrap_or_else(|err| {
                    panic!("pg.batch_execute(\"{}\"), error: {}", batch_query, err)
                });
                *last_batch_instant = Instant::now();
                *batch = Vec::new();
            }
        }
        Err(crossbeam::channel::TryRecvError::Empty) => {
            if !batch.is_empty() {
                let batch_query: String = batch.join(";");
                pg.batch_execute(&batch_query).await.unwrap_or_else(|err| {
                    panic!("pg.batch_execute(\"{}\"), error: {}", batch_query, err)
                });
                *last_batch_instant = Instant::now();
                *batch = Vec::new();
            }
        }
        Err(crossbeam::channel::TryRecvError::Disconnected) => {
            log::info!("exit ckb-analyzer");
            ::std::process::exit(0);
        }
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

async fn create_pg(pg_config: tokio_postgres::Config) -> tokio_postgres::Client {
    let (pg, conn) = pg_config.connect(tokio_postgres::NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            log::error!("postgres connection error: {}", err);
        }
    });
    pg
}
