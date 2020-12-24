use crate::analyzer::Analyzer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub ckb_network_name: String,
    pub influxdb: InfluxdbConfig,
    #[serde(default)]
    pub analyzers: Vec<Analyzer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct InfluxdbConfig {
    pub database: String,
    pub url: String,
}

pub fn init_config<P: AsRef<Path>>(filepath: P) -> Config {
    let bytes =
        fs::read_to_string(filepath).unwrap_or_else(|err| panic!("fs::read error: {:?}", err));
    toml::from_str(&bytes).unwrap_or_else(|err| panic!("toml::from_str error: {:?}", err))
}
