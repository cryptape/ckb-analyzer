use crate::config::Config;
use crate::table;
use crate::util::retry_send;
use ckb_suite_rpc::Jsonrpc;
use ckb_types::core::{BlockNumber, HeaderView};
use ckb_types::core::{BlockView, EpochNumber};
use ckb_types::packed::{CellbaseWitness, ProposalShortId, Script};
use ckb_types::prelude::*;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub const PROPOSAL_WINDOW: (u64, u64) = (2, 10);

pub struct CanonicalChainState {
    config: Config,
    query_sender: crossbeam::channel::Sender<String>,
    jsonrpc: Jsonrpc,
    start_number: BlockNumber,
    proposals_zones: HashMap<BlockNumber, HashSet<ProposalShortId>>,
}

impl CanonicalChainState {
    pub fn new(
        config: Config,
        jsonrpc: Jsonrpc,
        query_sender: crossbeam::channel::Sender<String>,
        start_number: BlockNumber,
    ) -> Self {
        Self {
            config,
            jsonrpc,
            query_sender,
            start_number,
            proposals_zones: Default::default(),
        }
    }

    pub async fn run(&mut self) {
        self.analyze_blocks().await
    }

    async fn analyze_blocks(&mut self) {
        let mut number = max(1, self.start_number + 1);
        let mut tip = self.jsonrpc.get_tip_block_number();
        let mut parent: BlockView = self
            .jsonrpc
            .get_block_by_number(number.saturating_sub(1))
            .unwrap()
            .into();
        let (mut current_epoch_number, mut current_epoch_uncles_total) =
            (parent.epoch().number(), 0);
        loop {
            // Assume 10 is block confirmation
            if number >= tip - 10 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                tip = self.jsonrpc.get_tip_block_number();
                continue;
            }

            if let Some(json_block) = self.jsonrpc.get_block_by_number(number) {
                let block: BlockView = json_block.into();
                self.analyze_block(&block, &parent).await;
                self.analyze_block_uncles(&block).await;
                self.analyze_block_transactions(&block).await;
                self.analyze_epoch(
                    &mut current_epoch_number,
                    &mut current_epoch_uncles_total,
                    &block,
                )
                .await;

                number += 1;
                parent = block;
            }
        }
    }

    async fn analyze_block(&self, block: &BlockView, parent: &BlockView) {
        let time = chrono::NaiveDateTime::from_timestamp(
            (block.timestamp() / 1000) as i64,
            (block.timestamp() % 1000 * 1000) as u32,
        );
        let number = block.number();
        let interval = block.timestamp() as i64 - parent.timestamp() as i64; // ms
        let n_transactions = block.transactions().len() as u32;
        let n_proposals = block.union_proposal_ids().len() as u32;
        let n_uncles = block.uncles().hashes().len() as u32;
        let version = block.version();
        let point = table::Block {
            network: self.config.network(),
            time,
            number: number as i64,
            interval: interval as i64,
            n_transactions: n_transactions as i32,
            n_proposals: n_proposals as i32,
            n_uncles: n_uncles as i32,
            version: version as i32,
            hash: format!("{:#x}", block.hash()),
            miner: format!("{:#x}", extract_miner_lock(&block).args()),
        };

        log::info!("block #{}, timestamp: {}", number, block.timestamp());
        retry_send(&self.query_sender, point.insert_query()).await;
    }

    async fn analyze_block_uncles(&self, block: &BlockView) {
        for uncle_hash in block.uncle_hashes() {
            if let Some(json_uncle) = self.jsonrpc.get_fork_block(uncle_hash) {
                self.analyze_block_uncle(&json_uncle.into()).await
            }
        }
    }

    async fn analyze_block_uncle(&self, uncle: &BlockView) {
        let uncle_number = uncle.number();
        let time = chrono::NaiveDateTime::from_timestamp(
            (uncle.timestamp() / 1000) as i64,
            (uncle.timestamp() % 1000 * 1000) as u32,
        );
        let n_transactions = uncle.transactions().len() as u32;
        let n_proposals = uncle.union_proposal_ids().len() as u32;
        let version = uncle.version();
        let lag_to_canonical = {
            let cousin = self
                .jsonrpc
                .get_header_by_number(uncle_number)
                .unwrap()
                .inner;
            cousin.timestamp.value() as i64 - uncle.timestamp() as i64
        };
        let point = table::Uncle {
            time,
            network: self.config.network(),
            number: uncle_number as i64,
            lag_to_canonical,
            n_transactions: n_transactions as i32,
            n_proposals: n_proposals as i32,
            version: version as i32,
            hash: format!("{:#x}", uncle.hash()),
            miner: format!("{:#x}", extract_miner_lock(&uncle).args()),
        };
        log::info!(
            "uncle #{}({:#x}), timestamp: {}, lag_to_canonical: {}",
            uncle_number,
            uncle.hash(),
            uncle.timestamp(),
            lag_to_canonical,
        );
        retry_send(&self.query_sender, point.insert_query()).await;
    }

    async fn analyze_block_transactions(&mut self, block: &BlockView) {
        let number = block.number();
        for transaction in block.transactions() {
            let proposal_id = transaction.proposal_short_id();
            for proposed_number in number - PROPOSAL_WINDOW.1..=number - PROPOSAL_WINDOW.0 {
                let removed = self
                    .proposals_zones
                    .get_mut(&proposed_number)
                    .map(|proposals_zone| proposals_zone.remove(&proposal_id))
                    .unwrap_or(false);
                if removed {
                    let time = chrono::NaiveDateTime::from_timestamp(
                        (block.timestamp() / 1000) as i64,
                        (block.timestamp() % 1000 * 1000) as u32,
                    );
                    let delay = number - proposed_number;
                    let point = table::TwoPCCommitment {
                        time,
                        network: self.config.network(),
                        number: number as i64,
                        delay: delay as i32,
                    };
                    retry_send(&self.query_sender, point.insert_query()).await;
                    break;
                }
            }
        }

        // Prune outdated proposals zone
        self.proposals_zones.remove(&(number - PROPOSAL_WINDOW.1));
        self.proposals_zones
            .insert(number, block.union_proposal_ids());
    }

    async fn analyze_epoch(
        &self,
        current_epoch_number: &mut EpochNumber,
        current_epoch_uncles_total: &mut u32,
        block: &BlockView,
    ) {
        if *current_epoch_number != block.epoch().number() {
            let epoch = self
                .jsonrpc
                .get_epoch_by_number(*current_epoch_number)
                .unwrap();
            let start_number = epoch.start_number.value();
            let length = epoch.length.value();
            let end_number = start_number + length - 1;
            let start_header: HeaderView = self
                .jsonrpc
                .get_header_by_number(start_number)
                .unwrap()
                .into();
            let end_header: HeaderView = self
                .jsonrpc
                .get_header_by_number(end_number)
                .unwrap()
                .into();
            let time = chrono::NaiveDateTime::from_timestamp(
                (end_header.timestamp() / 1000) as i64,
                (end_header.timestamp() % 1000 * 1000) as u32,
            );
            let duration = end_header
                .timestamp()
                .saturating_sub(start_header.timestamp());
            let point = table::Epoch {
                time,
                network: self.config.network(),
                number: *current_epoch_number as i64,
                length: length as i32,
                duration: duration as i32,
                n_uncles: *current_epoch_uncles_total as i32,
            };
            retry_send(&self.query_sender, point.insert_query()).await;

            *current_epoch_number = block.epoch().number();
            *current_epoch_uncles_total = 0;
        }

        // Sum epoch uncles total
        *current_epoch_uncles_total += block.uncle_hashes().len() as u32;
    }
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
