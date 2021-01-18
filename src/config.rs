use serde::{Deserialize, Serialize};
use tentacle_multiaddr::Multiaddr;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Config {
    pub network: String,
    pub topics: Vec<Topic>,
    pub influxdb: InfluxDBConfig,
    pub node: NodeConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum Topic {
    CanonicalChainState,
    Reorganization,
    TxTransition,
    PatternLogs,
    NetworkPropagation,
    NetworkTopology,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct InfluxDBConfig {
    pub url: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct NodeConfig {
    pub host: String,
    pub rpc_port: u16,
    pub subscription_port: u16,
    pub data_dir: String,
    pub bootnodes: Vec<Multiaddr>,
}

impl NodeConfig {
    pub fn rpc_url(&self) -> String {
        format!("http://{}:{}", self.host, self.rpc_port)
    }
    pub fn subscription_url(&self) -> String {
        format!("{}:{}", self.host, self.subscription_port)
    }
}
