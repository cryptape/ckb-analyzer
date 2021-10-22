use crate::entry;
use chrono::Utc;
use ckb_app_config::{NetworkConfig, SupportProtocol};
use ckb_network::{
    bytes::Bytes, multiaddr_to_socketaddr, CKBProtocol, CKBProtocolContext, CKBProtocolHandler,
    DefaultExitHandler, NetworkService, NetworkState, PeerIndex, SupportProtocols,
};
use ckb_testkit::util::find_available_port;
use ckb_testkit::Node;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tentacle_multiaddr::Multiaddr;

#[derive(Clone)]
pub struct PeerCollector {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
    async_handle: ckb_async_runtime::Handle,

    last_update_time: Instant,
}

impl PeerCollector {
    pub fn new(
        node: Node,
        query_sender: crossbeam::channel::Sender<String>,
        async_handle: ckb_async_runtime::Handle,
    ) -> Self {
        Self {
            node,
            query_sender,
            async_handle,
            last_update_time: Instant::now(),
        }
    }

    pub fn run(&self) {
        let network_state = Arc::new(NetworkState::from_config(self.network_config()).unwrap());
        let exit_handler = DefaultExitHandler::default();
        let protocols = vec![
            SupportProtocols::Discovery,
            SupportProtocols::Sync,
            // For communicating with CKB nodes, Relay protocol is not necessary.
            // Just open Relay protocol to trigger `fn received` callback.
            SupportProtocols::Relay,
            // SupportProtocols::Ping,
            // SupportProtocols::Identify,
            // SupportProtocols::DisconnectMessage,
            // ATTENTION: Don't enable Feeler, it causes incomplete protocol error
            // SupportProtocols::Feeler,
        ];
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
        let client_version = self.client_version();
        let network_identifier = self.network_identifier();
        let async_handle = self.async_handle.clone();
        let _network_controller = NetworkService::new(
            network_state,
            ckb_protocols,
            required_protocol_ids,
            network_identifier,
            client_version,
            exit_handler.clone(),
        )
        .start(&async_handle)
        .unwrap();

        exit_handler.wait_for_exit();
    }

    fn network_identifier(&self) -> String {
        let consensus = self.node.consensus();
        let genesis_hash = format!("{:x}", consensus.genesis_hash);
        format!("/{}/{}", consensus.id, &genesis_hash[..8])
    }

    fn client_version(&self) -> String {
        self.node.rpc_client().local_node_info().version
    }

    fn network_config(&self) -> NetworkConfig {
        let mut config = NetworkConfig::default();

        // https://github.com/nervosnetwork/ckb/pull/2641
        config.support_protocols = vec![SupportProtocol::Discovery];
        config.bootnodes = {
            let bootnodes = match self.node.consensus().id.as_str() {
                "ckb" => vec![
                    "/ip4/47.110.15.57/tcp/8114/p2p/QmXS4Kbc9HEeykHUTJCm2tNmqghbvWyYpUp6BtE5b6VrAU",
                    "/ip4/47.245.31.79/tcp/8114/p2p/QmUaSuEdXNGJEKvkE4rCn3cwBrpRFUm5TsouF4M3Sjursv",
                    "/ip4/13.234.144.148/tcp/8114/p2p/QmbT7QimcrcD5k2znoJiWpxoESxang6z1Gy9wof1rT1LKR",
                    "/ip4/3.218.170.86/tcp/8114/p2p/QmShw2vtVt49wJagc1zGQXGS6LkQTcHxnEV3xs6y8MAmQN",
                    "/ip4/52.59.155.249/tcp/8114/p2p/QmRHqhSGMGm5FtnkW8D6T83X7YwaiMAZXCXJJaKzQEo3rb",
                ],
                "ckb_testnet" =>  vec![
                    "/ip4/47.111.169.36/tcp/8111/p2p/QmNQ4jky6uVqLDrPU7snqxARuNGWNLgSrTnssbRuy3ij2W",
                    "/ip4/18.217.146.65/tcp/8111/p2p/QmT6DFfm18wtbJz3y4aPNn3ac86N4d4p4xtfQRRPf73frC",
                    "/ip4/18.136.60.221/tcp/8111/p2p/QmTt6HeNakL8Fpmevrhdna7J4NzEMf9pLchf1CXtmtSrwb",
                    "/ip4/35.176.207.239/tcp/8111/p2p/QmSJTsMsMGBjzv1oBNwQU36VhQRxc2WQpFoRu1ZifYKrjZ",
                ],
                _ => unimplemented!(),
            };
            bootnodes
                .into_iter()
                .map(|addr| Multiaddr::from_str(addr).unwrap())
                .collect()
        };

        config.path = PathBuf::from("network");

        // NOTE: Publishing CKBAnalyzer addresses pollute the CKB network, while
        // we intend to connect to as more as possible peers, including
        // private-address nodes.
        let port = find_available_port();
        config.listen_addresses = vec![format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()];

        // Connect peeres as more as possible
        config.max_peers = 5000;
        config.max_outbound_peers = 3000;
        config.ping_interval_secs = 10;
        config.ping_timeout_secs = 1200;
        config.connect_outbound_interval_secs = 1;
        config.discovery_announce_check_interval_secs = Some(1);
        config
    }
}

impl CKBProtocolHandler for PeerCollector {
    fn init(&mut self, _nc: Arc<dyn CKBProtocolContext + Sync>) {}

    fn connected(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        peer_index: PeerIndex,
        _version: &str,
    ) {
        if let Some(peer) = nc.get_peer(peer_index) {
            ckb_testkit::info!(
                "connect with #{}({:?}), protocols: {:?}",
                peer_index,
                peer.connected_addr,
                peer.protocols
            );
        }
    }

    fn disconnected(&mut self, nc: Arc<dyn CKBProtocolContext + Sync>, peer_index: PeerIndex) {
        if let Some(peer) = nc.get_peer(peer_index) {
            ckb_testkit::info!("disconnect with #{}({:?})", peer_index, peer.connected_addr);
        }
    }

    fn received(
        &mut self,
        nc: Arc<dyn CKBProtocolContext + Sync>,
        _peer_index: PeerIndex,
        data: Bytes,
    ) {
        {
            use ckb_types::packed;
            use ckb_types::prelude::*;
            if packed::RelayMessageReader::from_compatible_slice(&data).is_err() {
                if packed::SyncMessageReader::from_compatible_slice(&data).is_err() {
                    log::warn!(
                        "bilibili this message is not relay nor sync, protocol_id: {:?}",
                        nc.protocol_id()
                    );
                }
            };
        }

        if self.last_update_time.elapsed() < Duration::from_secs(5) {
            return;
        }

        for peer_index in nc.connected_peers() {
            if let Some(peer) = nc.get_peer(peer_index) {
                if let Some(ref identify_info) = peer.identify_info {
                    let socket_addr = multiaddr_to_socketaddr(&peer.connected_addr).unwrap();
                    let ip = socket_addr.ip().to_string();
                    let entry = entry::Peer {
                        network: self.node.consensus().id.clone(),
                        time: Utc::now().naive_utc(),
                        version: identify_info.client_version.clone(),
                        ip,
                        country: None,
                    };
                    let raw_query = format!(
                        "INSERT INTO peer(network, time, version, ip) \
            VALUES ('{}', '{}', '{}', '{}')",
                        entry.network, entry.time, entry.version, entry.ip,
                    );
                    self.query_sender.send(raw_query).unwrap();
                }
            }
        }
        self.last_update_time = Instant::now();
    }
}
