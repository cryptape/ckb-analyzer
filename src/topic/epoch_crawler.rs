use crate::ckb_types::core::EpochNumber;
use crate::ckb_types::utilities::compact_to_difficulty;
use crate::entry;
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

            self.process(&mut current_number).await;
        }
    }

    async fn process(&self, current_number: &mut EpochNumber) {
        if let Some(epoch) = self.node.rpc_client().get_epoch_by_number(*current_number) {
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
                    let n_uncles = (start_header.inner.number.value()
                        ..=end_header.inner.number.value())
                        .map(|number| self.node.get_block_by_number(number).uncles().data().len())
                        .sum::<usize>();
                    let difficulty = compact_to_difficulty(epoch.compact_target.value());
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
                        start_time,
                        end_time,
                        n_uncles: n_uncles as i32,
                        difficulty: difficulty.to_string(),
                    };
                    let raw_query = format!(
                        "INSERT INTO {}.epoch (start_time, end_time, number, length, start_number, n_uncles, difficulty)\
                             VALUES ('{}', '{}', {}, {}, {}, {}, '{}')",
                        entry.network,
                        entry.start_time,
                        entry.end_time,
                        entry.number,
                        entry.length,
                        entry.start_number,
                        entry.n_uncles,
                        entry.difficulty,
                    );
                    self.query_sender.send(raw_query).unwrap();

                    *current_number += 1;
                }
            }
        }
    }
}
