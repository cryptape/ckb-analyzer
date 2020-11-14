use ckb_suite_rpc::{ckb_types::core::BlockView, Jsonrpc};
use crossbeam::channel::{bounded, Sender};
use influxdb::{Client, InfluxDbWriteable, Timestamp, WriteQuery};
use lazy_static::lazy_static;
use std::env::var;
use std::thread::spawn;
use std::time::Instant;

mod network;

#[derive(InfluxDbWriteable)]
pub struct BlockSerie {
    time: Timestamp,

    number: u64,
    time_interval: u64, // ms
    transactions_count: u32,
    uncles_count: u32,
    proposals_count: u32,
}

#[derive(InfluxDbWriteable)]
pub struct EpochSerie {
    time: Timestamp,

    length: u64,
    duration: u64, // seconds
}

lazy_static! {
    static ref CKB_URL: String = var("CKB_URL").unwrap_or("http://0.0.0.0:8114".to_string());
    static ref INFLUXDB_URL: String =
        var("INFLUXDB_URL").unwrap_or("http://0.0.0.0:8086".to_string());
    static ref INFLUXDB_DATABASE: String = var("INFLUXDB_DATABASE").unwrap_or_else(|_| panic!(
        "please specify influxdb database name via environment variable INFLUXDB_DATABASE"
    ));
}

#[tokio::main]
async fn main() {
    let client = Client::new(INFLUXDB_URL.as_str(), INFLUXDB_DATABASE.as_str());
    if let Err(err) = client.ping().await {
        eprintln!("client.ping error: {:?}", err);
        return;
    }

    let (query_sender, query_receiver) = bounded(5000);
    let query_sender_ = query_sender.clone();
    spawn(move || analyze_blocks(query_sender_));
    spawn(move || analyze_epoches(query_sender));

    for query in query_receiver {
        let write_result = client.query(&query).await;
        assert!(
            write_result.is_ok(),
            "client.query({:?}), error: {:?}",
            query,
            write_result.unwrap_err()
        );
    }
}

fn analyze_epoches(query_sender: Sender<WriteQuery>) {
    let rpc = Jsonrpc::connect(CKB_URL.as_str());
    let current_epoch = rpc.get_current_epoch();
    for number in 0..current_epoch.number.value() {
        let epoch = rpc.get_epoch_by_number(number).unwrap();
        let length = epoch.length.value();
        let start_number: u64 = epoch.start_number.value();
        let end_number = start_number + length - 1;
        let start_header = rpc.get_header_by_number(start_number).unwrap();
        let end_header = rpc.get_header_by_number(end_number).unwrap();
        let start_timestamp = start_header.inner.timestamp.value() / 1000;
        let end_timestamp = end_header.inner.timestamp.value() / 1000;
        let write_query = EpochSerie {
            time: Timestamp::Seconds(start_timestamp as u128),
            length,
            duration: end_timestamp.saturating_sub(start_timestamp),
        }
        .into_query("epochs");
        query_sender.send(write_query).unwrap();
    }
}

fn analyze_blocks(query_sender: Sender<WriteQuery>) {
    let rpc = Jsonrpc::connect(CKB_URL.as_str());
    let start = Instant::now();

    let (from, to) = {
        let tip = rpc.get_tip_block_number();
        (tip.saturating_sub(100000), tip)
    };
    let mut previous: BlockView = rpc
        .get_block_by_number(from.saturating_sub(1))
        .unwrap()
        .into();
    for number in from..=to {
        if number % 10000 == 0 && number != from {
            let percent = (to - number + 1) as f64 / (number - from + 1) as f64;
            let left = start.elapsed().mul_f64(percent);
            println!(
                "Getting block #{number}, left {seconds}s ...",
                number = number,
                seconds = left.as_secs()
            );
        }

        if let Some(json_block) = rpc.get_block_by_number(number) {
            let block: BlockView = json_block.into();

            let time = Timestamp::Milliseconds(block.timestamp() as u128);
            let time_interval = block.timestamp().saturating_sub(previous.timestamp()); // ms
            let transactions_count = block.transactions().len() as u32;
            let uncles_count = block.uncles().hashes().len() as u32;
            let proposals_count = block.union_proposal_ids().len() as u32;
            let write_query = BlockSerie {
                time,
                number,
                time_interval,
                transactions_count,
                uncles_count,
                proposals_count,
            }
            .into_query("blocks");
            query_sender.send(write_query).unwrap();

            previous = block;
        }
    }
}
