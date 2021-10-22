use crate::topic::{PeerCollector, PeerScanner};
use crate::util::crossbeam_channel_to_tokio_channel;
use ckb_testkit::Node;
use clap::{crate_version, value_t_or_exit, values_t_or_exit, App, Arg};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

mod entry;
mod topic;
mod util;

fn main() {
    let (async_handle, _exit_handler) = ckb_async_runtime::new_global_runtime();
    async_handle.block_on(run(async_handle.clone()));
}

async fn run(async_handle: ckb_async_runtime::Handle) {
    let _logger_guard = init_logger();
    log::info!("ckb-analyzer starting");

    let matches = clap_app().get_matches();
    let rpc_url = value_t_or_exit!(matches, "node.rpc", String);
    let (query_sender, query_receiver) =
        crossbeam_channel_to_tokio_channel::channel::<String>(5000);
    let pg_config = {
        let postgres_str = env::var_os("CKB_ANALYZER_POSTGRES")
            .expect("requires environment variable \"CKB_ANALYZER_POSTGRES\"")
            .to_string_lossy()
            .to_string();
        tokio_postgres::Config::from_str(&postgres_str)
            .expect("failed to parse \"CKB_ANALYZER_POSTGRES\"")
    };
    // TODO default --topics is all
    let topics = values_t_or_exit!(matches, "topics", String);
    log::info!("Handler topics {:?}", topics);

    let pg = create_pg(&pg_config).await;
    for topic in topics {
        log::info!("Start topic {:?}", topic);
        match topic.as_str() {
            "PeerCollector" => {
                let node = Node::init_from_url(&rpc_url, PathBuf::new());
                let handler = PeerCollector::new(node, query_sender.clone(), async_handle.clone());
                thread::spawn(move || handler.run());
            }
            "PeerScanner" => {
                let mut handler = PeerScanner::new(&pg_config).await;
                ckb_async_runtime::tokio::spawn(async move {
                    handler.run().await;
                });
            }
            _ => unreachable!(),
        }
    }
    handle_message(&pg, query_receiver).await;
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

async fn handle_message(
    pg: &tokio_postgres::Client,
    mut query_receiver: ckb_async_runtime::tokio::sync::mpsc::Receiver<String>,
) {
    let max_batch_size: usize = 100;
    let max_batch_timeout = Duration::from_secs(3);
    let mut batch: Vec<String> = Vec::with_capacity(max_batch_size);
    let mut last_batch_instant = Instant::now();

    log::info!("bilibili handle_message start");
    while let Some(query) = query_receiver.recv().await {
        log::info!("bilibili handle_message query: {}", query);
        // TODO Attach built-in tags, hostname
        batch.push(query);
        if batch.len() >= max_batch_size || last_batch_instant.elapsed() >= max_batch_timeout {
            log::info!("bilibili handle_message insert: {}", batch.len());
            let batch_query: String = batch.join(";");
            pg.batch_execute(&batch_query).await.unwrap_or_else(|err| {
                panic!("pg.batch_execute(\"{}\"), error: {}", batch_query, err)
            });
            last_batch_instant = Instant::now();
            batch = Vec::new();
        }
    }
    log::info!("bilibili handle_message end");
}

async fn create_pg(pg_config: &tokio_postgres::Config) -> tokio_postgres::Client {
    let (pg, conn) = pg_config.connect(tokio_postgres::NoTls).await.unwrap();
    ckb_async_runtime::tokio::spawn(async move {
        if let Err(err) = conn.await {
            log::error!("postgres connection error: {}", err);
        }
    });
    pg
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
            Arg::with_name("topics")
                .long("topics")
                .value_name("TOPIC")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .possible_values(&["PeerCollector", "PeerScanner"]),
        )
}
