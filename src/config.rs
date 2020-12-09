use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Config {
    pub chain: ChainConfig,
    pub network: NetworkConfig,
    pub topology: TopologyConfig,
    pub reorganization: OrganizationConfig,
    pub influxdb: InfluxdbConfig,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct OrganizationConfig {
    pub enabled: bool,
    pub ckb_rpc_url: String,
    pub ckb_subscription_url: String,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ChainConfig {
    #[serde(default)]
    pub enabled: bool,
    pub ckb_rpc_url: String,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct NetworkConfig {
    #[serde(default)]
    pub enabled: bool,
    pub ckb_network_name: String,
    pub ckb_network_identifier: String,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TopologyConfig {
    #[serde(default)]
    pub enabled: bool,
    pub ckb_rpc_urls: Vec<String>,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct InfluxdbConfig {
    pub database: String,
    pub url: String,
}

pub fn init_config<P: AsRef<Path>>(filepath: P) -> Config {
    let bytes =
        fs::read_to_string(filepath).unwrap_or_else(|err| panic!("fs::read error: {:?}", err));
    toml::from_str(&bytes).unwrap_or_else(|err| panic!("toml::from_str error: {:?}", err))
}
