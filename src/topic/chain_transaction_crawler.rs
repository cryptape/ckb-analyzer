use crate::ckb_types::core::{BlockNumber, BlockView};
use crate::entry;
use ckb_testkit::Node;
use std::cmp::max;
use std::time::Duration;

const BLOCK_CONFIRMATION: BlockNumber = 10;

pub struct ChainTransactionCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl ChainTransactionCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self, last_block_number: BlockNumber) {
        let mut current_number = max(1, last_block_number);
        let mut tip_number = self.node.get_tip_block_number();
        loop {
            // Keep `BLOCK_CONFIRMATION` distance with node's tip
            if current_number >= tip_number - BLOCK_CONFIRMATION {
                tokio::time::sleep(Duration::from_secs(1)).await;
                tip_number = self.node.get_tip_block_number();
                continue;
            }

            let block = self.node.get_block_by_number(current_number);
            self.analyze_block_transactions(&block).await;

            current_number += 1;
        }
    }

    async fn analyze_block_transactions(&self, block: &BlockView) {
        let mut raw_queries = Vec::with_capacity(block.transactions().len());
        let time = chrono::NaiveDateTime::from_timestamp(
            (block.timestamp() / 1000) as i64,
            (block.timestamp() % 1000 * 1000) as u32,
        );
        for tx in block.transactions() {
            let size = tx.data().serialized_size_in_block();
            let n_inputs = tx.inputs().len();
            let n_outputs = tx.outputs().len();
            let n_header_deps = tx.header_deps().len();
            let n_cell_deps = tx.cell_deps().len();
            let total_data_size = tx.outputs_data().total_size();
            let entry = entry::BlockTransaction {
                time,
                network: self.node.consensus().id.clone(),
                number: block.number() as i64,
                size: size as i32,
                n_inputs: n_inputs as i32,
                n_outputs: n_outputs as i32,
                n_cell_deps: n_cell_deps as i32,
                n_header_deps: n_header_deps as i32,
                total_data_size: total_data_size as i32,
                proposal_id: format!("{:#x}", tx.proposal_short_id()),
                hash: format!("{:#x}", tx.hash()),
            };
            let raw_query = format!(
                "INSERT INTO {}.block_transaction(time, number, size, n_inputs, n_outputs, n_header_deps, n_cell_deps, total_data_size, proposal_id, hash) \
                VALUES ('{}', {}, {}, {}, {}, {}, {}, {}, '{}', '{}') \
                ON CONFLICT (number) DO NOTHING",
                entry.network,
                entry.time,
                entry.number,
                entry.size,
                entry.n_inputs,
                entry.n_outputs,
                entry.n_header_deps,
                entry.n_cell_deps,
                entry.total_data_size,
                entry.proposal_id,
                entry.hash,
            );
            raw_queries.push(raw_query);
        }

        self.query_sender.send(raw_queries.join(";")).unwrap();
    }
}
