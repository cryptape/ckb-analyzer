/// Peer info
#[derive(Clone, Debug)]
pub struct Peer {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub version: String,
    pub ip: String,
    pub country: Option<String>,
}

/// Block Info
#[derive(Clone, Debug)]
pub struct Block {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub number: i64,
    pub n_transactions: i32,
    pub n_proposals: i32,
    pub n_uncles: i32,
    pub cellbase_message: Option<String>,

    pub interval: Option<i64>, // ms
}
