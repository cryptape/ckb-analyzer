use crate::ckb_jsonrpc_types::{RawTxPool, TxPoolIds};
use crate::ckb_types::{prelude::*, H256};
use crate::entry;
use ckb_testkit::Node;
use std::collections::HashSet;
use std::time::Duration;

pub struct RetentionTransactionCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl RetentionTransactionCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self) {
        let mut last_observation_pool = HashSet::new();
        loop {
            match self.node.rpc_client().get_raw_tx_pool(Some(false)) {
                Err(err) => {
                    log::error!("RPC get_raw_tx_pool error: {:?}", err)
                }
                Ok(tx_pool) => match tx_pool {
                    RawTxPool::Ids(TxPoolIds { pending, proposed }) => {
                        let in_pool: HashSet<H256> = pending.into_iter().chain(proposed).collect();
                        let retention = last_observation_pool.intersection(&in_pool);
                        let now = chrono::Utc::now().naive_utc();
                        for hash in retention {
                            let entry = entry::RetentionTransaction {
                                network: self.node.consensus().id.clone(),
                                time: now,
                                hash: hash.pack(),
                            };
                            let raw_query = format!(
                                    "INSERT INTO {}.retention_transaction (time, hash) VALUES ('{}', '{:#x}')",
                                    entry.network, entry.time, entry.hash
                                );
                            self.query_sender.send(raw_query).unwrap();
                        }

                        last_observation_pool = in_pool;
                    }
                    RawTxPool::Verbose(_) => {
                        log::error!("RPC get_raw_tx_pool returns RawTxPool::Verbose")
                    }
                },
            }

            tokio::time::sleep(Duration::from_secs(60 * 10)).await;
        }
    }
}
