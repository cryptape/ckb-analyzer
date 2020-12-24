use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub influxdb: InfluxdbConfig,
    pub chain: ChainConfig,
    pub network: NetworkConfig,
    pub topology: TopologyConfig,
    pub reorganization: OrganizationConfig,
    pub pool_transaction: PoolTransactionConfig,
    pub node_log: TailLogConfig,
    pub miner_log: TailLogConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct InfluxdbConfig {
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ChainConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ckb_rpc_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ckb_network_name: String,
    #[serde(default)]
    pub ckb_network_identifier: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TopologyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ckb_rpc_urls: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct OrganizationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ckb_rpc_url: String,
    #[serde(default)]
    pub ckb_subscription_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct PoolTransactionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ckb_rpc_url: String,
    #[serde(default)]
    pub ckb_subscription_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TailLogConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub filepath: String,
    #[serde(default)]
    pub classify: HashMap<String, String>, // #{ category => regex }
}

pub fn init_config<P: AsRef<Path>>(filepath: P) -> Config {
    let bytes =
        fs::read_to_string(filepath).unwrap_or_else(|err| panic!("fs::read error: {:?}", err));
    toml::from_str(&bytes).unwrap_or_else(|err| panic!("toml::from_str error: {:?}", err))
}
