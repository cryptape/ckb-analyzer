use crate::entry;
use ckb_testkit::Node;
use std::time::Duration;

pub struct PoolCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl PoolCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self) {
        loop {
            let tx_pool_info = self.node.rpc_client().tx_pool_info();
            let entry = entry::TxPoolInfo {
                network: self.node.consensus().id.clone(),
                time: chrono::Utc::now().naive_utc(),
                total_tx_cycles: tx_pool_info.total_tx_cycles.value() as i64,
                total_tx_size: tx_pool_info.total_tx_size.value() as i64,
                pending: tx_pool_info.pending.value() as i64,
                proposed: tx_pool_info.proposed.value() as i64,
                orphan: tx_pool_info.orphan.value() as i64,
            };
            let raw_query = format!(
                "INSERT INTO {}.tx_pool_info(time, total_tx_cycles, total_tx_size, pending, proposed, orphan) \
            VALUES ('{}', {}, {}, {}, {}, {})",
                entry.network, entry.time, entry.total_tx_cycles, entry.total_tx_size, entry.pending, entry.proposed, entry.orphan
            );
            self.query_sender.send(raw_query).unwrap();

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
