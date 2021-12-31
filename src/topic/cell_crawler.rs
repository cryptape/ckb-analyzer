use crate::ckb_types::{
    core::{BlockNumber, BlockView},
    h256,
    packed::OutPoint,
    prelude::*,
};
use crate::entry;
use ckb_testkit::Node;
use std::cmp::max;
use std::convert::TryInto;
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

            let block = self.node.get_block_by_number(current_number);
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
        for (tx_index, tx) in block.transactions().iter().enumerate() {
            let tx_hash = tx.hash();
            if tx_index != 0 {
                for input in tx.input_pts_iter() {
                    let entry = entry::SpentCell {
                        network: self.node.consensus().id.clone(),
                        time,
                        block_number: block.number(),
                        out_point: input,
                    };
                    let raw_query = format!(
                        "INSERT INTO {}.spent_cell (time, block_number, tx_hash, index) \
                    VALUES ('{}', {}, '{:#x}', {})",
                        entry.network,
                        entry.time,
                        entry.block_number,
                        entry.out_point.tx_hash(),
                        Unpack::<u32>::unpack(&entry.out_point.index()),
                    );
                    queries.push(raw_query);
                }
            }

            for (index, output) in tx.outputs().into_iter().enumerate() {
                let out_point = OutPoint::new(tx_hash.clone(), index as u32);
                let entry = entry::CreatedCell {
                    network: self.node.consensus().id.clone(),
                    time,
                    block_number: block.number(),
                    tx_index,
                    out_point,
                    lock_hash_type: output.lock().hash_type().try_into().unwrap(),
                    lock_code_hash: output.lock().code_hash(),
                    lock_args: {
                        if output.lock().code_hash() == h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8").pack() {
                            if output.lock().args().raw_data().len() <= 48 {
                                Some(output.lock().args().raw_data())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    type_hash_type: output
                        .type_()
                        .to_opt()
                        .map(|script| script.hash_type().try_into().unwrap()),
                    type_code_hash: output.type_().to_opt().map(|script| script.code_hash()),
                };
                let raw_query = format!(
                    "INSERT INTO {}.created_cell (time, block_number, tx_index, tx_hash, index, lock_hash_type, lock_code_hash, lock_args, type_hash_type, type_code_hash) \
                    VALUES ('{}', {}, {}, '{:#x}', {}, {}, '{:#x}', '{}', {}, '{}')",
                    entry.network, entry.time, entry.block_number, entry.tx_index, entry.out_point.tx_hash(), Unpack::<u32>::unpack(&entry.out_point.index()),
                    Into::<u8>::into(entry.lock_hash_type),
                    entry.lock_code_hash,
                    entry.lock_args.map(|h| format!("{:#x}", h)).unwrap_or_default(),
                    entry.type_hash_type.map(|t| Into::<u8>::into(t)).unwrap_or(u8::max_value()),
                    entry.type_code_hash.map(|h| format!("{:#x}", h)).unwrap_or_default(),
                );
                queries.push(raw_query);
            }
        }

        let batch_query = queries.join(";");
        self.query_sender.send(batch_query).unwrap();
    }
}
