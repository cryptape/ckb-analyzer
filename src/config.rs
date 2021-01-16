use crate::topic::Topic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Foo {
    Logs { path: String, pattern: String },
    Logs2 { path: String, pattern: String },
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
