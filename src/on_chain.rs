use crate::{CKB_URL, LOG_LEVEL};
use ckb_suite_rpc::Jsonrpc;
use ckb_types::core::BlockNumber;
use ckb_types::core::BlockView;
use ckb_types::packed::{CellbaseWitness, ProposalShortId, Script};
use ckb_types::prelude::*;
use crossbeam::channel::Sender;
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

#[derive(InfluxDbWriteable)]
pub struct BlockSerie {
    time: Timestamp,

    number: u64,
    time_interval: u64, // ms
    transactions_count: u32,
    uncles_count: u32,
    proposals_count: u32,
    version: u32,

    #[tag]
    miner_lock_args: String,
}

#[derive(InfluxDbWriteable)]
pub struct UncleSerie {
    time: Timestamp,

    number: u64,
    transactions_count: u32,
    proposals_count: u32,
    version: u32,
    slower_than_cousin: i64,

    #[tag]
    miner_lock_args: String,
}

#[derive(InfluxDbWriteable)]
pub struct EpochSerie {
    time: Timestamp,

    length: u64,
    duration: u64, // seconds
}

#[derive(InfluxDbWriteable)]
pub struct TransactionSerie {
    time: Timestamp,

    number: u64, // block number
    pc_delay: u32,
}

pub fn spawn_analyze(query_sender: Sender<WriteQuery>) {
    let query_sender_ = query_sender.clone();
    spawn(move || analyze_blocks(query_sender_));
    spawn(move || analyze_epochs(query_sender));
}

fn analyze_epochs(query_sender: Sender<WriteQuery>) {
    let rpc = Jsonrpc::connect(CKB_URL.as_str());
    let current_epoch = rpc.get_current_epoch();
    for number in 0..current_epoch.number.value() {
        let epoch = rpc.get_epoch_by_number(number).unwrap();
        let length = epoch.length.value();
        let start_number: u64 = epoch.start_number.value();
        let end_number = start_number + length - 1;
        let start_header = rpc.get_header_by_number(start_number).unwrap();
        let end_header = rpc.get_header_by_number(end_number).unwrap();
        let start_timestamp = start_header.inner.timestamp.value() / 1000;
        let end_timestamp = end_header.inner.timestamp.value() / 1000;
        let write_query = EpochSerie {
            time: Timestamp::Seconds(start_timestamp as u128),
            length,
            duration: end_timestamp.saturating_sub(start_timestamp),
        }
        .into_query("epochs");
        query_sender.send(write_query).unwrap();
    }
}

fn analyze_blocks(query_sender: Sender<WriteQuery>) {
    let from = var("ANALYZE_ON_CHAIN_FROM")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(1);
    let mut number = from;
    assert!(from > 0);

    let (window, mut proposals_zones) = {
        let window = (2u64, 10u64);
        let proposals_zones = HashMap::with_capacity(window.1 as usize);
        (window, proposals_zones)
    };

    let rpc = Jsonrpc::connect(CKB_URL.as_str());
    let mut tip = rpc.get_tip_block_number();
    let mut parent: BlockView = rpc
        .get_block_by_number(number.saturating_sub(1))
        .unwrap()
        .into();
    loop {
        // Assume 10 is block confirmation
        if number >= tip - 10 {
            sleep(Duration::from_secs(1));
            tip = rpc.get_tip_block_number();
            continue;
        }

        if let Some(json_block) = rpc.get_block_by_number(number) {
            let block: BlockView = json_block.into();
            analyze_block(&block, &parent, &query_sender);
            analyze_block_uncles(&rpc, &block, &query_sender);
            analyze_block_transactions(&block, &window, &mut proposals_zones, &query_sender);

            number += 1;
            parent = block;
        }
    }
}

