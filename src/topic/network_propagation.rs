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

use crate::util::{get_network_identifier, get_version};
use chrono::Utc;
use ckb_app_config::NetworkConfig;
use ckb_network::{
    bytes::Bytes, CKBProtocol, CKBProtocolContext, CKBProtocolHandler, DefaultExitHandler,
    NetworkService, NetworkState, Peer, PeerIndex, SupportProtocols,
};
use ckb_suite_rpc::Jsonrpc;
use ckb_types::packed::{
    Byte32, CompactBlock, RelayMessage, RelayMessageUnion, SendBlock, SyncMessage, SyncMessageUnion,
};
use ckb_types::prelude::*;
use ipinfo::IpInfo;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tentacle_multiaddr::Multiaddr;

// TODO handle threads panic
// TODO logger
// TODO install node-exporter on testnet machines
// TODO maintain `nc`, so that not maintain `peers`

/// # Notes
///
/// This thread only records events into db.
/// And there is another thread that event history from db, then write the processed state into another db table.
///
/// First, let's consider what the dashboards present:
/// * time - live peers count, graph
/// * time - 80%/90%/95% percent propagation, graph
/// * node version distribution, pie
/// * node country distribution, pie
/// * ~~time - count of peers with 1m/1h/1d/1M connection duration, graph~~
///
/// Next, list what kind of original information we can collect:
/// 1. insert a heartbeat record into db with time=now. then db select unique peer id in the time range to calculate the live peers count.
/// 2. background thread insert the calculated live peers count into another table "peers{time, count}".
/// 3. insert (CONFLICT ON (peer_id, hash) DO NOTHING ) "propagation{time,peer,hash,message_type}" into db when receives messages.
/// 4. background thread calculate 80%/90%/95% percent propagation and insert into db "propagation_percentage".
/// 5. select count(*) from peer group by version in the time range, the node version distribution. do it on Grafana

#[derive(Clone)]
pub(crate) struct NetworkPropagation {
    config: crate::Config,
    jsonrpc: Jsonrpc,
    query_sender: crossbeam::channel::Sender<String>,
    async_handle02: ckb_async_runtime::Handle,
    last_heartbeat: Arc<Mutex<VecDeque<(PeerIndex, Instant)>>>,
    ipinfo: Arc<Mutex<IpInfo>>,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10 * 60);

impl NetworkPropagation {
    pub(crate) fn new(
        config: crate::Config,
        jsonrpc: Jsonrpc,
        query_sender: crossbeam::channel::Sender<String>,
        async_handle02: ckb_async_runtime::Handle,
    ) -> Self {
        let ipinfo = ipinfo::IpInfo::new(ipinfo::IpInfoConfig {
            token: config.ipinfo_io_token.clone(),
            cache_size: 1000,
            timeout: ::std::time::Duration::from_secs(2 * 60),
        })
        .expect("connect to https://ipinfo.io");
        Self {
            config,
            jsonrpc,
            query_sender,
            async_handle02,
            last_heartbeat: Arc::new(Mutex::new(Default::default())),
            ipinfo: Arc::new(Mutex::new(ipinfo)),
        }
    }

    pub(crate) fn run(&mut self) {
        let ckb_network_config = build_network_config(self.config.node.bootnodes.clone());
        let network_state =
            Arc::new(NetworkState::from_config(ckb_network_config.clone()).unwrap());
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
        let network_identifier = get_network_identifier(&self.jsonrpc);
        let async_handle = self.async_handle02.clone();
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
        if let Some(peer) = nc.get_peer(peer_index) {
            self.report_propagation("b", &peer, &hash)
        }
    }

    fn received_transaction_hash(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        hash: Byte32,
    ) {
        if let Some(peer) = nc.get_peer(peer_index) {
            self.report_propagation("t", &peer, &hash)
        }
    }

    fn received_send_block(
        &mut self,
        _nc: Arc<dyn CKBProtocolContext + Sync>,
        _peer_index: PeerIndex,
        _send_block: SendBlock,
    ) {
    }

    fn report_propagation<M>(&self, message_name: M, peer: &Peer, hash: &Byte32)
    where
        M: ToString,
    {
        let point = crate::table::Propagation {
            network: self.config.network(),
            time: Utc::now().naive_utc(),
            peer_id: peer.peer_id.to_base58(),
            hash: format!("{:#x}", hash),
            message_name: message_name.to_string(),
        };
        self.query_sender.send(point.insert_query()).unwrap();
    }

