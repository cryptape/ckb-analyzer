use crossbeam::channel::bounded;
use influxdb::Client;
use std::env::var;

mod config;
mod dashboard;
mod get_version;
mod measurement;
mod role;
mod subscribe;

#[tokio::main]
async fn main() {
    let config = {
        let config_path = var("CKB_ANALYZER_CONFIG").unwrap_or_else(|_| {
            panic!("please specify config path via environment variable CKB_ANALYZER_CONFIG")
        });
        config::init_config(config_path)
    };
    let influx = {
        let username = var("INFLUXDB_USERNAME").unwrap_or_else(|_| "".to_string());
        let password = var("INFLUXDB_PASSWORD").unwrap_or_else(|_| "".to_string());
        if username.is_empty() {
            Client::new(
                config.influxdb.url.as_str(),
                config.influxdb.database.as_str(),
            )
        } else {
            Client::new(
                config.influxdb.url.as_str(),
                config.influxdb.database.as_str(),
            )
            .with_auth(&username, &password)
        }
    };
    let (query_sender, query_receiver) = bounded(5000);
    for (role_name, role) in config.roles.iter() {
        tokio::spawn(role.clone().run(
            role_name.clone(),
            config.ckb_network_name.clone(),
            influx.clone(),
            query_sender.clone(),
        ));
    }

    let hostname = var("HOSTNAME")
        .unwrap_or_else(|_| gethostname::gethostname().to_string_lossy().to_string());
    for mut query in query_receiver {
        // Attach built-in tags
        query = query
            .add_tag("network", config.ckb_network_name.clone())
            .add_tag("hostname", hostname.clone());

        // Writes asynchronously
        let asynchronize = true;
        if asynchronize {
            let influx_ = influx.clone();
            tokio::spawn(async move {
                if let Err(err) = influx_.query(&query).await {
                    log::error!("influxdb.query, error: {}", err);
                }
            });
        } else if let Err(err) = influx.query(&query).await {
            log::error!("influxdb.query, error: {}", err);
        }
    }
}
