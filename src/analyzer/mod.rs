use crossbeam::channel::Sender;
use influxdb::{Client as Influx, WriteQuery};
use serde::{Deserialize, Serialize};

mod reorganization;
mod main_chain;
mod network_probe;
mod network_topology;

pub use reorganization::{Reorganization, ReorganizationConfig};
pub use main_chain::{select_last_block_number_in_influxdb, MainChain, MainChainConfig};
pub use network_probe::NetworkProbe;
pub use network_topology::{NetworkTopology, NetworkTopologyConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Analyzer {
    MainChain(MainChainConfig),
    NetworkProbe,
    NetworkTopology(NetworkTopologyConfig),
    Reorganization(ReorganizationConfig),
}

impl Analyzer {
    pub async fn run(self, influx: Influx, query_sender: Sender<WriteQuery>) {
        match self {
            Self::MainChain(config) => {
                let last_number = select_last_block_number_in_influxdb(&influx).await;
                MainChain::new(config, query_sender, last_number)
                    .run()
                    .await
            }
            Self::NetworkProbe => NetworkProbe::new(query_sender).run().await,
            Self::NetworkTopology(config) => NetworkTopology::new(config).run().await,
            Self::Reorganization(config) => {
                let (reorganization, subscription) = Reorganization::init(config, query_sender);

                // IMPORTANT: Use tokio 1.0 to run subscription. Since jsonrpc has not support 2.0 yet
                ::std::thread::spawn(move || {
                    jsonrpc_server_utils::tokio::run(subscription.run());
                });

                // // PROBLEM: With delaying a while, both tasks subscription and reorganization will run;
                // // But without delaying, only the task reorganization will run.
                // tokio::spawn(async { subscription.run().await });
                // tokio::time::delay_for(::std::time::Duration::from_secs(3)).await;

                reorganization.run().await;
            }
        }
    }
}
