use crate::ckb_types::{packed, prelude::Pack};
use crate::entry;
use ckb_testkit::Node;
use futures::stream::StreamExt;
use tokio::net::ToSocketAddrs;

pub struct SubscribeNewTransaction {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl SubscribeNewTransaction {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run<A: ToSocketAddrs>(&mut self, subscription_addr: A) {
        self.node.subscribe_new_transaction(subscription_addr).await;
        while let Some(Ok((_topic, pool_tx_entry))) =
            self.node.new_transaction_subscriber().next().await
        {
            let packed_tx: packed::Transaction = pool_tx_entry.transaction.inner.clone().into();
            let entry = entry::SubscribedNewTransaction {
                network: self.node.consensus().id.clone(),
                time: chrono::Utc::now().naive_utc(),
                size: pool_tx_entry.size.value(),
                cycles: pool_tx_entry.cycles.value(),
                fee: pool_tx_entry.fee.value(),
                n_inputs: pool_tx_entry.transaction.inner.inputs.len(),
                n_outputs: pool_tx_entry.transaction.inner.outputs.len(),
                n_cell_deps: pool_tx_entry.transaction.inner.cell_deps.len(),
                n_header_deps: pool_tx_entry.transaction.inner.header_deps.len(),
                hash: pool_tx_entry.transaction.hash.pack(),
                proposal_id: packed_tx.proposal_short_id(),
            };
            let raw_query = format!(
                "INSERT INTO {}.subscribed_new_transaction (time, size, cycles, fee, n_inputs, n_outputs, n_cell_deps, n_header_deps, hash, proposal_id) \
                VALUES ('{}', {}, {}, {}, {}, {}, {}, {}, '{:#x}', '{:#x}')",
                entry.network, entry.time, entry.size, entry.cycles, entry.fee, entry.n_inputs, entry.n_outputs, entry.n_cell_deps, entry.n_header_deps,
                entry.hash, entry.proposal_id,
            );
            self.query_sender.send(raw_query).unwrap();
        }
    }
}
