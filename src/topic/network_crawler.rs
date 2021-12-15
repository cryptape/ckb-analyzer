use ckb_testkit::{
    ckb_types::{packed, prelude::*},
    compress,
    connector::SharedState,
    decompress, Node, SupportProtocols,
};
use p2p::{
    builder::MetaBuilder as P2PMetaBuilder,
    bytes::{Bytes, BytesMut},
    context::ProtocolContext as P2PProtocolContext,
    context::ProtocolContextMutRef as P2PProtocolContextMutRef,
    context::ServiceContext as P2PServiceContext,
    context::SessionContext,
    multiaddr,
    multiaddr::Multiaddr,
    service::ProtocolHandle as P2PProtocolHandle,
    service::ProtocolMeta as P2PProtocolMeta,
    service::ServiceError as P2PServiceError,
    service::ServiceEvent as P2PServiceEvent,
    service::TargetProtocol as P2PTargetProtocol,
    traits::ServiceHandle as P2PServiceHandle,
    traits::ServiceProtocol as P2PServiceProtocol,
    utils::multiaddr_to_socketaddr,
};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio_util::codec::{length_delimited::LengthDelimitedCodec, Decoder, Encoder};

// TODO Adjust the parameters
const DIAL_ONLINE_ADDRESSES_INTERVAL: Duration = Duration::from_secs(1);
const PRUNE_OFFLINE_ADDRESSES_INTERVAL: Duration = Duration::from_secs(30 * 60);
const DISCONNECT_TIMEOUT_SESSION_INTERVAL: Duration = Duration::from_secs(10);
const POSTGRES_ONLINE_ADDRESS_INTERVAL: Duration = Duration::from_secs(60);
const DIAL_ONLINE_ADDRESSES_TOKEN: u64 = 1;
const PRUNE_OFFLINE_ADDRESSES_TOKEN: u64 = 2;
const DISCONNECT_TIMEOUT_SESSION_TOKEN: u64 = 3;
const POSTGRES_ONLINE_ADDRESSES_TOKEN: u64 = 4;

const ADDRESS_TIMEOUT: Duration = Duration::from_secs(30);

/// NetworkCrawler crawl the network reachability info.
///
/// This service opens 2 protocols, Identify and Discovery:
///
/// * A ticker to trigger dialing observed addresses
/// * A ticker to trigger pruning timeout sessions
/// * TODO A ticker to trigger pruning offline addresses
/// * When opening Identify protocol on a session, reject it if its session type is inbound or
/// identify name is "CKBAnalyzer", record into `self.online_nodes`.
/// * When opening Discovery protocol on a session, send `GetNodes` message.
/// * When receiving inv `Nodes`, record into `self.reachable`
pub struct NetworkCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
    shared: Arc<RwLock<SharedState>>,

    // all observed addresses
    observed_addresses: Arc<RwLock<HashSet<Multiaddr>>>,

    // #{ ip => peer_info }
    online: Arc<RwLock<HashMap<Ip, PeerInfo>>>,
}

pub type Ip = String;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    address: Multiaddr,
    last_seen_time: Option<Instant>,
    reachable: HashSet<Ip>,
    client_version: String,
}

impl Clone for NetworkCrawler {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            query_sender: self.query_sender.clone(),
            shared: Arc::clone(&self.shared),
            observed_addresses: Arc::clone(&self.observed_addresses),
            online: Arc::clone(&self.online),
        }
    }
}

impl NetworkCrawler {
    /// Create a NetworkCrawler
    pub fn new(
        node: Node,
        query_sender: crossbeam::channel::Sender<String>,
        shared: Arc<RwLock<SharedState>>,
    ) -> Self {
        let bootnode = match node.consensus().id.as_str() {
            "ckb" => {
                "/ip4/47.110.15.57/tcp/8114/p2p/QmXS4Kbc9HEeykHUTJCm2tNmqghbvWyYpUp6BtE5b6VrAU"
            }
            "ckb_testnet" => {
                "/ip4/47.111.169.36/tcp/8111/p2p/QmNQ4jky6uVqLDrPU7snqxARuNGWNLgSrTnssbRuy3ij2W"
            }
            _ => unreachable!(),
        };
        let mut bootnodes = HashSet::new();
        bootnodes.insert(bootnode.parse().unwrap());
        Self {
            node,
            query_sender,
            shared,
            observed_addresses: Arc::new(RwLock::new(bootnodes.clone())),
            online: Arc::new(RwLock::new(
                bootnodes
                    .into_iter()
                    .map(|address| {
                        (
                            addr_to_ip(&address),
                            PeerInfo {
                                address,
                                last_seen_time: Default::default(),
                                reachable: Default::default(),
                                client_version: Default::default(),
                            },
                        )
                    })
                    .collect(),
            )),
        }
    }

