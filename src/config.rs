use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tentacle_multiaddr::Multiaddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub network: String,
    #[serde(default)]
    pub ipinfo_io_token: Option<String>,
    pub topics: Vec<Topic>,
    pub postgres: String,
    pub node: NodeConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Topic {
    CanonicalChainState,
    Reorganization,
    TxTransition,
    NetworkPropagation,
    NetworkTopology,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeConfig {
    pub host: String,
    pub rpc_port: u16,
    pub subscription_port: u16,
    pub data_dir: String,
    pub bootnodes: Vec<Multiaddr>,
}

impl Config {
    pub fn postgres(&self) -> tokio_postgres::Config {
        tokio_postgres::Config::from_str(&self.postgres).unwrap()
    }

    pub fn network(&self) -> String {
        self.network.clone()
    }

    pub fn rpc_url(&self) -> String {
        format!("http://{}:{}", self.node.host, self.node.rpc_port)
    }

    pub fn subscription_url(&self) -> String {
        format!("{}:{}", self.node.host, self.node.subscription_port)
    }
}
