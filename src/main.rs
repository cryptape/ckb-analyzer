use crate::topic::{
    CellCrawler, ChainCrawler, ChainTransactionCrawler, EpochCrawler, NetworkCrawler, PoolCrawler,
    RetentionTransactionCrawler, SubscribeNewTransaction, SubscribeProposedTransaction,
    SubscribeRejectedTransaction,
};
use crate::util::crossbeam_channel_to_tokio_channel;
use ckb_testkit::{connector::SharedState, ConnectorBuilder, Node};
use clap::{crate_version, value_t_or_exit, values_t_or_exit, App, Arg};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub use ckb_testkit::ckb_jsonrpc_types;
pub use ckb_testkit::ckb_types;

mod entry;
mod topic;
mod util;

#[tokio::main]
async fn main() {
    let _logger_guard = init_logger();
    log::info!("CKBAnalyzer starting");

    let matches = clap_app().get_matches();
    let rpc_url = value_t_or_exit!(matches, "node.rpc", String);
    let subscription_addr = value_t_or_exit!(matches, "node.subscription", SocketAddr);
    let topics = values_t_or_exit!(matches, "topics", String);
    log::info!(
        "CKBAnalyzer parameters: node.rpc={}, node.subscription={}, topics: {:?}",
        rpc_url,
        subscription_addr,
        topics
    );

    let pg_config = {
        let postgres_str = env::var_os("CKB_ANALYZER_POSTGRES")
            .expect("requires environment variable \"CKB_ANALYZER_POSTGRES\"")
            .to_string_lossy()
            .to_string();
        tokio_postgres::Config::from_str(&postgres_str)
            .expect("failed to parse \"CKB_ANALYZER_POSTGRES\"")
    };
    let pg = {
        let (pg, conn) = pg_config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                log::error!("postgres connection error: {}", err);
            }
        });
        pg
    };
    log::info!("connected to Postgres");

    // start handlers
    let (query_sender, mut query_receiver) =
        crossbeam_channel_to_tokio_channel::channel::<String>(5000);
    let node = Node::init_from_url(&rpc_url, PathBuf::new());
    let mut _connectors = Vec::new();
    for topic in topics {
        match topic.as_str() {
            "ChainCrawler" => {
                let last_block_number = {
                    match pg
                        .query_opt(
                            format!(
                                "SELECT number FROM {}.block ORDER BY time DESC LIMIT 1",
                                node.consensus().id,
                            )
                            .as_str(),
                            &[],
                        )
                        .await
                        .expect("query last block number")
                    {
                        None => 0,
                        Some(raw) => {
                            let number: i64 = raw.get(0);
                            number as u64
                        }
                    }
                };
                let handler = ChainCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(last_block_number).await;
                });
            }
            "EpochCrawler" => {
                let last_epoch_number = {
                    match pg
                        .query_opt(
                            format!(
                                "SELECT number FROM {}.epoch ORDER BY start_time DESC LIMIT 1",
                                node.consensus().id,
                            )
                            .as_str(),
                            &[],
                        )
                        .await
                        .expect("query last epoch number")
                    {
                        None => 0,
                        Some(raw) => {
                            let number: i64 = raw.get(0);
                            number as u64
                        }
                    }
                };
                let handler = EpochCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(last_epoch_number).await;
                });
            }
            "PoolCrawler" => {
                let handler = PoolCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run().await;
                });
            }
            "ChainTransactionCrawler" => {
                let last_block_number = {
                    match pg
                        .query_opt(
                            format!(
                                "SELECT number FROM {}.block_transaction ORDER BY time DESC LIMIT 1",
                                node.consensus().id,
                            )
                                .as_str(),
                            &[],
                        )
                        .await
                        .expect("query last block number")
                    {
                        None => 0,
                        Some(raw) => {
                            let number: i64 = raw.get(0);
                            number as u64
                        }
                    }
                };
                let handler = ChainTransactionCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(last_block_number).await;
                });
            }
            "SubscribeNewTransaction" => {
                let mut handler = SubscribeNewTransaction::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(subscription_addr.clone()).await;
                });
            }
            "SubscribeProposedTransaction" => {
                let mut handler =
                    SubscribeProposedTransaction::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(subscription_addr.clone()).await;
                });
            }
            "SubscribeRejectedTransaction" => {
                let mut handler =
                    SubscribeRejectedTransaction::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(subscription_addr.clone()).await;
                });
            }
            "RetentionTransactionCrawler" => {
                let handler = RetentionTransactionCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run().await;
                });
            }
            "CellCrawler" => {
                let last_cell_block_number = {
                    match pg
                        .query_opt(
                            format!(
                                "SELECT creating_number FROM {}.cell ORDER BY creating_time DESC LIMIT 1",
                                node.consensus().id,
                            )
                            .as_str(),
                            &[],
                        )
                        .await
                        .expect("query last block number")
                    {
                        None => 0,
                        Some(raw) => {
                            let number: i64 = raw.get(0);
                            number as u64
                        }
                    }
                };
                let handler = CellCrawler::new(node.clone(), query_sender.clone());
                tokio::spawn(async move {
                    handler.run(last_cell_block_number).await;
                });
            }
            "NetworkCrawler" => {
                let shared = Arc::new(RwLock::new(SharedState::new()));
                let network_crawler =
                    NetworkCrawler::new(node.clone(), query_sender.clone(), Arc::clone(&shared));
                // workaround for Rust lifetime
                _connectors.push(
                    ConnectorBuilder::new()
                        .protocol_metas(network_crawler.build_protocol_metas())
                        .listening_addresses(vec![])
                        .build(network_crawler, shared),
                );
            }
            _ => {
                ckb_testkit::error!("Unknown topic \"{}\"", topic);
                unreachable!()
            }
        }
    }

    // loop listen and batch execute queries
    let max_batch_size: usize = 200;
    let max_batch_timeout = Duration::from_secs(3);
    let mut batch: Vec<String> = Vec::with_capacity(max_batch_size);
    let mut last_batch_instant = Instant::now();
    while let Some(query) = query_receiver.recv().await {
        log::debug!("push new query: {}", query);
        batch.push(query);

        if batch.len() >= max_batch_size || last_batch_instant.elapsed() >= max_batch_timeout {
            log::debug!("batch_execute {} queries", batch.len());

            let batch_query: String = batch.join(";");
            pg.batch_execute(&batch_query).await.unwrap_or_else(|err| {
                log::error!("batch_execute(\"{}\"), error: {}", batch_query, err)
            });

            last_batch_instant = Instant::now();
            batch = Vec::new();
        }
    }
    log::info!("CKBAnalyzer shutdown");
}