    /// Convert NetworkCrawler into P2PProtocolMeta
    pub fn build_protocol_metas(&self) -> Vec<P2PProtocolMeta> {
        vec![
            {
                let meta_builder: P2PMetaBuilder = SupportProtocols::Identify.into();
                meta_builder
                    .service_handle(move || P2PProtocolHandle::Callback(Box::new(self.clone())))
                    .build()
            },
            {
                let meta_builder: P2PMetaBuilder = SupportProtocols::Discovery.into();
                meta_builder
                    .service_handle(move || P2PProtocolHandle::Callback(Box::new(self.clone())))
                    .build()
            },
            {
                // Necessary to communicate with CKB full node
                let meta_builder: P2PMetaBuilder = SupportProtocols::Sync.into();
                meta_builder
                    // Only Timer, Sync, Relay make compress
                    .before_send(compress)
                    .before_receive(|| Some(Box::new(decompress)))
                    .service_handle(move || P2PProtocolHandle::Callback(Box::new(self.clone())))
                    .build()
            },
        ]
    }

    fn received_identify(&mut self, context: P2PProtocolContextMutRef, data: Bytes) {
        match packed::IdentifyMessage::from_compatible_slice(data.as_ref()) {
            Ok(message) => {
                match packed::Identify::from_compatible_slice(
                    message.identify().raw_data().as_ref(),
                ) {
                    Ok(identify_payload) => {
                        let client_version_vec: Vec<u8> =
                            identify_payload.client_version().unpack();
                        let client_version =
                            String::from_utf8_lossy(&client_version_vec).to_string();
                        ckb_testkit::debug!(
                            "NetworkCrawler received IdentifyMessage, address: {}, time: {:?}",
                            context.session.address,
                            Instant::now()
                        );
                        if let Ok(mut online) = self.online.write() {
                            let entry = online
                                .entry(addr_to_ip(&context.session.address))
                                .or_insert_with(|| PeerInfo {
                                    address: context.session.address.clone(),
                                    last_seen_time: Default::default(),
                                    reachable: Default::default(),
                                    client_version: Default::default(),
                                });
                            entry.client_version = client_version;
                            entry.last_seen_time = Some(Instant::now());
                        }
                    }
                    Err(err) => {
                        ckb_testkit::error!("NetworkCrawler received invalid Identify Payload, session: {:?}, error: {:?}", context.session, err);
                    }
                }
            }
            Err(err) => {
                ckb_testkit::error!(
                    "NetworkCrawler received invalid IdentifyMessage, session: {:?}, error: {:?}",
                    context.session,
                    err
                );
            }
        }
    }

