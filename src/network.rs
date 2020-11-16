use crate::app_config::app_config;
use crate::get_version::get_version;
use crate::CKB_NETWORK_IDENTIFIER;
use chrono::Utc;
use ckb_network::{
    bytes::Bytes, CKBProtocol, CKBProtocolContext, CKBProtocolHandler, DefaultExitHandler,
    NetworkService, NetworkState, PeerIndex, SupportProtocols,
};
use ckb_types::packed::{
    Byte32, CompactBlock, RelayMessage, RelayMessageUnion, SendBlock, SyncMessage, SyncMessageUnion,
};
use ckb_types::prelude::*;
use crossbeam::channel::Sender;
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// TODO --sync-historical-uncles

#[derive(Clone)]
pub struct Handler {
    peers: Arc<Mutex<HashMap<PeerIndex, bool>>>,
    compact_blocks: Arc<Mutex<HashMap<Byte32, (Instant, HashSet<PeerIndex>)>>>,
    transaction_hashes: Arc<Mutex<HashMap<Byte32, (Instant, HashSet<PeerIndex>)>>>,
    query_sender: Sender<WriteQuery>,
}

#[derive(InfluxDbWriteable)]
pub struct PropagationSerie {
    time: Timestamp,

    time_interval: u64, // ms

    #[tag]
    percentile: u32,
    #[tag]
    message_type: String,
}

impl Handler {
    pub fn new(query_sender: Sender<WriteQuery>) -> Self {
        Self {
            peers: Default::default(),
            compact_blocks: Default::default(),
            transaction_hashes: Default::default(),
            query_sender,
        }
    }

    #[allow(clippy::single_match)]
    fn received_relay(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        data: Bytes,
    ) {
        let relay_message = RelayMessage::from_slice(&data).unwrap().to_enum();
        match relay_message {
            RelayMessageUnion::CompactBlock(compact_block) => {
                self.received_compact_block(nc, peer_index, compact_block)
            }
            RelayMessageUnion::RelayTransactions(transactions) => {
                for transaction in transactions.transactions() {
                    let txhash = transaction.transaction().calc_tx_hash();
                    self.received_transaction_hash(nc.clone(), peer_index, txhash)
                }
            }
            RelayMessageUnion::RelayTransactionHashes(transaction_hashes) => {
                for txhash in transaction_hashes.tx_hashes() {
                    self.received_transaction_hash(nc.clone(), peer_index, txhash)
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::single_match)]
    fn received_sync(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        data: Bytes,
    ) {
        let sync_message = SyncMessage::from_slice(&data).unwrap().to_enum();
        match sync_message {
            SyncMessageUnion::SendBlock(send_block) => {
                self.received_send_block(nc, peer_index, send_block)
            }
            _ => {}
        }
    }

    fn received_compact_block(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        compact_block: CompactBlock,
    ) {
        let hash = compact_block.calc_header_hash();
        let mut compact_blocks = self.compact_blocks.lock().unwrap();
        let (first_received, peers) = compact_blocks
            .entry(hash)
            .or_insert_with(|| (Instant::now(), HashSet::default()));
        if peers.insert(peer_index) {
            let peers_total = self.peers.lock().unwrap().len();
            let elapsed = first_received.elapsed();
            if peers.len() == peers_total / 2 {
                let query = make_propagation_query(elapsed.as_millis() as u64, 50, "compact_block");
                self.query_sender.send(query).unwrap();
            } else if peers.len() * 10 == peers_total * 9 {
                let query = make_propagation_query(elapsed.as_millis() as u64, 90, "compact_block");
                self.query_sender.send(query).unwrap();
            }
        }
    }

    fn received_send_block(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        _peer_index: PeerIndex,
        _send_block: SendBlock,
    ) {
    }

    fn received_transaction_hash(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        hash: Byte32,
    ) {
        let mut transaction_hashes = self.transaction_hashes.lock().unwrap();
        let (first_received, peers) = transaction_hashes
            .entry(hash)
            .or_insert_with(|| (Instant::now(), HashSet::default()));
        if peers.insert(peer_index) {
            let peers_total = self.peers.lock().unwrap().len();
            let elapsed = first_received.elapsed();
            if peers.len() == peers_total / 2 {
                let query =
                    make_propagation_query(elapsed.as_millis() as u64, 50, "transaction_hash");
                self.query_sender.send(query).unwrap();
            } else if peers.len() * 10 == peers_total * 9 {
                let query =
                    make_propagation_query(elapsed.as_millis() as u64, 90, "transaction_hash");
                self.query_sender.send(query).unwrap();
            }
        }
    }
}

impl CKBProtocolHandler for Handler {
    fn init(&mut self, _nc: Arc<dyn CKBProtocolContext + Sync>) {}

    fn connected(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        _version: &str,
    ) {
        let mut peers = self.peers.lock().unwrap();
        peers.entry(peer_index).or_insert(true);
    }

    fn disconnected(&mut self, _nc: Arc<dyn CKBProtocolContext + Sync>, peer_index: PeerIndex) {
        self.peers.lock().unwrap().remove(&peer_index);
    }

    fn received(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        data: Bytes,
    ) {
        if nc.protocol_id() == SupportProtocols::Relay.protocol_id() {
            self.received_relay(nc, peer_index, data);
        } else if nc.protocol_id() == SupportProtocols::Sync.protocol_id() {
            self.received_sync(nc, peer_index, data);
        }
    }
}

fn make_propagation_query<M>(time_interval: u64, percentile: u32, message_type: M) -> WriteQuery
where
    M: ToString,
{
    static QUERY_NAME: &str = "propagation";

    PropagationSerie {
        time: Utc::now().into(),
        time_interval,
        percentile,
        message_type: message_type.to_string(),
    }
    .into_query(QUERY_NAME)
}

pub(crate) fn run_network_service(handler: Handler) {
    let config = app_config().network;
    let network_state = Arc::new(NetworkState::from_config(config).unwrap());
    let exit_handler = DefaultExitHandler::default();
    let version = get_version();
    let protocols = vec![SupportProtocols::Sync, SupportProtocols::Relay];
    let required_protocol_ids = protocols
        .iter()
        .map(|protocol| protocol.protocol_id())
        .collect();
    let ckb_protocols = protocols
        .into_iter()
        .map(|protocol| {
            CKBProtocol::new_with_support_protocol(
                protocol,
                Box::new(handler.clone()),
                Arc::clone(&network_state),
            )
        })
        .collect();
    let _network_controller = NetworkService::new(
        network_state,
        ckb_protocols,
        required_protocol_ids,
        CKB_NETWORK_IDENTIFIER.clone(),
        version.to_string(),
        exit_handler.clone(),
    )
    .start(Some("NetworkService"))
    .unwrap();

    exit_handler.wait_for_exit();
}