    fn report_heartbeat(&self, peer: &Peer) {
        let host = {
            let host = peer
                .connected_addr
                .iter()
                .find_map(|protocol| match protocol {
                    tentacle_multiaddr::Protocol::IP4(ip4) => Some(ip4.to_string()),
                    tentacle_multiaddr::Protocol::IP6(ip6) => ip6
                        .to_ipv4()
                        .map(|ip4| ip4.to_string())
                        .or_else(|| Some(ip6.to_string())),
                    tentacle_multiaddr::Protocol::DNS4(dns4) => Some(dns4.to_string()),
                    tentacle_multiaddr::Protocol::DNS6(dns6) => Some(dns6.to_string()),
                    _ => None,
                });
            host.unwrap_or_else(|| peer.connected_addr.to_string())
        };
        let client_version = if let Some(ref identify) = peer.identify_info {
            identify.client_version.clone()
        } else {
            "-".to_string()
        };
        let country = self.lookup_country(&host);
        let point = crate::table::Heartbeat {
            network: self.config.network(),
            time: Utc::now().naive_utc(),
            peer_id: peer.peer_id.to_base58(),
            host,
            connected_duration: peer.connected_time.elapsed().as_millis() as i64,
            client_version,
            country,
        };
        self.query_sender.send(point.insert_query()).unwrap();
    }

    pub fn lookup_country(&self, ip: &str) -> String {
        let mut ipinfo = self.ipinfo.lock().expect("lock ipinfo");
        match ipinfo.lookup(&[ip]) {
            Ok(info_map) => {
                let ipdetail = &info_map[ip];
                ipdetail.country.clone()
            }
            Err(err) => {
                log::error!("lookup ipinfo.io for {}, error: {:?}", ip, err);
                String::from("")
            }
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
        if let Some(peer) = nc.get_peer(peer_index) {
            log::info!("connect with #{}({:?})", peer_index, peer.connected_addr);
        }
        if let Ok(mut last_heartbeat) = self.last_heartbeat.lock() {
            if !last_heartbeat.iter().any(|(pi, _)| *pi == peer_index) {
                // Assign an old timestamp, so that it can be report heartbeat ASAP
                let tricky_instant = Instant::now() - HEARTBEAT_INTERVAL + Duration::from_secs(10);
                last_heartbeat.push_back((peer_index, tricky_instant));
            }
        }
    }

    fn disconnected(&mut self, nc: Arc<dyn CKBProtocolContext + Sync>, peer_index: PeerIndex) {
        if let Some(peer) = nc.get_peer(peer_index) {
            log::info!("disconnect with #{}({:?})", peer_index, peer.connected_addr);
        }
    }

    fn received(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        data: Bytes,
    ) {
        if nc.protocol_id() == SupportProtocols::Relay.protocol_id() {
            self.received_relay(nc.clone(), peer_index, data);
        } else if nc.protocol_id() == SupportProtocols::Sync.protocol_id() {
            self.received_sync(nc.clone(), peer_index, data);
        }

        if let Ok(mut last_heartbeat) = self.last_heartbeat.lock() {
            let mut to_heartbeat = Vec::new();
            while let Some((_pi, instant)) = last_heartbeat.front() {
                if instant.elapsed() < HEARTBEAT_INTERVAL {
                    break;
                }
                let (pi, _) = last_heartbeat.pop_front().unwrap();
                to_heartbeat.push(pi);
            }
            for pi in to_heartbeat.into_iter().rev() {
                // If nc.get_peer() return None, this peer was disconnected, just discard it.
                if let Some(peer) = nc.get_peer(pi) {
                    self.report_heartbeat(&peer);
                    last_heartbeat.push_back((pi, Instant::now()));
                }
            }
        }
    }
}

fn build_network_config(bootnodes: Vec<Multiaddr>) -> NetworkConfig {
    // let port = find_available_port();
    let mut config = NetworkConfig::default();
    config.bootnodes = bootnodes;
    config.path = PathBuf::from("network");
    // Don't listen public port!
    // config.listen_addresses = vec![format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()];
    config.listen_addresses = vec![];
    config.max_peers = 5000;
    config.max_outbound_peers = 5000;
    config.ping_interval_secs = 120;
    config.ping_timeout_secs = 1200;
    config.connect_outbound_interval_secs = 15;
    config
}
