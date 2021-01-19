use crate::measurement::{self, IntoWriteQuery};
use crate::subscribe::{Subscription, Topic};
use chrono::{DateTime, Utc};
use ckb_suite_rpc::{
    ckb_jsonrpc_types::{PoolTransactionEntry, Status},
    Jsonrpc,
};
use ckb_types::{packed::Byte32, prelude::*};
use crossbeam::channel::Sender;
use influxdb::{Timestamp, WriteQuery};
use jsonrpc_core::serde_from_str;
use std::collections::HashMap;
use std::time::Instant;

/// This module aims to the pool transactions
///
/// ### How it works?
///
/// We dont care about chain reorganization at present, which means it will not observe orphan transactions.
///
/// ### Why measure it?
///
/// Know the transaction traffic and estimate the mean waiting time for a pending transaction become from pending to committed.
///
/// ### Note
///
/// Consider a dashboard that tells a story about transaction events.
///
/// List known transaction events here:
///   - "pending", start pending in pool;
///   - "pending_long", being pending for too long;
///   - "propose", transit from pending to proposed, with the delay duration, proposed block number;
///   - "propose_long", expire proposal window, with the delay duration;
///   - "commit", transit from proposed to committed, with the delay duration, delay block count, proposed block number, committed block number;
///   - "remove", remove with unknown reason;
///
/// The serie may be like below. Some of events are used to visualize in heatmap, some in table:
/// ```json
/// {
///   "query_name": "tx-transition",
///
///   "txhash": h256,
///
///   "event": "pending"/"pending_long"/"propose"/"propose_long"/"commit"/"remove",
///
///   "time": time instant;
///         if event is "pending", assign it `now`;
///         if event is "pending_long", keeps this field the same as "pending";
///         if event is "propose", assign it proposed block timestamp;
///         if event is "propose_long", keeps this field the same as "propose";
///         if event is "commit", assign it committed block timestamp;
///         if event is "remove", assign it `now`;
///
///   "elapsed": duration;
///         if event is "pending", assign it 0;
///         if event is "pending_long", assign `now - pending.time`;
///         if event is "propose", assign `proposed.time - pending.time`;
///         if event is "propose_long", assign `now - proposed.time`;
///         if event is "commit", assign `committed.time - proposed.time";
///         if event is "remove", assign it `now - pending.time`;
/// }
/// ```
///
/// * maintaining fields:
///   - pending: #{txhash => pending timestamp}
///   - proposed: #{txhash => pending timestamp}
///
/// * Information source:
///   - Subscription of NewTransaction
///     Retrieve transaction status via `get_transaction`, if status is pending, then report a
///     "pending" event, otherwise do nothing.
///
/// * Periodically checking
///   Travels transactions in self.pending and self.proposed, check the transaction status
///   transition.
///   As for transaction in self.pending,
///     - if its current status is unknown, remove out and report "remove" event;
///     - if its current status is pending and elapses within threshold, discard;
///     - if its current status is pending and elapses beyond threshold, remove out and report
///       "pending_long" event;
///     - if its current status is proposed, move into self.proposed and report "propose" event;
///     - if its current status is committed, remove out;
///   As for transaction in self.proposed,
///     - if its current status is unknown, remove out and report "remove" event;
///     - if its current status is pending, move into self.pending and report
///       "propose_back_pending" event;
///     - if its current status is proposed and elapses within threshold, discard
///     - if its current status is proposed and elapses beyond threshold, remove out and report
///       "propose_long" event;
///     - if its current status is committed, remove out and report "commit" event;
///
/// * TODO
///   - Retrieve all pool transactions via RPC `get_raw_tx_pool` at the start.
///   - Fix the issue that it takes too long to travel all the pending/proposed transactions.
///   - Enable subscription at ckb

const WAITING_THRESHOLD_MILLIS: u64 = ::std::time::Duration::from_secs(5 * 60).as_millis() as u64;

pub(crate) struct TxTransition {
    // waiting: HashMap<Byte32, DateTime<Utc>>,
    pending: HashMap<Byte32, Timestamp>,
    proposed: HashMap<Byte32, Timestamp>,

    subscriber: crossbeam::channel::Receiver<(Topic, String)>,

    jsonrpc: Jsonrpc,
    query_sender: Sender<WriteQuery>,
    last_checking_at: Instant,
}

impl TxTransition {
    pub(crate) fn new(
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
        query_sender: Sender<WriteQuery>,
    ) -> (Self, Subscription) {
        let (subscription, subscriber) =
            Subscription::new(ckb_subscribe_url.clone(), Topic::NewTransaction);
        let jsonrpc = Jsonrpc::connect(&ckb_rpc_url);
        (
            Self {
                jsonrpc,
                query_sender,
                subscriber,
                pending: Default::default(),
                proposed: Default::default(),
                last_checking_at: Instant::now(),
            },
            subscription,
        )
    }

