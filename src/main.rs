use ckb_suite_rpc::{ckb_types::core::BlockView, Jsonrpc};
use crossbeam::channel::{bounded, Sender};
use influxdb::{Client, InfluxDbWriteable, Timestamp};
use lazy_static::lazy_static;
use std::env::var;
use std::thread::spawn;
use std::time::Instant;

#[derive(InfluxDbWriteable)]
pub struct BlockSerie {
    time: Timestamp,

    number: u64,
    time_interval: u64, // ms
    uncles_count: u32,
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

    let (serie_sender, serie_receiver) = bounded(5000);
    spawn(move || {
        stream_series(serie_sender);
    });
    while let Ok(block_serie) = serie_receiver.recv() {
        let write_result = client.query(&block_serie.into_query("blocks")).await;
        assert!(write_result.is_ok(), "{:?}", write_result);
    }

    {
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
            let epoch_serie = EpochSerie {
                time: Timestamp::Seconds(start_timestamp as u128),
                length,
                duration: end_timestamp.saturating_sub(start_timestamp),
            };
            let write_result = client.query(&epoch_serie.into_query("epochs")).await;
            assert!(write_result.is_ok(), "{:?}", write_result);
        }
    }
}

fn stream_series(block_serie_sender: Sender<BlockSerie>) {
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
            let uncles_count = block.uncles().hashes().len() as u32;
            let block_serie = BlockSerie {
                time,
                number,
                time_interval,
                uncles_count,
            };
            block_serie_sender.send(block_serie).unwrap();

            previous = block;
        }
    }
}
