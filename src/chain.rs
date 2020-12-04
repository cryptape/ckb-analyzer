use crate::serie::{BlockSerie, EpochSerie, TransactionSerie, UncleSerie};
use crate::CONFIG;
use crate::LOG_LEVEL;
use ckb_suite_rpc::Jsonrpc;
use ckb_types::core::{BlockNumber, HeaderView};
use ckb_types::core::{BlockView, EpochNumber};
use ckb_types::packed::{CellbaseWitness, ProposalShortId, Script};
use ckb_types::prelude::*;
use crossbeam::channel::Sender;
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

pub fn spawn_analyze(query_sender: Sender<WriteQuery>, last_number: BlockNumber) {
    spawn(move || analyze_blocks(query_sender, last_number));
}

fn analyze_blocks(query_sender: Sender<WriteQuery>, last_number: BlockNumber) {
    let from = last_number + 1;
    let mut number = max(1, from);

    let (window, mut proposals_zones) = {
        let window = (2u64, 10u64);
        let proposals_zones = HashMap::with_capacity(window.1 as usize);
        (window, proposals_zones)
    };

    let rpc = Jsonrpc::connect(CONFIG.chain.ckb_url.as_str());
    let mut tip = rpc.get_tip_block_number();
    let mut parent: BlockView = rpc
        .get_block_by_number(number.saturating_sub(1))
        .unwrap()
        .into();
    let (mut current_epoch_number, mut current_epoch_uncles_total) =
        { (parent.epoch().number(), 0) };
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
            analyze_epoch(
                &rpc,
                &mut current_epoch_number,
                &mut current_epoch_uncles_total,
                &block,
                &query_sender,
            );

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
            "[DEBUG] block #{}, timestamp: {}",
            number,
            block.timestamp(),
        );
    }
    query_sender.send(query).unwrap();
}

fn analyze_block_uncles(rpc: &Jsonrpc, block: &BlockView, query_sender: &Sender<WriteQuery>) {
    static QUERY_NAME: &str = "uncles";

    for uncle_view in block.uncles() {
        let uncle_number = uncle_view.number();
        let uncle_hash = uncle_view.hash();
        if let Some(json_uncle) = rpc.get_fork_block(uncle_hash.clone()) {
            let uncle: BlockView = json_uncle.into();
            let time = Timestamp::Milliseconds(uncle.timestamp() as u128);
            let proposals_count = uncle.union_proposal_ids().len() as u32;
            let transactions_count = uncle.transactions().len() as u32;
            let version = uncle.version();
            let miner_lock_args = extract_miner_lock(&uncle).args().to_string();
            let slower_than_cousin = {
                let cousin = rpc.get_header_by_number(uncle_number).unwrap().inner;
                cousin.timestamp.value() as i64 - uncle.timestamp() as i64
            };
            let query = UncleSerie {
                time,
                number: uncle_number,
                proposals_count,
                transactions_count,
                version,
                miner_lock_args,
                slower_than_cousin,
            }
            .into_query(QUERY_NAME);
            if LOG_LEVEL.as_str() != "ERROR" {
                println!(
                    "[DEBUG] uncle #{}({}), timestamp: {}, slower_than_cousin: {}",
                    uncle_number,
                    uncle.hash(),
                    uncle.timestamp(),
                    slower_than_cousin,
                );
            }
            query_sender.send(query).unwrap();
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

fn analyze_epoch(
    rpc: &Jsonrpc,
    current_epoch_number: &mut EpochNumber,
    current_epoch_uncles_total: &mut u32,
    block: &BlockView,
    query_sender: &Sender<WriteQuery>,
) {
    static QUERY_NAME: &str = "epochs";

    if *current_epoch_number != block.epoch().number() {
        let epoch = rpc.get_epoch_by_number(*current_epoch_number).unwrap();
        let start_number = epoch.start_number.value();
        let length = epoch.length.value();
        let end_number = start_number + length - 1;
        let start_header: HeaderView = rpc.get_header_by_number(start_number).unwrap().into();
        let end_header: HeaderView = rpc.get_header_by_number(end_number).unwrap().into();
        let duration = end_header
            .timestamp()
            .saturating_sub(start_header.timestamp());
        let query = EpochSerie {
            time: Timestamp::Milliseconds(end_header.timestamp() as u128),
            number: *current_epoch_number,
            length,
            duration,
            uncles_count: *current_epoch_uncles_total,
        }
        .into_query(QUERY_NAME);
        query_sender.send(query).unwrap();

        *current_epoch_number = block.epoch().number();
        *current_epoch_uncles_total = 0;
    }

    // Sum epoch uncles total
    *current_epoch_uncles_total += block.uncle_hashes().len() as u32;
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
