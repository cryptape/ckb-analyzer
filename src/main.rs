pub use config::{init_config, Config};
use crossbeam::channel::bounded;
use influxdb::Client;
use lazy_static::lazy_static;
use std::env::var;

mod analyzer;
mod app_config;
mod config;
mod dashboard;
mod get_version;
mod measurement;
mod subscribe;

lazy_static! {
    static ref LOG_LEVEL: String = var("LOG_LEVEL").unwrap_or_else(|_| "ERROR".to_string());
    static ref CONFIG: Config = {
        let config_path = var("CKB_ANALYZER_CONFIG").unwrap_or_else(|_| {
            panic!("please specify config path via environment variable CKB_ANALYZER_CONFIG")
        });
        init_config(config_path)
    };
    static ref HOSTNAME: String = var("HOSTNAME")
        .unwrap_or_else(|_| gethostname::gethostname().to_string_lossy().to_string());
    static ref INFLUXDB_USERNAME: String =
        var("INFLUXDB_USERNAME").unwrap_or_else(|_| "".to_string());
    static ref INFLUXDB_PASSWORD: String =
        var("INFLUXDB_PASSWORD").unwrap_or_else(|_| "".to_string());
}

#[tokio::main]
async fn main() {
    let influx = if INFLUXDB_USERNAME.is_empty() {
        Client::new(
            CONFIG.influxdb.url.as_str(),
            CONFIG.influxdb.database.as_str(),
        )
    } else {
        Client::new(
            CONFIG.influxdb.url.as_str(),
            CONFIG.influxdb.database.as_str(),
        )
        .with_auth(INFLUXDB_USERNAME.as_str(), INFLUXDB_PASSWORD.as_str())
    };
    let (query_sender, query_receiver) = bounded(5000);
    for analyzer in CONFIG.analyzers.iter() {
        tokio::spawn(analyzer.clone().run(
            CONFIG.ckb_network_name.as_str(),
            influx.clone(),
            query_sender.clone(),
        ));
    }

    for mut query in query_receiver {
        // Attach built-in tags
        query = query
            .add_tag("network", CONFIG.ckb_network_name.clone())
            .add_tag("hostname", HOSTNAME.clone());

        // Writes asynchronously
        let asynchronize = true;
        if asynchronize {
            let influx_ = influx.clone();
            tokio::spawn(async move {
                if let Err(err) = influx_.query(&query).await {
                    eprintln!("influxdb.query, error: {}", err);
                }
            });
        } else if let Err(err) = influx.query(&query).await {
            eprintln!("influxdb.query, error: {}", err);
        }
    }
}
