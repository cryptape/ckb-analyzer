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

/// Tx Pool Info
#[derive(Clone, Debug)]
pub struct TxPoolInfo {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub total_tx_cycles: i64,
    pub total_tx_size: i64,
    pub pending: i64,
    pub proposed: i64,
    pub orphan: i64,
}

/// Block Transaction Info
#[derive(Clone, Debug)]
pub struct BlockTransaction {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub number: i64,
    pub size: i32,
    pub n_inputs: i32,
    pub n_outputs: i32,
    pub n_header_deps: i32,
    pub n_cell_deps: i32,
    pub total_data_size: i32,
    pub proposal_id: String,
    pub hash: String,
}
