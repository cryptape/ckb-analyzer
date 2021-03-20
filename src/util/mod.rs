use crate::tokio01::prelude::*;
use ckb_build_info::Version;
use ckb_suite_rpc::Jsonrpc;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::sync::atomic::{AtomicU16, Ordering::SeqCst};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(18000);
const VERSION_CODE_NAME: &str = "probe";

#[allow(dead_code)]
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

// Just transform tokio 0.1 channel to crossbeam channel
// I don't know how to transform into tokio 0.2 channel
pub fn forward_tokio1_channel<T>(
    tokio1_receiver: crate::tokio01::sync::mpsc::Receiver<T>,
) -> crossbeam::channel::Receiver<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = crossbeam::channel::bounded(100);
    ::std::thread::spawn(move || {
        tokio1_receiver
            .for_each(|item| Ok(sender.send(item).unwrap()))
            .wait()
            .unwrap()
    });
    receiver
}

pub async fn get_last_updated_block_number(
    pg: &tokio_postgres::Client,
    ckb_network_name: &str,
) -> u64 {
    match pg
        .query_opt(
            "SELECT number FROM block WHERE network = $1 ORDER BY time DESC LIMIT 1",
            &[&ckb_network_name],
        )
        .await
        .unwrap()
    {
        None => 0,
        Some(raw) => {
            let number: i64 = raw.get(0);
            number as u64
        }
    }
}

pub fn get_network_identifier(jsonrpc: &Jsonrpc) -> String {
    let consensus = jsonrpc.get_consensus();
    let genesis_hash = format!("{:x}", consensus.genesis_hash);
    format!("/{}/{}", consensus.id, &genesis_hash[..8])
}

// Copy from https://github.com/nervosnetwork/ckb/blob/develop/src/main.rs
pub fn get_version() -> Version {
    let major = env!("CARGO_PKG_VERSION_MAJOR")
        .parse::<u8>()
        .expect("CARGO_PKG_VERSION_MAJOR parse success");
    let minor = env!("CARGO_PKG_VERSION_MINOR")
        .parse::<u8>()
        .expect("CARGO_PKG_VERSION_MINOR parse success");
    let patch = env!("CARGO_PKG_VERSION_PATCH")
        .parse::<u16>()
        .expect("CARGO_PKG_VERSION_PATCH parse success");
    let dash_pre = {
        let pre = env!("CARGO_PKG_VERSION_PRE");
        if pre == "" {
            pre.to_string()
        } else {
            "-".to_string() + pre
        }
    };

    let commit_describe = option_env!("COMMIT_DESCRIBE").map(ToString::to_string);
    #[cfg(docker)]
    let commit_describe = commit_describe.map(|s| s.replace("-dirty", ""));
    let commit_date = option_env!("COMMIT_DATE").map(ToString::to_string);
    let code_name = Some(VERSION_CODE_NAME.to_string());
    Version {
        major,
        minor,
        patch,
        dash_pre,
        code_name,
        commit_describe,
        commit_date,
    }
}
