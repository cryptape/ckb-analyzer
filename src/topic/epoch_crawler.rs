use crate::ckb_types::core::EpochNumber;
use crate::entry;
use crate::tokio;
use ckb_testkit::Node;
use std::cmp::max;
use std::time::Duration;

pub struct EpochCrawler {
    node: Node,
    query_sender: crossbeam::channel::Sender<String>,
}

impl EpochCrawler {
    pub fn new(node: Node, query_sender: crossbeam::channel::Sender<String>) -> Self {
        Self { node, query_sender }
    }

    pub async fn run(&self, last_epoch_number: EpochNumber) {
        let mut current_number = max(1, last_epoch_number + 1);
        let mut tip_epoch = self.node.rpc_client().get_current_epoch();
        loop {
            if current_number >= tip_epoch.number.value() {
                tokio::time::sleep(Duration::from_secs(10)).await;
                tip_epoch = self.node.rpc_client().get_current_epoch();
                continue;
            }

            if let Some(epoch) = self.node.rpc_client().get_epoch_by_number(current_number) {
                if let Some(start_header) = self
                    .node
                    .rpc_client()
                    .get_header_by_number(epoch.start_number.value())
                {
                    if let Some(end_header) = self
                        .node
                        .rpc_client()
                        .get_header_by_number(epoch.start_number.value() + epoch.length.value() - 1)
                    {
                        let start_time = chrono::NaiveDateTime::from_timestamp(
                            (start_header.inner.timestamp.value() / 1000) as i64,
                            (start_header.inner.timestamp.value() % 1000 * 1000) as u32,
                        );
                        let end_time = chrono::NaiveDateTime::from_timestamp(
                            (end_header.inner.timestamp.value() / 1000) as i64,
                            (end_header.inner.timestamp.value() % 1000 * 1000) as u32,
                        );
                        let entry = entry::Epoch {
                            network: self.node.consensus().id.clone(),
                            number: epoch.number.value(),
                            length: epoch.length.value(),
                            start_number: epoch.start_number.value(),
                            start_time: start_time,
                            end_time: end_time,
                        };
                        let raw_query = format!(
                            "INSERT INTO {}.epoch (start_time, end_time, number, length, start_number) VALUES ('{}', '{}', {}, {}, {})",
                            entry.network,
                            entry.start_time,
                            entry.end_time,
                            entry.number,
                            entry.length,
                            entry.start_number,
                        );
                        self.query_sender.send(raw_query).unwrap();

                        current_number += 1;
                    }
                }
            }
        }
    }
}