fn init_logger() -> ckb_logger_service::LoggerInitGuard {
    let filter = match env::var("RUST_LOG") {
        Ok(filter) if filter.is_empty() => Some("info".to_string()),
        Ok(filter) => Some(filter.to_string()),
        Err(_) => Some("info".to_string()),
    };
    let config = ckb_logger_config::Config {
        filter,
        color: false,
        log_to_file: false,
        log_to_stdout: true,
        ..Default::default()
    };
    ckb_logger_service::init(None, config)
        .unwrap_or_else(|err| panic!("failed to init the logger service, error: {}", err))
}

pub fn clap_app() -> App<'static, 'static> {
    App::new("ckb-analyzer")
        .version(crate_version!())
        .arg(
            Arg::with_name("node.rpc")
                .long("node.rpc")
                .value_name("URL")
                .required(true)
                .takes_value(true)
                .validator(|s| {
                    url::Url::parse(&s)
                        .map(|_| ())
                        .map_err(|err| err.to_string())
                }),
        )
        .arg(
            Arg::with_name("node.subscription")
                .long("node.subscription")
                .value_name("SOCKET_ADDR")
                .required(true)
                .takes_value(true)
                .validator(|s| {
                    s.parse::<SocketAddr>()
                        .map(|_| ())
                        .map_err(|err| err.to_string())
                }),
        )
        .arg(
            Arg::with_name("topics")
                .long("topics")
                .value_name("TOPIC")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .default_value(
                    "ChainCrawler,\
                    EpochCrawler,\
                    PoolCrawler,\
                    ChainTransactionCrawler,\
                    SubscribeNewTransaction,\
                    SubscribeProposedTransaction,\
                    SubscribeRejectedTransaction,\
                    RetentionTransactionCrawler,\
                    CellCrawler,\
                    NetworkCrawler",
                )
                .possible_values(&[
                    "ChainCrawler",
                    "EpochCrawler",
                    "PoolCrawler",
                    "ChainTransactionCrawler",
                    "SubscribeNewTransaction",
                    "SubscribeProposedTransaction",
                    "SubscribeRejectedTransaction",
                    "RetentionTransactionCrawler",
                    "CellCrawler",
                    "NetworkCrawler",
                ]),
        )
}
