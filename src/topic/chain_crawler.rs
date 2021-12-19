use crate::ckb_types::{
    core::{BlockNumber, BlockView},
    h256, packed,
    prelude::*,
};
use crate::entry;
use ckb_testkit::Node;
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

            current_number += 1;
        }
    }

    async fn analyze_block(&self, block: &BlockView) {
        let time = chrono::NaiveDateTime::from_timestamp(
            (block.timestamp() / 1000) as i64,
            (block.timestamp() % 1000 * 1000) as u32,
        );
        let interval = {
            let parent = self
                .node
                .rpc_client()
                .get_header(block.parent_hash())
                .unwrap();
            block
                .timestamp()
                .saturating_sub(parent.inner.timestamp.value())
        };
        let number = block.number();
        let n_transactions = block.transactions().len() as u32;
        let n_proposals = block.union_proposal_ids().len() as u32;
        let n_uncles = block.uncles().hashes().len() as u32;
        let miner_lock = extract_miner_lock_from_cellbase(block);
        let miner_lock_args = if miner_lock.code_hash()
            == h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8").pack()
        {
            if miner_lock.args().len() <= 48 {
                Some(miner_lock.args())
            } else {
                None
            }
        } else {
            None
        };
        let (cellbase_client_version, cellbase_miner_source) = extract_cellbase_message(block);
        let entry = entry::Block {
            network: self.node.consensus().id.clone(),
            time,
            number: number as i64,
            n_transactions: n_transactions as i32,
            n_proposals: n_proposals as i32,
            n_uncles: n_uncles as i32,
            cellbase_client_version,
            cellbase_miner_source,
            miner_lock_args: miner_lock_args
                .map(|arg| format!("{:#x}", arg))
                .unwrap_or_else(|| "-".to_string()),
            interval: interval as i64,
        };
        self.retry_send_entry_query(&entry).await;
    }

    async fn retry_send_entry_query(&self, entry: &entry::Block) {
        let query =
            format!(
                "INSERT INTO {}.block(time, number, n_transactions, n_proposals, n_uncles, miner_lock_args, cellbase_client_version, cellbase_miner_source, interval) \
            VALUES ('{}', {}, {}, {}, {}, '{}', '{}', '{}', {})",
                entry.network,
                entry.time,
                entry.number,
                entry.n_transactions,
                entry.n_proposals,
                entry.n_uncles,
                entry.miner_lock_args,
                entry.cellbase_client_version,
                entry.cellbase_miner_source,
                entry.interval,
        );
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

// When cellbase message contains 1 vector, it indicates miner identifier;
// When cellbase message contains more than one vectors, the first one indicates client version,
// the second one indicates miner identifier.
fn extract_cellbase_message(block: &BlockView) -> (String, String) {
    let cellbase = block.transaction(0).unwrap();
    let witness = cellbase.witnesses().get(0).unwrap().raw_data();
    let cellbase_witness = packed::CellbaseWitness::from_slice(witness.as_ref()).unwrap();
    if cellbase_witness.message().is_empty() {
        return ("-".to_string(), "-".to_string());
    }

    let raw_data = cellbase_witness.message().raw_data();
    let raw_string = String::from_utf8(raw_data.as_ref().to_vec())
        .map(|s| s.trim_matches('\0').to_string())
        .unwrap_or_default();
    let contents = raw_string.split(' ').collect::<Vec<_>>();
    if contents.is_empty() {
        return ("-".to_string(), "-".to_string());
    }

    let mut version = "-";
    let mut identifier = "-";
    for content in contents.iter() {
        if content.starts_with("0.") && version == "-" {
            version = content;
        } else if identifier == "-" {
            identifier = content;
        }
    }

    (version.to_string(), identifier.to_string())
}

fn extract_miner_lock_from_cellbase(block: &BlockView) -> packed::Script {
    let cellbase = block.transaction(0).unwrap();
    let witness = cellbase.witnesses().get(0).unwrap().raw_data();
    let cellbase_witness = packed::CellbaseWitness::from_slice(witness.as_ref()).unwrap();
    cellbase_witness.lock()
}
