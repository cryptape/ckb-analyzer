#[derive(Clone, Debug)]
pub struct Peer {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub version: String,
    pub ip: String,
    pub country: Option<String>,
}

impl Peer {
    pub fn insert_raw_peer_query(&self) -> String {
        format!(
            "INSERT INTO peer(network, time, version, ip) \
            VALUES ('{}', '{}', '{}', '{}')",
            &self.network, &self.time, &self.version, &self.ip,
        )
    }

    pub fn update_peer_country(id: i32, country: &str) -> String {
        format!("UPDATE peer SET country = '{}' WHERE id = {}", country, id,)
    }
}
