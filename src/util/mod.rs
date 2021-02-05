pub mod log_watcher;

use crate::measurement;
use ckb_build_info::Version;
use ckb_suite_rpc::Jsonrpc;
use influxdb::{Client as Influx, ReadQuery};
use jsonrpc_server_utils::tokio::prelude::*;
pub use log_watcher::LogWatcher;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::SeqCst;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(18000);
const VERSION_CODE_NAME: &str = "probe";

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

// Just transform tokio 1.0 channel to crossbeam channel
// I don't know how to transform into tokio 2.0 channel
pub fn forward_tokio1_channel<T>(
    tokio1_receiver: jsonrpc_server_utils::tokio::sync::mpsc::Receiver<T>,
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

pub async fn select_last_block_number_in_influxdb(influx: &Influx, ckb_network_name: &str) -> u64 {
    let query_name = {
        let type_name = tynm::type_name::<measurement::Block>();
        case_style::CaseStyle::guess(type_name)
            .unwrap_or_else(|_| panic!("failed to convert the type name to kebabcase style"))
            .to_kebabcase()
    };
    let sql = format!(
        "SELECT last(number) FROM {query_name} WHERE network = '{ckb_network_name}'",
        query_name = query_name,
        ckb_network_name = ckb_network_name,
    );
    let query_last_number = ReadQuery::new(&sql);
    match influx.query(&query_last_number).await {
        Err(err) => {
            log::error!("influxdb.query(\"{}\"), error: {}", sql, err);
            ::std::process::exit(1);
        }
        Ok(results) => {
            let json: HashMap<String, serde_json::Value> = serde_json::from_str(&results).unwrap();
            let results = json.get("results").unwrap().as_array().unwrap();
            let result = results.get(0).unwrap().as_object().unwrap();
            if let Some(series) = result.get("series") {
                let series = series.as_array().unwrap();
                let serie = series.get(0).unwrap().as_object().unwrap();
                let values = serie.get("values").unwrap().as_array().unwrap();
                let value = values.get(0).unwrap().as_array().unwrap();
                value.get(1).unwrap().as_u64().unwrap()
            } else {
                1
            }
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
