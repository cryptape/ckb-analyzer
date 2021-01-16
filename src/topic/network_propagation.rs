//! This module aims to collect the blockchain network information at p2p level.
//! This modules produces the below measurements:
//!   - [Propagation](TODO link)
//!   - [Peers](TODO link)
//!   - [HighLatency](TODO link)
//!
//! ### How it works?
//!
//! This module runs a p2p service, compatible with ckb network protocol, to connect peers(ckb nodes) as more as
//! possible. By listening and aggregating the messages relayed from peers we can estimate that, from our owned
//! perspective, how long it take to propagate a compate block or transaction, the number of public nodes, the version distribution around the network and so on.
//!
//! ### Why measure it?
//!
//! As a decentralized network, understanding the network is important.
//!
//! It is unable to observe a network directly. Instead we observe from the perspective of the network particants and then try to aggregate these information from different particants to construct the network picture.
//!
//! Note that in a decentralized network, one can only deploy programs on its owned machines, but not the entire network. So of cause, no one can see the whole picture of a decentralized network.

use crate::get_version::get_version;
use crate::measurement::{self, IntoWriteQuery, WriteQuery};
use crate::util::find_available_port::find_available_port;
use crate::util::get_network_identifier::get_network_identifier;
use chrono::Utc;
use ckb_app_config::NetworkConfig;
use ckb_network::{
    bytes::Bytes, CKBProtocol, CKBProtocolContext, CKBProtocolHandler, DefaultExitHandler,
    NetworkService, NetworkState, Peer, PeerIndex, SupportProtocols,
};
use ckb_types::packed::{
    Byte32, CompactBlock, RelayMessage, RelayMessageUnion, SendBlock, SyncMessage, SyncMessageUnion,
};
use ckb_types::prelude::*;
use crossbeam::channel::Sender;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tentacle_multiaddr::Multiaddr;

// TODO handle threads panic
// TODO logger
// TODO install node-exporter on testnet machines

type PropagationHashes = Arc<Mutex<HashMap<Byte32, (Instant, HashSet<PeerIndex>)>>>;

#[derive(Clone)]
pub(crate) struct NetworkPropagation {
    ckb_network_config: NetworkConfig,
    ckb_rpc_url: String,
    peers: Arc<Mutex<HashMap<PeerIndex, bool>>>,
    compact_blocks: PropagationHashes,
    transaction_hashes: PropagationHashes,
    query_sender: Sender<WriteQuery>,
}

impl NetworkPropagation {
    pub(crate) fn new(
        ckb_rpc_url: String,
        bootnodes: Vec<Multiaddr>,
        query_sender: Sender<WriteQuery>,
    ) -> Self {
        let ckb_network_config = build_network_config(bootnodes);
        Self {
            ckb_network_config,
            ckb_rpc_url,
            peers: Default::default(),
            compact_blocks: Default::default(),
            transaction_hashes: Default::default(),
            query_sender,
        }
    }

