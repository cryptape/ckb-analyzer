use crate::topic::Topic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Foo  {
    Logs {
        path: String,
        pattern: String,
    },
    Logs2 {
        path: String,
        pattern: String,
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ckb_network_name: String,
    pub topics: HashMap<String, Topic>,
    pub influxdb: InfluxdbConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfluxdbConfig {
    pub database: String,
    pub url: String,
}

pub fn init_config<P: AsRef<Path>>(filepath: P) -> Config {
    let bytes =
        fs::read_to_string(filepath).unwrap_or_else(|err| panic!("fs::read error: {:?}", err));
    toml::from_str(&bytes).unwrap_or_else(|err| panic!("toml::from_str error: {:?}", err))
}
