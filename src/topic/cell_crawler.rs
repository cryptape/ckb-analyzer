use crate::ckb_types::core::{BlockNumber, BlockView};
use crate::ckb_types::prelude::*;
use crate::entry;
use ckb_testkit::Node;
use std::cmp::max;
use std::time::Duration;

const BLOCK_CONFIRMATION: BlockNumber = 10;

pub struct CellCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl CellCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self, last_cell_block_number: BlockNumber) {
        let mut current_number = max(1, last_cell_block_number + 1);
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
            self.analyze_block_cells(&block).await;

            current_number += 1;
        }
    }

    async fn analyze_block_cells(&self, block: &BlockView) {
        let time = chrono::NaiveDateTime::from_timestamp(
            (block.timestamp() / 1000) as i64,
            (block.timestamp() % 1000 * 1000) as u32,
        );
        let mut queries = Vec::new();
        for tx in block.transactions() {
            let tx_hash = tx.hash();
            for input in tx.input_pts_iter() {
                let index: usize = input.index().unpack();
                let update_raw_query = format!(
                    "UPDATE {}.cell SET consuming_time = '{}' WHERE tx_hash = '{:#x}' AND index = {}",
                    self.node.consensus().id,
                    time,
                    input.tx_hash(),
                    index,
                );
                queries.push(update_raw_query);
            }

            let mut index = 0;
            for output in tx.outputs() {
                let entry = entry::Cell {
                    network: self.node.consensus().id.clone(),
                    creating_time: time,
                    consuming_time: None,
                    creating_number: block.number(),
                    tx_hash: tx_hash.clone(),
                    index,
                    lock_code_hash: output.lock().code_hash(),
                    lock_args: {
                        if output.lock().args().total_size() > 100 {
                            None
                        } else {
                            Some(output.lock().args().raw_data())
                        }
                    },
                    type_code_hash: output.type_().to_opt().map(|script| script.code_hash()),
                };
                let raw_query = format!(
                    "INSERT INTO {}.cell (creating_time, creating_number, tx_hash, index, lock_code_hash, lock_args, type_code_hash) \
                    VALUES ('{}', {}, '{:#x}', {}, '{:#x}', '{}', '{}')",
                    entry.network, entry.creating_time, entry.creating_number, entry.tx_hash, entry.index, entry.lock_code_hash,
                    entry.lock_args.map(|h| format!("{:#x}", h)).unwrap_or_default(),
                    entry.type_code_hash.map(|h| format!("{:#x}", h)).unwrap_or_default(),
                );
                queries.push(raw_query);

                index += 1;
            }
        }

        let batch_query = queries.join(";");
        self.query_sender.send(batch_query).unwrap();
    }
}
