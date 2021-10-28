use crate::entry;
use crate::tokio;
use ckb_testkit::Node;
use ckb_types::core::{BlockNumber, BlockView};
use std::cmp::max;
use std::time::Duration;

const BLOCK_CONFIRMATION: BlockNumber = 10;

pub struct ChainCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl ChainCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self, last_block_number: BlockNumber) {
        let mut current_number = max(1, last_block_number + 1);
        let mut tip_number = self.node.get_tip_block_number();
        loop {
            // Keep `BLOCK_CONFIRMATION` distance with node's tip
            if current_number >= tip_number - BLOCK_CONFIRMATION {
                tokio::time::sleep(Duration::from_secs(1)).await;
                tip_number = self.node.get_tip_block_number();
                continue;
            }

            let json_block = self.node.get_block_by_number(current_number);
            let block: BlockView = json_block.into();
            self.analyze_block(&block).await;
            // self.analyze_block_uncles(&block).await;
            // self.analyze_block_transactions(&block).await;

            current_number += 1;
        }
    }

    async fn analyze_block(&self, block: &BlockView) {
        let time = chrono::NaiveDateTime::from_timestamp(
            (block.timestamp() / 1000) as i64,
            (block.timestamp() % 1000 * 1000) as u32,
        );
        let number = block.number();
        let n_transactions = block.transactions().len() as u32;
        let n_proposals = block.union_proposal_ids().len() as u32;
        let n_uncles = block.uncles().hashes().len() as u32;
        let entry = entry::Block {
            network: self.node.consensus().id.clone(),
            time,
            number: number as i64,
            n_transactions: n_transactions as i32,
            n_proposals: n_proposals as i32,
            n_uncles: n_uncles as i32,
            cellbase_message: None,
            interval: None,
        };
        self.retry_send_entry_query(&entry).await;
    }

    async fn retry_send_entry_query(&self, entry: &entry::Block) {
        let query = if let Some(ref cellbase_message) = entry.cellbase_message {
            log::debug!(
                "block #{} Some({}) {:?}",
                entry.number,
                cellbase_message,
                cellbase_message
            );
            format!(
            "INSERT INTO {}.block(time, number, n_transactions, n_proposals, n_uncles, cellbase_message) \
            VALUES ('{}', {}, {}, {}, {}, '{}')",
            entry.network,
            entry.time,
            entry.number,
            entry.n_transactions,
            entry.n_proposals,
            entry.n_uncles,
            cellbase_message,
        )
        } else {
            log::debug!("block #{} None", entry.number);
            format!(
                "INSERT INTO {}.block(time, number, n_transactions, n_proposals, n_uncles) \
            VALUES ('{}', {}, {}, {}, {})",
                entry.network,
                entry.time,
                entry.number,
                entry.n_transactions,
                entry.n_proposals,
                entry.n_uncles,
            )
        };
        loop {
            match self.query_sender.send(query.clone()) {
                Ok(_) => return,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}
