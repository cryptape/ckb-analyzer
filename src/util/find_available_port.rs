use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::SeqCst;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(18000);

pub fn find_available_port() -> u16 {
    for _ in 0..2000 {
        let port = PORT_COUNTER.fetch_add(1, SeqCst);
        let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
        if TcpListener::bind(address).is_ok() {
            return port;
        }
    }
    panic!("failed to allocate available port")
}
