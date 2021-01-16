use crossbeam::channel::Sender;
use influxdb::{Client as Influx, WriteQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tentacle_multiaddr::Multiaddr;

mod canonical_chain_state;
mod network_propagation;
mod network_topology;
mod pattern_logs;
mod reorganization;
mod tx_transition;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "topic", content = "args")]
pub enum Topic {
    CanonicalChainState {
        ckb_rpc_url: String,
    },
    Reorganization {
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
    },
    TxTransition {
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
    },
    PatternLogs {
        filepath: String,
        patterns: HashMap<String, pattern_logs::Regex>,
    },
    NetworkPropagation {
        ckb_rpc_url: String,
        bootnodes: Vec<Multiaddr>,
    },
    NetworkTopology {
        ckb_rpc_urls: Vec<String>,
    },
}

impl Topic {
    pub async fn run(
        self,
        topic_name: String,
        ckb_network_name: String,
        influx: Influx,
        query_sender: Sender<WriteQuery>,
        async_handle: ckb_async_runtime::Handle,
    ) {
        log::info!("{} starting ...", topic_name);
        match self {
            Self::CanonicalChainState { ckb_rpc_url } => {
                let last_number = canonical_chain_state::select_last_block_number_in_influxdb(
                    &influx,
                    &ckb_network_name,
                )
                .await;
                canonical_chain_state::Handler::new(&ckb_rpc_url, query_sender, last_number)
                    .run()
                    .await
            }

            Self::NetworkPropagation {
                ckb_rpc_url,
                bootnodes,
            } => {
                // WARNING: As network service is synchronous, DON'T use async runtime.
                ::std::thread::spawn(move || {
                    network_propagation::Handler::new(ckb_rpc_url, bootnodes, query_sender)
                        .run(async_handle)
                });
            }

            Self::NetworkTopology { ckb_rpc_urls } => {
                network_topology::Handler::new(ckb_rpc_urls).run().await
            }

            Self::Reorganization {
                ckb_rpc_url,
                ckb_subscribe_url,
            } => {
                let (handler, subscription) =
                    reorganization::Handler::new(ckb_rpc_url, ckb_subscribe_url, query_sender);

                // WARNING: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                // ::std::thread::spawn(move || {
                //     jsonrpc_server_utils::tokio::run(subscription.run());
                // });
                jsonrpc_server_utils::tokio::spawn(subscription.run());

                // // PROBLEM: With delaying a while, both tasks subscription and reorganization will run;
                // // But without delaying, only the task reorganization will run.
                // tokio::spawn(async { subscription.run().await });
                // tokio::time::delay_for(::std::time::Duration::from_secs(3)).await;

                handler.run().await;
            }

            Self::TxTransition {
                ckb_rpc_url,
                ckb_subscribe_url,
            } => {
                let (handler, subscription) = tx_transition::Handler::new(
                    ckb_rpc_url,
                    ckb_subscribe_url,
                    query_sender.clone(),
                );

                // WARNING: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                // ::std::thread::spawn(move || {
                //     jsonrpc_server_utils::tokio::run(subscription.run());
                // });
                jsonrpc_server_utils::tokio::spawn(subscription.run());

                handler.run().await;
            }

            Self::PatternLogs { filepath, patterns } => {
                let mut handler =
                    pattern_logs::Handler::new(filepath, patterns, query_sender).await;
                handler.run().await;
            }
        }
    }
}