    pub(crate) fn run(&mut self, async_handle: ckb_async_runtime::Handle) {
        let network_state =
            Arc::new(NetworkState::from_config(self.ckb_network_config.clone()).unwrap());
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
                    Box::new(self.clone()),
                    Arc::clone(&network_state),
                )
            })
            .collect();
        let network_identifier = get_network_identifier(&self.ckb_rpc_url);
        let _network_controller = NetworkService::new(
            network_state,
            ckb_protocols,
            required_protocol_ids,
            network_identifier,
            version.to_string(),
            exit_handler.clone(),
        )
        .start(&async_handle)
        .unwrap();

        exit_handler.wait_for_exit();
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
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        compact_block: CompactBlock,
    ) {
        let hash = compact_block.calc_header_hash();
        let (peers_received, first_received, newly_inserted) = {
            let mut compact_blocks = self.compact_blocks.lock().unwrap();
            let (first_received, peers) = compact_blocks
                .entry(hash)
                .or_insert_with(|| (Instant::now(), HashSet::default()));
            let newly_inserted = peers.insert(peer_index);
            (peers.len() as u32, *first_received, newly_inserted)
        };
        if newly_inserted {
            if let Some(peer) = nc.get_peer(peer_index) {
                self.send_high_latency_query(first_received, &peer);
                self.send_propagation_query("compact_block", peers_received, first_received)
            }
        }
    }

    fn received_transaction_hash(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        hash: Byte32,
    ) {
        let (peers_received, first_received, newly_inserted) = {
            let mut transaction_hashes = self.transaction_hashes.lock().unwrap();
            let (first_received, peers) = transaction_hashes
                .entry(hash)
                .or_insert_with(|| (Instant::now(), HashSet::default()));
            let newly_inserted = peers.insert(peer_index);
            (peers.len() as u32, *first_received, newly_inserted)
        };
        if newly_inserted {
            self.send_propagation_query("transaction_hash", peers_received, first_received)
        }
    }

    fn received_send_block(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        _peer_index: PeerIndex,
        _send_block: SendBlock,
    ) {
    }

    fn send_high_latency_query(&self, first_received: Instant, peer: &Peer) {
        let time_interval = first_received.elapsed();
        if time_interval < Duration::from_secs(8) {
            return;
        }

        let query = measurement::HighLatency {
            time: Utc::now().into(),
            time_interval: time_interval.as_millis() as u64,
            addr: peer.connected_addr.to_string(),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn send_propagation_query<M>(
        &self,
        message_type: M,
        peers_received: u32,
        first_received: Instant,
    ) where
        M: ToString,
    {
        let peers_total = self.peers.lock().unwrap().len();
        let last_percentile = (peers_received - 1) as f32 * 100.0 / peers_total as f32;
        let current_percentile = peers_received as f32 * 100.0 / peers_total as f32;
        let time_interval = first_received.elapsed().as_millis() as u64;
        let percentile = if last_percentile < 99.0 && current_percentile >= 99.0 {
            Some(99)
        } else if last_percentile < 95.0 && current_percentile >= 95.0 {
            Some(95)
        } else if last_percentile < 80.0 && current_percentile >= 80.0 {
            Some(80)
        } else {
            None
        };

        if let Some(percentile) = percentile {
            let query = measurement::Propagation {
                time: Utc::now().into(),
                time_interval,
                percentile,
                message_type: message_type.to_string(),
            }
            .into_write_query();
            self.query_sender.send(query).unwrap();
        }
    }

    fn send_peers_total_query(&self) {
        if let Ok(guard) = self.peers.lock() {
            let peers_total = guard.len() as u32;
            let query = measurement::Peers {
                time: Utc::now().into(),
                peers_total,
            }
            .into_write_query();
            self.query_sender.send(query).unwrap();
        }
    }
}

impl CKBProtocolHandler for NetworkPropagation {
    fn init(&mut self, _nc: Arc<dyn CKBProtocolContext + Sync>) {}

    fn connected(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        _version: &str,
    ) {
        if let Ok(mut peers) = self.peers.lock() {
            if *peers.entry(peer_index).or_insert(true) {
                if let Some(peer) = nc.get_peer(peer_index) {
                    log::info!("connect with #{}({:?})", peer_index, peer.connected_addr);
                }
            }
        }
        self.send_peers_total_query();
    }

    fn disconnected(&mut self, nc: Arc<dyn CKBProtocolContext + Sync>, peer_index: PeerIndex) {
        if let Ok(mut peers) = self.peers.lock() {
            if peers.remove(&peer_index).is_some() {
                if let Some(peer) = nc.get_peer(peer_index) {
                    log::info!("disconnect with #{}({:?})", peer_index, peer.connected_addr);
                }
            }
        }
        self.send_peers_total_query();
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

fn build_network_config(bootnodes: Vec<Multiaddr>) -> NetworkConfig {
    let port = find_available_port();
    let mut config = NetworkConfig::default();
    config.bootnodes = bootnodes;
    config.path = PathBuf::from("network");
    config.listen_addresses = vec![format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()];
    config.max_peers = 5000;
    config.max_outbound_peers = 5000;
    config.ping_interval_secs = 120;
    config.ping_timeout_secs = 1200;
    config.connect_outbound_interval_secs = 15;
    config
}
