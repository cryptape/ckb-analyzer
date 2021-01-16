use crate::measurement;
use influxdb::{Client as Influx, ReadQuery};
use std::collections::HashMap;

pub async fn select_last_block_number_in_influxdb(influx: &Influx, ckb_network_name: &str) -> u64 {
    let query_name = {
        let type_name = tynm::type_name::<measurement::Block>();
        case_style::CaseStyle::guess(type_name)
            .unwrap_or_else(|_| panic!("failed to convert the type name to kebabcase style"))
            .to_kebabcase()
    };
    let sql = format!(
        "SELECT last(number) FROM {query_name} WHERE network = '{ckb_network_name}'",
        query_name = query_name,
        ckb_network_name = ckb_network_name,
    );
    let query_last_number = ReadQuery::new(&sql);
    match influx.query(&query_last_number).await {
        Err(err) => {
            log::error!("influxdb.query(\"{}\"), error: {}", sql, err);
            ::std::process::exit(1);
        }
        Ok(results) => {
            let json: HashMap<String, serde_json::Value> = serde_json::from_str(&results).unwrap();
            let results = json.get("results").unwrap().as_array().unwrap();
            let result = results.get(0).unwrap().as_object().unwrap();
            if let Some(series) = result.get("series") {
                let series = series.as_array().unwrap();
                let serie = series.get(0).unwrap().as_object().unwrap();
                let values = serie.get("values").unwrap().as_array().unwrap();
                let value = values.get(0).unwrap().as_array().unwrap();
                value.get(1).unwrap().as_u64().unwrap()
            } else {
                1
            }
        }
    }
}
