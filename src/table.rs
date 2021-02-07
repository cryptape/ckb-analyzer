// TODO How to store u64 in Postgres?
// TODO Can we use "TIME" to represent duration?

/// # Table Schema
///
/// ```
/// CREATE TABLE IF NOT EXISTS block (
///     time            TIMESTAMP       NOT NULL,
///     number          BIGINT          NOT NULL,
///     interval        BIGINT          NOT NULL,
///     n_transactions  INT             NOT NULL,
///     n_proposals     INT             NOT NULL,
///     n_uncles        INT             NOT NULL,
///     hash            CHAR ( 34 )     NOT NULL,
///     miner           CHAR ( 34 )     NOT NULL,
///     version         INT             NOT NULL
/// );
///
/// SELECT create_hypertable('block', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Block {
    pub time: chrono::NaiveDateTime,
    pub number: i64,
    pub interval: i64, // ms
    pub n_transactions: i32,
    pub n_proposals: i32,
    pub n_uncles: i32,
    pub hash: String,  // hex hash
    pub miner: String, // hex hash
    pub version: i32,
}

/// # Table Schema
///
/// ```
/// CREATE TABLE IF NOT EXISTS uncle (
///     time                TIMESTAMP   NOT NULL,
///     number              BIGINT      NOT NULL,
///     lag_to_canonical    BIGINT      NOT NULL,
///     n_transactions      INT         NOT NULL,
///     n_proposals         INT         NOT NULL,
///     hash                CHAR ( 34 ) NOT NULL,
///     miner               CHAR ( 34 ) NOT NULL,
///     version             INT         NOT NULL
/// );
///
/// SELECT create_hypertable('uncle', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Uncle {
    pub time: chrono::NaiveDateTime,
    pub number: i64,
    pub lag_to_canonical: i64, // ms
    pub n_transactions: i32,
    pub n_proposals: i32,
    pub hash: String,  // hex hash
    pub miner: String, // hex hash
    pub version: i32,
}

/// # Table Schema
///
/// ```
/// CREATE TABLE IF NOT EXISTS two_pc_commitment (
///     time        TIMESTAMP       NOT NULL,
///     number      BIGINT          NOT NULL,
///     delay       INT             NOT NULL
/// );
///
/// SELECT create_hypertable('two_pc_commitment', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct TwoPCCommitment {
    pub time: chrono::NaiveDateTime,
    pub number: i64, // committed block number
    pub delay: i32,
}

/// # Table Schema
///
/// ```
/// CREATE TABLE IF NOT EXISTS epoch (
///     time        TIMESTAMP       NOT NULL,
///     number      BIGINT          NOT NULL,
///     length      INT             NOT NULL,
///     duration    INT             NOT NULL,
///     n_uncles    INT             NOT NULL
/// );
///
/// SELECT create_hypertable('epoch', 'time');
/// ```
pub struct Epoch {
    pub time: chrono::NaiveDateTime, // ended block timestamp
    pub number: i64,
    pub length: i32,
    pub duration: i32, // ms
    pub n_uncles: i32,
}
