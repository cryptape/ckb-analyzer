use crate::config::Config;
use crate::subscribe::{Subscription, Topic};
use crate::util::retry_send;
use ckb_suite_rpc::{
    ckb_jsonrpc_types::{PoolTransactionEntry, Status},
    Jsonrpc,
};
use ckb_types::{packed::Byte32, prelude::*};
use jsonrpc_core::serde_from_str;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// This module aims to the pool transactions
///
/// ### Note
///
/// * A transaction enters the pool, then becomes committed or removed, or suspended forever.
/// * Subscribe "NewBlock", so that we can know the committed transactions.
///
/// * TODO
///   - Retrieve all pool transactions via RPC `get_raw_tx_pool` at the start.
///   - Fix the issue that it takes too long to travel all the pending/proposed transactions.
///   - Enable subscription at ckb

const SUSPEND_SECONDS: i64 = 3 * 60;

pub(crate) struct TxTransition {
    entries: HashMap<Byte32, TxEntry>,
    subscriber: crossbeam::channel::Receiver<(Topic, String)>,
    config: Config,
    jsonrpc: Jsonrpc,
    query_sender: crossbeam::channel::Sender<String>,
    last_checking_at: Instant,
}

impl TxTransition {
    pub(crate) fn new(
        config: Config,
        jsonrpc: Jsonrpc,
        query_sender: crossbeam::channel::Sender<String>,
    ) -> (Self, Subscription) {
        let (subscription, subscriber) =
            Subscription::new(config.subscription_url(), Topic::NewTransaction);
        (
            Self {
                config,
                jsonrpc,
                query_sender,
                subscriber,
                last_checking_at: Instant::now(),
                entries: Default::default(),
            },
            subscription,
        )
    }

    async fn try_recv(&self) -> Result<(Topic, String), crossbeam::channel::TryRecvError> {
        self.subscriber.try_recv()
    }

    pub(crate) async fn run(mut self) {
        loop {
            match self.try_recv().await {
                Ok((_, message)) => {
                    let time = chrono::Utc::now().naive_utc();
                    let pool_transaction_entry =
                        serde_from_str::<PoolTransactionEntry>(&message).unwrap();
                    let txhash = pool_transaction_entry.transaction.hash.pack();
                    if !self.entries.contains_key(&txhash) {
                        let entry = TxEntry {
                            enter_time: time,
                            pool_transaction_entry,
                        };
                        self.report_enter(&entry).await;
                        self.entries.insert(txhash, entry);
                    }
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => return,
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    // TODO 分批检查，或者更高效的方式
                    if self.last_checking_at.elapsed() >= ::std::time::Duration::from_secs(60) {
                        self.check().await;
                        self.last_checking_at = Instant::now();
                    } else {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    async fn check(&mut self) {
        let now = chrono::Utc::now().naive_utc();
        let mut to_remove_hashes = Vec::new();
        for (txhash, entry) in self.entries.iter() {
            if (now - entry.enter_time).num_seconds() > SUSPEND_SECONDS {
                to_remove_hashes.push(txhash.clone());
            }
        }
        let mut to_remove_entries = Vec::with_capacity(to_remove_hashes.len());
        for txhash in to_remove_hashes.iter() {
            to_remove_entries.push(self.entries.remove(txhash).unwrap());
        }

        for entry in to_remove_entries.iter() {
            self.check_entry(entry).await;
        }
    }

    async fn check_entry(&self, entry: &TxEntry) {
        let txhash = entry.pool_transaction_entry.transaction.hash.pack();
        let txstatus = self.jsonrpc.get_transaction(txhash);
        match txstatus.map(|s| {
            (
                s.tx_status.status,
                s.tx_status.block_hash.map(|hash| hash.pack()),
            )
        }) {
            Some((Status::Committed, Some(block_hash))) => {
                if let Some(timestamp) = self.get_block_timestamp(block_hash) {
                    self.report_commit(entry, timestamp).await;
                }
            }
            Some((_status, None)) => {
                return;
            }
            None => {
                self.report_remove(entry).await;
            }
            _ => unreachable!(),
        }
    }

    async fn report_enter(&self, entry: &TxEntry) {
        let point = crate::table::Transaction {
            network: self.config.network(),
            enter_time: Some(entry.enter_time),
            commit_time: None,
            remove_time: None,
            hash: format!("{:#x}", entry.pool_transaction_entry.transaction.hash),
        };
        retry_send(&self.query_sender, point.insert_query()).await;
    }

    async fn report_commit(&self, entry: &TxEntry, committed_time: chrono::NaiveDateTime) {
        let point = crate::table::Transaction {
            network: self.config.network(),
            enter_time: None,
            commit_time: Some(committed_time),
            remove_time: None,
            hash: format!("{:#x}", entry.pool_transaction_entry.transaction.hash),
        };
        retry_send(&self.query_sender, point.update_query()).await;
    }

    async fn report_remove(&self, entry: &TxEntry) {
        let point = crate::table::Transaction {
            network: self.config.network(),
            enter_time: None,
            commit_time: None,
            remove_time: Some(chrono::Utc::now().naive_utc()),
            hash: format!("{:#x}", entry.pool_transaction_entry.transaction.hash),
        };
        retry_send(&self.query_sender, point.update_query()).await;
    }

    fn get_block_timestamp(&self, block_hash: Byte32) -> Option<chrono::NaiveDateTime> {
        self.jsonrpc.get_header(block_hash).map(|header| {
            chrono::NaiveDateTime::from_timestamp(
                (header.inner.timestamp.value() / 1000) as i64,
                (header.inner.timestamp.value() % 1000 * 1000) as u32,
            )
        })
    }
}

#[derive(Clone, Debug)]
struct TxEntry {
    pool_transaction_entry: PoolTransactionEntry,
    enter_time: chrono::NaiveDateTime, // pending time
}
