use ckb_network::peer_store::types::IpPort;
use ckb_network::MultiaddrExt;
use ckb_suite_rpc::ckb_jsonrpc_types::{LocalNode, RemoteNode};
use ckb_suite_rpc::Jsonrpc;
use multiaddr::MultiAddr;
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};

fn network_nodes() -> Vec<String> {
    let urls = if crate::CKB_NETWORK.as_str() == "mainnet" {
        vec![
            "http://47.110.15.57:8124",
            "http://47.245.31.79:8124",
            "http://13.234.144.148:8124",
            "http://3.218.170.86:8124",
            "http://52.59.155.249:8124",
            "http://39.107.100.85:8124",
            "http://47.103.44.208:8124",
            "http://120.24.85.239:8124",
            "http://47.91.238.128:8124",
            "http://147.139.137.53:8124",
            "http://13.52.18.181:8124",
            "http://18.140.240.153:8124",
            "http://35.183.172.68:8124",
            "http://3.10.216.39:8124",
            "http://3.105.209.193:8124",
            "http://18.229.214.173:8124",
        ]
    } else if crate::CKB_NETWORK.as_str() == "testnet" {
        vec![
            "http://47.111.169.36:18121",
            "http://18.217.146.65:18121",
            "http://18.136.60.221:18121",
            "http://35.176.207.239:18121",
        ]
    } else {
        unimplemented!()
    };
    urls.into_iter().map(|url| url.to_string()).collect()
}

pub(crate) fn spawn_analyze() {
    let nodes = network_nodes();
    analyze(&nodes)
}

pub(crate) fn analyze<S: AsRef<str>>(ckb_urls: &[S]) {
    let mut connections = HashSet::new();

    let rpcs: Vec<_> = ckb_urls
        .into_iter()
        .map(|url| Jsonrpc::connect(url.as_ref()))
        .collect();
    for rpc in rpcs.iter() {
        let local = rpc.local_node_info();
        let local_addr = extract_socket_addr(&local.addresses[0].address);
        for remote in rpc.get_peers() {
            let remote_addr = extract_socket_addr(&remote.addresses[0].address);
            if local_addr > remote_addr {
                connections.insert((local_addr.clone(), remote_addr));
            } else {
                connections.insert((remote_addr, local_addr.clone()));
            }
        }
    }

    connections.iter().for_each(|(inbound, outbound)| {
        println!("{} --- {}", inbound, outbound);
    });
}

fn extract_socket_addr(address: &str) -> String {
    let multiaddr = address.parse::<MultiAddr>().unwrap();
    let ip_port = multiaddr.extract_ip_addr().unwrap();
    format!("{ip}:{port}", ip = ip_port.ip, port = ip_port.port)
}