fn analyze_block(block: &BlockView, parent: &BlockView, query_sender: &Sender<WriteQuery>) {
    static QUERY_NAME: &str = "blocks";

    let time = Timestamp::Milliseconds(block.timestamp() as u128);
    let number = block.number();
    let time_interval = block.timestamp().saturating_sub(parent.timestamp()); // ms
    let transactions_count = block.transactions().len() as u32;
    let uncles_count = block.uncles().hashes().len() as u32;
    let proposals_count = block.union_proposal_ids().len() as u32;
    let version = block.version();
    let miner_lock_args = extract_miner_lock(&block).args().to_string();
    let query = BlockSerie {
        time,
        number,
        time_interval,
        transactions_count,
        uncles_count,
        proposals_count,
        version,
        miner_lock_args,
    }
    .into_query(QUERY_NAME);
    if LOG_LEVEL.as_str() != "ERROR" {
        println!(
            "[DEBUG] block #{}, miner: {}, timestamp: {}",
            number,
            extract_miner_lock(&block).args(),
            block.timestamp(),
        );
    }
    query_sender.send(query).unwrap();
}

fn analyze_block_uncles(rpc: &Jsonrpc, block: &BlockView, query_sender: &Sender<WriteQuery>) {
    static QUERY_NAME: &str = "uncles";

    for uncle_hash in block.uncle_hashes() {
        match rpc.get_fork_block(uncle_hash.clone()) {
            None => eprintln!("rpc.get_fork_block(\"{}\") return None", uncle_hash),
            Some(json_uncle) => {
                let uncle: BlockView = json_uncle.into();
                let number = uncle.number();
                let time = Timestamp::Milliseconds(uncle.timestamp() as u128);
                let proposals_count = uncle.union_proposal_ids().len() as u32;
                let transactions_count = uncle.transactions().len() as u32;
                let version = uncle.version();
                let miner_lock_args = extract_miner_lock(&uncle).args().to_string();
                let slower_than_cousin = {
                    let cousin = rpc.get_header_by_number(number).unwrap().inner;
                    cousin.timestamp.value() as i64 - uncle.timestamp() as i64
                };
                let query = UncleSerie {
                    time,
                    number,
                    proposals_count,
                    transactions_count,
                    version,
                    miner_lock_args,
                    slower_than_cousin,
                }
                .into_query(QUERY_NAME);
                if LOG_LEVEL.as_str() != "ERROR" {
                    println!(
                        "[DEBUG] uncle #{}({}), miner: {}, timestamp: {}, slower_than_cousin: {}",
                        number,
                        uncle.hash(),
                        extract_miner_lock(&uncle).args(),
                        uncle.timestamp(),
                        slower_than_cousin,
                    );
                }
                query_sender.send(query).unwrap();
            }
        }
    }
}

fn analyze_block_transactions(
    block: &BlockView,
    window: &(u64, u64),
    proposals_zones: &mut HashMap<BlockNumber, HashSet<ProposalShortId>>,
    query_sender: &Sender<WriteQuery>,
) {
    static QUERY_NAME: &str = "transactions";

    let number = block.number();
    for transaction in block.transactions() {
        let proposal_id = transaction.proposal_short_id();
        for proposed in number - window.1..=number - window.0 {
            let removed = proposals_zones
                .get_mut(&proposed)
                .map(|proposals_zone| proposals_zone.remove(&proposal_id))
                .unwrap_or(false);
            if removed {
                let time = Timestamp::Milliseconds(block.timestamp() as u128);
                let pc_delay = (number - proposed) as u32;
                let query = TransactionSerie {
                    time,
                    number,
                    pc_delay,
                }
                .into_query(QUERY_NAME);
                query_sender.send(query).unwrap();
                break;
            }
        }
    }

    // Prune outdated proposals zone
    proposals_zones.remove(&(number - window.1));
    proposals_zones.insert(number, block.union_proposal_ids());
}

#[allow(unused)]
fn prompt_progress(total: u64, processed: u64, start: Instant) {
    const PROMPT_STEP: u64 = 10000;

    if processed % PROMPT_STEP == 0 && processed > 0 {
        let left = total - processed;
        let processed_duration = start.elapsed();
        let left_duration = processed_duration
            .mul_f64(left as f64)
            .div_f64(processed as f64);
        let processed_percent = processed as f64 / total as f64;

        println!(
            "Progress {:.2}, left {}s ...",
            processed_percent,
            left_duration.as_secs()
        );
    }
}

fn extract_miner_lock(block: &BlockView) -> Script {
    let cellbase = block.transaction(0).unwrap();
    let witness = cellbase.witnesses().get(0).unwrap().raw_data();
    let cellbase_witness = CellbaseWitness::from_slice(witness.as_ref()).unwrap();
    cellbase_witness.lock()
}