    fn received_discovery(&mut self, context: P2PProtocolContextMutRef, data: Bytes) {
        match packed::DiscoveryMessage::from_compatible_slice(data.as_ref()) {
            Ok(message) => {
                match message.payload().to_enum() {
                    packed::DiscoveryPayloadUnion::Nodes(discovery_nodes) => {
                        ckb_testkit::debug!(
                            "NetworkCrawler received DiscoveryMessages Nodes, session: {:?}, nodes.len: {}",
                            context.session,
                            discovery_nodes.items().len(),
                        );

                        if let Ok(mut observed_addresses) = self.observed_addresses.write() {
                            for node in discovery_nodes.items() {
                                for address in node.addresses() {
                                    if let Ok(addr) =
                                        Multiaddr::try_from(address.raw_data().to_vec())
                                    {
                                        if observed_addresses.insert(addr.clone()) {
                                            ckb_testkit::debug!(
                                                "NetworkCrawler observed new address: {}",
                                                addr
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        {
                            if let Ok(mut online) = self.online.write() {
                                let entry = online
                                    .entry(addr_to_ip(&context.session.address))
                                    .or_insert_with(|| PeerInfo {
                                        address: context.session.address.clone(),
                                        last_seen_time: Default::default(),
                                        reachable: Default::default(),
                                        client_version: Default::default(),
                                    });
                                for node in discovery_nodes.items() {
                                    for address in node.addresses() {
                                        if let Ok(addr) =
                                            Multiaddr::try_from(address.raw_data().to_vec())
                                        {
                                            entry.reachable.insert(addr_to_ip(&addr));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    packed::DiscoveryPayloadUnion::GetNodes(_discovery_get_nodes) => {
                        // discard
                    }
                }
            }
            Err(err) => {
                // ckb2019, before hard fork
                let mut data = BytesMut::from(data.as_ref());
                let mut codec = LengthDelimitedCodec::new();
                match codec.decode(&mut data) {
                    Ok(Some(frame)) => self.received_discovery(context, frame.freeze()),
                    _ => {
                        ckb_testkit::error!(
                            "NetworkCrawler received invalid DiscoveryMessage, session: {:?}, error: {:?}",
                            context.session,
                            err
                        );
                    }
                }
            }
        }
    }

    fn connected_discovery(&mut self, context: P2PProtocolContextMutRef, protocol_version: &str) {
        let discovery_get_node_message = packed::DiscoveryMessage::new_builder()
            .payload(
                packed::DiscoveryPayload::new_builder()
                    .set(
                        packed::GetNodes::new_builder()
                            .count(1000u32.pack())
                            .version(1u32.pack())
                            .build(),
                    )
                    .build(),
            )
            .build();
        if protocol_version == "0.0.1" {
            let mut codec = LengthDelimitedCodec::new();
            let mut bytes = BytesMut::new();
            codec
                .encode(discovery_get_node_message.as_bytes(), &mut bytes)
                .expect("encode must be success");
            let message_bytes = bytes.freeze();
            context.send_message(message_bytes).unwrap();
        } else {
            let message_bytes = discovery_get_node_message.as_bytes();
            context.send_message(message_bytes).unwrap();
        }
    }

    fn connected_identify(&mut self, context: P2PProtocolContextMutRef, _protocol_version: &str) {
        ckb_testkit::debug!(
            "NetworkCrawler open Identify protocol, session: {:?}",
            context.session
        );
        // It's okay for us that don't send IdentifyMessage to peer.
        // Just discard it.
    }
}

impl P2PServiceProtocol for NetworkCrawler {
    fn init(&mut self, context: &mut P2PProtocolContext) {
        if context.proto_id == SupportProtocols::Sync.protocol_id() {
            context
                .set_service_notify(
                    SupportProtocols::Sync.protocol_id(),
                    DIAL_ONLINE_ADDRESSES_INTERVAL,
                    DIAL_ONLINE_ADDRESSES_TOKEN,
                )
                .unwrap();
            context
                .set_service_notify(
                    SupportProtocols::Sync.protocol_id(),
                    PRUNE_OFFLINE_ADDRESSES_INTERVAL,
                    PRUNE_OFFLINE_ADDRESSES_TOKEN,
                )
                .unwrap();
            context
                .set_service_notify(
                    SupportProtocols::Sync.protocol_id(),
                    DISCONNECT_TIMEOUT_SESSION_INTERVAL,
                    DISCONNECT_TIMEOUT_SESSION_TOKEN,
                )
                .unwrap();
            context
                .set_service_notify(
                    SupportProtocols::Sync.protocol_id(),
                    POSTGRES_ONLINE_ADDRESS_INTERVAL,
                    POSTGRES_ONLINE_ADDRESSES_TOKEN,
                )
                .unwrap();
        }
    }

    fn notify(&mut self, context: &mut P2PProtocolContext, token: u64) {
        match token {
            DIAL_ONLINE_ADDRESSES_TOKEN => {
                // TODO reset notify to adjust the length of observed_addresses
                // context.remove_service_notify();
                // context.set_service_notify();
                let mut rng = thread_rng();
                if let Ok(observed_addresses) = self.observed_addresses.write() {
                    let random_index = rng.gen_range(0..observed_addresses.len());
                    let random_address = observed_addresses
                        .iter()
                        .collect::<Vec<_>>()
                        .get(random_index)
                        .cloned()
                        .unwrap();
                    let _ = context.dial((*random_address).clone(), P2PTargetProtocol::All);
                };
            }
            DISCONNECT_TIMEOUT_SESSION_TOKEN => {
                let sessions = {
                    self.shared
                        .read()
                        .map(|shared| {
                            shared
                                .get_sessions()
                                .iter()
                                .map(|s| (*s).clone())
                                .collect::<Vec<SessionContext>>()
                        })
                        .unwrap_or_default()
                };
                if let Ok(online) = self.online.read() {
                    for session in sessions {
                        if let Some(peer_info) = online.get(&addr_to_ip(&session.address)) {
                            if let Some(last_seen_time) = peer_info.last_seen_time {
                                if last_seen_time.elapsed() > Duration::from_secs(10) {
                                    let _ = context.disconnect(session.id);
                                }
                            }
                        }
                    }
                }
            }
            POSTGRES_ONLINE_ADDRESSES_TOKEN => {
                let now = chrono::Utc::now().naive_utc();
                let mut entries = Vec::new();
                if let Ok(online) = self.online.read() {
                    for (ip, peer_info) in online.iter() {
                        if let Some(last_seen_time) = peer_info.last_seen_time {
                            if last_seen_time.elapsed() <= ADDRESS_TIMEOUT {
                                // It's a online address
                                let n_reachable = {
                                    peer_info
                                        .reachable
                                        .iter()
                                        .filter(|ip1| online.contains_key(*ip1))
                                        .count()
                                };
                                let entry = crate::entry::Peer {
                                    network: self.node.consensus().id.clone(),
                                    time: now,
                                    version: peer_info.client_version.clone(),
                                    ip: ip.clone(),
                                    n_reachable: n_reachable as i32,
                                    country: None,
                                };
                                entries.push(entry);
                            }
                        }
                    }
                }

                for entry in entries {
                    let raw_query = format!(
                        "INSERT INTO {}.peer(time, version, ip, n_reachable) \
                             VALUES ('{}', '{}', '{}', {})",
                        entry.network, entry.time, entry.version, entry.ip, entry.n_reachable,
                    );
                    self.query_sender.send(raw_query).unwrap();
                }
            }
            PRUNE_OFFLINE_ADDRESSES_TOKEN => {
                // TODO prune offline addresses
            }
            _ => unreachable!(),
        }
    }

    fn connected(&mut self, context: P2PProtocolContextMutRef, protocol_version: &str) {
        if let Ok(mut shared) = self.shared.write() {
            shared.add_protocol(context.session, context.proto_id);
        }

        if context.proto_id() == SupportProtocols::Discovery.protocol_id() {
            self.connected_discovery(context, protocol_version)
        } else if context.proto_id() == SupportProtocols::Identify.protocol_id() {
            self.connected_identify(context, protocol_version)
        } else if context.proto_id() == SupportProtocols::Sync.protocol_id() {
            // discard
        } else {
            unreachable!()
        }
    }

    fn disconnected(&mut self, context: P2PProtocolContextMutRef) {
        if let Ok(mut shared) = self.shared.write() {
            shared.remove_protocol(&context.session.id, &context.proto_id());
        }
    }

    fn received(&mut self, context: P2PProtocolContextMutRef, data: Bytes) {
        if context.proto_id == SupportProtocols::Discovery.protocol_id() {
            self.received_discovery(context, data)
        } else if context.proto_id == SupportProtocols::Identify.protocol_id() {
            self.received_identify(context, data)
        } else if context.proto_id() == SupportProtocols::Sync.protocol_id() {
            // discard
        } else {
            unreachable!()
        }
    }
}

impl P2PServiceHandle for NetworkCrawler {
    fn handle_error(&mut self, _context: &mut P2PServiceContext, error: P2PServiceError) {
        match &error {
            P2PServiceError::DialerError { .. } | P2PServiceError::ProtocolSelectError { .. } => {
                // discard
            }
            _ => {
                ckb_testkit::error!("NetworkCrawler detect service error, error: {:?}", error);
            }
        }
    }

    /// Handling session establishment and disconnection events
    fn handle_event(&mut self, context: &mut P2PServiceContext, event: P2PServiceEvent) {
        match event {
            P2PServiceEvent::SessionOpen {
                session_context: session,
            } => {
                ckb_testkit::debug!("NetworkCrawler open session: {:?}", session);
                // Reject passive connection
                if session.ty.is_inbound() {
                    let _ = context.disconnect(session.id);
                    return;
                }

                let _ = self
                    .shared
                    .write()
                    .map(|mut shared| shared.add_session(session.as_ref().to_owned()));
            }
            P2PServiceEvent::SessionClose {
                session_context: session,
            } => {
                ckb_testkit::debug!("NetworkCrawler close session: {:?}", session);
                let _ = self
                    .shared
                    .write()
                    .map(|mut shared| shared.remove_session(&session.id));
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

fn addr_to_ip(addr: &Multiaddr) -> Ip {
    addr.iter()
        .find_map(|protocol| match protocol {
            multiaddr::Protocol::Ip4(ip4) => Some(ip4.to_string()),
            multiaddr::Protocol::Ip6(ip6) => ip6
                .to_ipv4()
                .map(|ip4| ip4.to_string())
                .or_else(|| Some(ip6.to_string())),
            multiaddr::Protocol::Dns4(dns4) => Some(dns4.to_string()),
            multiaddr::Protocol::Dns6(dns6) => Some(dns6.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| {
            let socket_addr = multiaddr_to_socketaddr(&addr).unwrap();
            socket_addr.ip().to_string()
        })
}
