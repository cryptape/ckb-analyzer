use crate::CONFIG;
use ckb_network::{
    MultiaddrExt,
    multiaddr::MultiAddr,
};
use ckb_suite_rpc::Jsonrpc;
use crossbeam::channel::Sender;
use influxdb::WriteQuery;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub(crate) fn spawn_analyze(query_sender: Sender<WriteQuery>) {
    spawn(move || {
        let ckb_urls = CONFIG.topology.ckb_urls.clone();
        loop {
            sleep(Duration::from_secs(60 * 10));
            analyze(&query_sender, &ckb_urls);
        }
    });
}

pub(crate) fn analyze<S: AsRef<str>>(_query_sender: &Sender<WriteQuery>, ckb_urls: &[S]) {
    let mut connections = HashSet::new();

    let rpcs: Vec<_> = ckb_urls
        .iter()
        .map(|url| Jsonrpc::connect(url.as_ref()))
        .collect();
    for rpc in rpcs.iter() {
        let local = rpc.local_node_info();
        let local_ip = extract_ip(&local.addresses[0].address);
        for remote in rpc.get_peers() {
            let remote_ip = extract_ip(&remote.addresses[0].address);
            if local_ip > remote_ip {
                connections.insert((local_ip.clone(), remote_ip));
            } else {
                connections.insert((remote_ip, local_ip.clone()));
            }
        }
    }

    println!("graph Topology {{");

    let known_ips = ckb_urls
        .iter()
        .map(|ckb_url| extract_ip(ckb_url.as_ref()))
        .collect::<HashSet<_>>();
    let dot2line = |ip: &str| {
        let new = ip.replace(".", "_");
        format!("host_{}", new)
    };

    {
        let mut counts = HashMap::new();
        connections.iter().for_each(|(inbound, outbound)| {
            if known_ips.contains(inbound) && known_ips.contains(outbound) {
                let entry = counts.entry(inbound.to_owned()).or_insert(0);
                *entry += 1;
                let entry = counts.entry(outbound.to_owned()).or_insert(0);
                *entry += 1;
            }
        });
        connections.iter().for_each(|(inbound, outbound)| {
            if let Some(count) = counts.remove(inbound) {
                println!(
                    "    {} [ label = \"{} *{}*\" ];",
                    dot2line(inbound),
                    inbound,
                    count
                );
            }
            if let Some(count) = counts.remove(outbound) {
                println!(
                    "    {} [ label = \"{} *{}*\" ];",
                    dot2line(outbound),
                    outbound,
                    count
                );
            }
        });
    }

    {
        connections.iter().for_each(|(inbound, outbound)| {
            if known_ips.contains(inbound) && known_ips.contains(outbound) {
                println!("    {} -- {};", dot2line(inbound), dot2line(outbound));
            }
        });
    }
    println!("}}");
}

// FIXME Use more accurate identifier
fn extract_ip(address: &str) -> String {
    if let Ok(multiaddr) = address.parse::<MultiAddr>() {
        let ip_port = multiaddr.extract_ip_addr().unwrap();
        ip_port.ip.to_string()
    } else if let Ok(socket_addr) = address[7..].parse::<SocketAddr>() {
        // FIXME ugly
        socket_addr.ip().to_string()
    } else {
        panic!("cannot parse {}", address)
    }
}
