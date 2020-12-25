use crossbeam::channel::Sender;
use influxdb::{Client as Influx, WriteQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod canonical_chain;
mod network_probe;
mod network_topology;
mod pool_transaction;
mod reorganization;
mod tail_log;

pub use canonical_chain::{select_last_block_number_in_influxdb, CanonicalChain};
pub use network_probe::NetworkProbe;
pub use network_topology::NetworkTopology;
pub use pool_transaction::PoolTransaction;
pub use reorganization::Reorganization;
pub use tail_log::{Regex, TailLog};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Analyzer {
    CanonicalChain {
        ckb_rpc_url: String,
    },
    NetworkTopology {
        ckb_rpc_urls: Vec<String>,
    },
    Reorganization {
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
    },
    PoolTransaction {
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
    },
    TailLog {
        filepath: String,
        patterns: HashMap<String, Regex>,
    },
    NetworkProbe {
        ckb_network_identifier: String,
    },
}

impl Analyzer {
    pub async fn run(
        self,
        analyzer_name: String,
        ckb_network_name: String,
        influx: Influx,
        query_sender: Sender<WriteQuery>,
    ) {
        log::info!("{} starting ...", analyzer_name);
        match self {
            Self::CanonicalChain { ckb_rpc_url } => {
                let last_number =
                    select_last_block_number_in_influxdb(&influx, &ckb_network_name).await;
                CanonicalChain::new(&ckb_rpc_url, query_sender, last_number)
                    .run()
                    .await
            }
            Self::NetworkProbe {
                ckb_network_identifier,
            } => {
                NetworkProbe::new(ckb_network_name, ckb_network_identifier, query_sender)
                    .run()
                    .await
            }
            Self::NetworkTopology { ckb_rpc_urls } => {
                NetworkTopology::new(ckb_rpc_urls).run().await
            }
            Self::Reorganization {
                ckb_rpc_url,
                ckb_subscribe_url,
            } => {
                let (reorganization, subscription) =
                    Reorganization::new(ckb_rpc_url, ckb_subscribe_url, query_sender);

                // IMPORTANT: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                ::std::thread::spawn(move || {
                    jsonrpc_server_utils::tokio::run(subscription.run());
                });

                // // PROBLEM: With delaying a while, both tasks subscription and reorganization will run;
                // // But without delaying, only the task reorganization will run.
                // tokio::spawn(async { subscription.run().await });
                // tokio::time::delay_for(::std::time::Duration::from_secs(3)).await;

                reorganization.run().await;
            }
            Self::PoolTransaction {
                ckb_rpc_url,
                ckb_subscribe_url,
            } => {
                let (pool_transaction, subscription) =
                    PoolTransaction::new(ckb_rpc_url, ckb_subscribe_url, query_sender.clone());

                // IMPORTANT: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                ::std::thread::spawn(move || {
                    jsonrpc_server_utils::tokio::run(subscription.run());
                });
                pool_transaction.run().await;
            }
            Self::TailLog { filepath, patterns } => {
                // let patterns = patterns
                //     .into_iter()
                //     .map(|(category, regex)| {
                //         (
                //             category,
                //             Regex::new(&regex).unwrap_or_else(|err| {
                //                 panic!("invalid regex, str: \"{}\", err: {}", regex, err)
                //             }),
                //         )
                //     })
                //     .collect();
                let mut tail_log = TailLog::new(filepath, patterns, query_sender);
                ::std::thread::spawn(move || tail_log.run());
            }
        }
    }
}
