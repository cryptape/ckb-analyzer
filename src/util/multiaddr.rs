use p2p::{multiaddr, utils::multiaddr_to_socketaddr};

pub fn addr_to_ip(addr: &multiaddr::Multiaddr) -> String {
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
