use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tentacle_multiaddr::Multiaddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub network: String,
    pub topics: Vec<Topic>,
    pub influxdb: InfluxDBConfig,
    pub postgres: String,
    pub node: NodeConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Topic {
    CanonicalChainState,
    Reorganization,
    TxTransition,
    PatternLogs,
    NetworkPropagation,
    NetworkTopology,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfluxDBConfig {
    pub url: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl Config {
    pub fn postgres(&self) -> tokio_postgres::Config {
        tokio_postgres::Config::from_str(&self.postgres).unwrap()
    }
}