    async fn recv_new_transaction(&mut self, txhash: Byte32) {
        // Discard the newly subscribed transaction for unknown, proposed or committed status
        if self.get_tx_status(&txhash).is_pending() && !self.pending.contains_key(&txhash) {
            let now = Utc::now().into();
            self.report_pending(&txhash, now);
            self.pending.insert(txhash, now);
        }
    }

    async fn try_recv_subscription(
        &self,
    ) -> Result<(Topic, String), crossbeam::channel::TryRecvError> {
        self.subscriber.try_recv()
    }

    pub(crate) async fn run(mut self) {
        loop {
            match self.try_recv_subscription().await {
                Ok((topic, message)) => {
                    assert_eq!(Topic::NewTransaction, topic);
                    let tx = serde_from_str::<PoolTransactionEntry>(&message).unwrap();
                    let txhash = tx.transaction.hash.pack();
                    self.recv_new_transaction(txhash).await;
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => return,
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    // TODO 分批检查，或者更高效的方式
                    if self.last_checking_at.elapsed() >= ::std::time::Duration::from_secs(60) {
                        self.check_pending().await;
                        self.check_proposed().await;
                        self.last_checking_at = Instant::now();
                    } else {
                        tokio::time::delay_for(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }

    // Returns (removed, proposed)
    async fn check_pending_tx(
        &self,
        txhash: &Byte32,
        pending_timestamp: Timestamp,
    ) -> (bool, bool) {
        match self.get_tx_status(txhash) {
            TxStatus::Unknown => {
                self.report_remove(txhash, pending_timestamp);
                (true, false)
            }
            TxStatus::Pending => {
                if elapsed_millis(pending_timestamp) < WAITING_THRESHOLD_MILLIS {
                    (false, false)
                } else {
                    self.report_pending_long(txhash, pending_timestamp);
                    (true, false)
                }
            }
            TxStatus::Proposed => {
                if !self.proposed.contains_key(&txhash) {
                    self.report_propose(txhash, pending_timestamp);
                    (true, true)
                } else {
                    (true, false)
                }
            }
            TxStatus::Committed(_block_hash) => (true, false),
        }
    }

    //   As for transaction in self.pending,
    //     - if its current status is unknown, remove out and report "remove" event;
    //     - if its current status is pending and elapses within threshold, discard;
    //     - if its current status is pending and elapses beyond threshold, remove out and report
    //       "pending_long" event;
    //     - if its current status is proposed, move into self.proposed and report "propose" event;
    //     - if its current status is committed, remove out;
    async fn check_pending(&mut self) {
        let mut to_remove = Vec::new();
        for (txhash, pending_timestamp) in self.pending.iter() {
            let (removed, proposed) = self.check_pending_tx(txhash, *pending_timestamp).await;
            if removed {
                to_remove.push(txhash.clone());
            }
            if proposed {
                self.proposed.insert(txhash.clone(), *pending_timestamp);
            }
        }

        for txhash in to_remove.iter() {
            self.pending.remove(txhash);
        }
    }

    // Returns (removed, re-pending)
    async fn check_proposed_tx(
        &self,
        txhash: &Byte32,
        pending_timestamp: Timestamp,
    ) -> (bool, bool) {
        match self.get_tx_status(txhash) {
            TxStatus::Unknown => {
                self.report_remove(txhash, pending_timestamp);
                (true, false)
            }
            TxStatus::Pending => {
                // Move back to self.pending.
                // Although reusing proposed timestamp is not good, it is not bad as well
                if !self.pending.contains_key(txhash) {
                    (true, true)
                } else {
                    (false, false)
                }
            }
            TxStatus::Proposed => {
                if elapsed_millis(pending_timestamp) < WAITING_THRESHOLD_MILLIS {
                    // discard
                    (false, false)
                } else {
                    self.report_propose_long(txhash, pending_timestamp);
                    (true, false)
                }
            }
            TxStatus::Committed(block_hash) => {
                if let Some(committed_timestamp) = self.get_block_timestamp(block_hash) {
                    self.report_commit(txhash, pending_timestamp, committed_timestamp);
                    (true, false)
                } else {
                    log::error!("get_block_timestamp with committed tx returns unknown; discard");
                    // discard
                    (false, false)
                }
            }
        }
    }

    //   As for transaction in self.proposed,
    //     - if its current status is unknown, remove out and report "remove" event;
    //     - if its current status is pending, move into self.pending;
    //     - if its current status is proposed and elapses within threshold, discard
    //     - if its current status is proposed and elapses beyond threshold, remove out and report
    //       "propose_long" event;
    //     - if its current status is committed, remove out and report "commit" event;
    async fn check_proposed(&mut self) {
        let mut to_remove = Vec::new();
        for (txhash, pending_timestamp) in self.proposed.iter() {
            let (removed, repending) = self.check_proposed_tx(&txhash, *pending_timestamp).await;
            if removed {
                to_remove.push(txhash.clone());
            }
            if repending {
                self.pending.insert(txhash.clone(), *pending_timestamp);
            }
        }

        for txhash in to_remove.iter() {
            self.proposed.remove(txhash);
        }
    }

    fn report_pending(&self, txhash: &Byte32, timestamp: Timestamp) {
        let query = measurement::TxTransition {
            time: timestamp,
            elapsed: 0,
            event: TxEvent::Pending.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn report_propose(&self, txhash: &Byte32, pending_timestamp: Timestamp) {
        // NOTE: We re-use the pending timestamp in self.proposed. Then assign elapsed with `now - pending timestamp`
        let now = Utc::now().into();
        let query = measurement::TxTransition {
            time: now,
            elapsed: diff_millis(now, pending_timestamp),
            event: TxEvent::Propose.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn report_commit(
        &self,
        txhash: &Byte32,
        proposed_timestamp: Timestamp,
        committed_timestamp: Timestamp,
    ) {
        let query = measurement::TxTransition {
            time: committed_timestamp,
            elapsed: diff_millis(committed_timestamp, proposed_timestamp),
            event: TxEvent::Commit.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn report_pending_long(&self, txhash: &Byte32, timestamp: Timestamp) {
        let query = measurement::TxTransition {
            time: timestamp,
            elapsed: elapsed_millis(timestamp),
            event: TxEvent::PendingLong.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn report_propose_long(&self, txhash: &Byte32, timestamp: Timestamp) {
        let query = measurement::TxTransition {
            time: timestamp,
            elapsed: elapsed_millis(timestamp),
            event: TxEvent::ProposeLong.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn report_remove(&self, txhash: &Byte32, timestamp: Timestamp) {
        let query = measurement::TxTransition {
            time: timestamp,
            elapsed: elapsed_millis(timestamp),
            event: TxEvent::Remove.to_string(),
            txhash: format!("{:?}", txhash),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn get_tx_status(&self, txhash: &Byte32) -> TxStatus {
        if let Some(txstatus) = self.jsonrpc.get_transaction(txhash.clone()) {
            match txstatus.tx_status.status {
                Status::Pending => TxStatus::Pending,
                Status::Proposed => TxStatus::Proposed,
                Status::Committed => {
                    TxStatus::Committed(txstatus.tx_status.block_hash.unwrap().pack())
                }
            }
        } else {
            TxStatus::Unknown
        }
    }

    fn get_block_timestamp(&self, block_hash: Byte32) -> Option<Timestamp> {
        if let Some(header) = self.jsonrpc.get_header(block_hash) {
            return Some(Timestamp::Milliseconds(
                header.inner.timestamp.value() as u128
            ));
        }
        None
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum TxStatus {
    Unknown,
    Pending,
    Proposed,
    Committed(Byte32),
}

#[allow(dead_code)]
impl TxStatus {
    fn is_unknown(&self) -> bool {
        self == &TxStatus::Unknown
    }
    fn is_pending(&self) -> bool {
        self == &TxStatus::Pending
    }
    fn is_proposed(&self) -> bool {
        self == &TxStatus::Proposed
    }
    fn is_committed(&self) -> bool {
        if let Self::Committed(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
enum TxEvent {
    Pending,
    PendingLong,
    Propose,
    ProposeLong,
    Commit,
    Remove,
}

impl ::std::fmt::Display for TxEvent {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::PendingLong => write!(f, "pending_long"),
            Self::Propose => write!(f, "propose"),
            Self::ProposeLong => write!(f, "propose_long"),
            Self::Commit => write!(f, "commit"),
            Self::Remove => write!(f, "remove"),
        }
    }
}

fn elapsed_millis(instant: Timestamp) -> u64 {
    let instant: DateTime<Utc> = instant.into();
    let now = Utc::now();
    (now.timestamp_millis() as u64).saturating_sub(instant.timestamp_millis() as u64)
}

fn diff_millis(minuend: Timestamp, minus: Timestamp) -> u64 {
    let minuend: DateTime<Utc> = minuend.into();
    let minus: DateTime<Utc> = minus.into();
    (minuend.timestamp_millis() as u64).saturating_sub(minus.timestamp_millis() as u64)
}
