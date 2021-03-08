// TODO How to store u64 in Postgres?
// TODO Can we use "TIME" to represent duration?

use std::fmt::Debug;
use tokio_postgres::types::ToSql;

pub trait Point: Send + Debug {
    fn name(&self) -> &'static str;
    fn insert_query(&self) -> &'static str;
    fn params(&self) -> Vec<&(dyn ToSql + Sync)>;
}

/// ```
/// CREATE TABLE IF NOT EXISTS block (
///     network         VARCHAR ( 10 )  NOT NULL,
///     time            TIMESTAMP       NOT NULL,
///     number          BIGINT          NOT NULL,
///     interval        BIGINT          NOT NULL,
///     n_transactions  INT             NOT NULL,
///     n_proposals     INT             NOT NULL,
///     n_uncles        INT             NOT NULL,
///     hash            CHAR ( 66 )     NOT NULL,
///     miner           CHAR ( 66 )     NOT NULL,
///     version         INT             NOT NULL
/// );
///
/// SELECT create_hypertable('block', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Block {
    pub network: String,
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

impl Point for Block {
    fn name(&self) -> &'static str {
        "block"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO block (network, time, number, interval, n_transactions, n_proposals, n_uncles, hash, miner, version)\
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.number,
            &self.interval,
            &self.n_transactions,
            &self.n_proposals,
            &self.n_uncles,
            &self.hash,
            &self.miner,
            &self.version,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS uncle (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP   NOT NULL,
///     number              BIGINT      NOT NULL,
///     lag_to_canonical    BIGINT      NOT NULL,
///     n_transactions      INT         NOT NULL,
///     n_proposals         INT         NOT NULL,
///     hash                CHAR ( 66 ) NOT NULL,
///     miner               CHAR ( 66 ) NOT NULL,
///     version             INT         NOT NULL
/// );
///
/// SELECT create_hypertable('uncle', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Uncle {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub number: i64,
    pub lag_to_canonical: i64, // ms
    pub n_transactions: i32,
    pub n_proposals: i32,
    pub hash: String,  // hex hash
    pub miner: String, // hex hash
    pub version: i32,
}

impl Point for Uncle {
    fn name(&self) -> &'static str {
        "uncle"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO uncle (network, time, number, lag_to_canonical, n_transactions, n_proposals, hash, miner, version) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.number,
            &self.lag_to_canonical,
            &self.n_transactions,
            &self.n_proposals,
            &self.hash,
            &self.miner,
            &self.version,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS two_pc_commitment (
///     network     VARCHAR ( 10 )  NOT NULL,
///     time        TIMESTAMP       NOT NULL,
///     number      BIGINT          NOT NULL,
///     delay       INT             NOT NULL
/// );
///
/// SELECT create_hypertable('two_pc_commitment', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct TwoPCCommitment {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub number: i64, // committed block number
    pub delay: i32,
}

impl Point for TwoPCCommitment {
    fn name(&self) -> &'static str {
        "two_pc_commitment"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO two_pc_commitment (network, time, number, delay) \
            VALUES ($1, $2, $3, $4)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.network, &self.time, &self.number, &self.delay]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS epoch (
///     network     VARCHAR ( 10 )  NOT NULL,
///     time        TIMESTAMP       NOT NULL,
///     number      BIGINT          NOT NULL,
///     length      INT             NOT NULL,
///     duration    INT             NOT NULL,
///     n_uncles    INT             NOT NULL
/// );
///
/// SELECT create_hypertable('epoch', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Epoch {
    pub network: String,
    pub time: chrono::NaiveDateTime, // ended block timestamp
    pub number: i64,
    pub length: i32,
    pub duration: i32, // ms
    pub n_uncles: i32,
}

impl Point for Epoch {
    fn name(&self) -> &'static str {
        "epoch"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO epoch (network, time, number, length, duration, n_uncles) VALUES ($1, $2, $3, $4, $5, $6)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.number,
            &self.length,
            &self.duration,
            &self.n_uncles,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS reorganization (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     attached_length     INT             NOT NULL,
///     old_tip_number      BIGINT          NOT NULL,
///     new_tip_number      BIGINT          NOT NULL,
///     ancestor_number     BIGINT          NOT NULL,
///     old_tip_hash        CHAR ( 66 )     NOT NULL,
///     new_tip_hash        CHAR ( 66 )     NOT NULL,
///     ancestor_hash       CHAR ( 66 )     NOT NULL
/// );
///
/// SELECT create_hypertable('reorganization', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Reorganization {
    pub network: String,
    // timestamp of the fixed common block of forks
    pub time: chrono::NaiveDateTime,

    pub attached_length: i32,
    pub old_tip_number: i64,
    pub new_tip_number: i64,
    pub ancestor_number: i64,

    pub old_tip_hash: String,
    pub new_tip_hash: String,
    pub ancestor_hash: String,
}

impl Point for Reorganization {
    fn name(&self) -> &'static str {
        "reorganization"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO reorganization (network, time, attached_length, old_tip_number, \
            new_tip_number, ancestor_number, old_tip_hash, new_tip_hash, ancestor_hash)\
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.attached_length,
            &self.old_tip_number,
            &self.new_tip_number,
            &self.ancestor_number,
            &self.old_tip_hash,
            &self.new_tip_hash,
            &self.ancestor_hash,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS transaction (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     elapsed             BIGINT          NOT NULL,
///     event               CHAR ( 10 )     NOT NULL,
///     hash                CHAR ( 66 )     NOT NULL
/// );
///
/// SELECT create_hypertable('transaction', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Transaction {
    pub network: String,
    // timestamp of entering in transaction pool
    pub time: chrono::NaiveDateTime,
    pub elapsed: i64, // ms
    pub event: String,
    pub hash: String,
}

impl Point for Transaction {
    fn name(&self) -> &'static str {
        "transaction"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO transaction (network, time, elapsed, event, hash)\
            VALUES ($1, $2, $3, $4, $5)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.elapsed,
            &self.event,
            &self.hash,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS heartbeat (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     peer_id             VARCHAR ( 46 )  NOT NULL,
///     host                VARCHAR ( 46 )  NOT NULL,
///     connected_duration  BIGINT          NOT NULL,
///     client_version      VARCHAR ( 200 ) NOT NULL
/// );
///
/// SELECT create_hypertable('heartbeat', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Heartbeat {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub peer_id: String,
    pub host: String,
    pub connected_duration: i64, // ms
    pub client_version: String,
}

impl Point for Heartbeat {
    fn name(&self) -> &'static str {
        "heartbeat"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO heartbeat(network, time, peer_id, host, connected_duration, client_version)\
            VALUES ($1, $2, $3, $4, $5, $6)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.peer_id,
            &self.host,
            &self.connected_duration,
            &self.client_version,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS peers (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     number              INT             NOT NULL
/// );
///
/// SELECT create_hypertable('peers', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Peers {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub number: i32,
}

impl Point for Peers {
    fn name(&self) -> &'static str {
        "peers"
    }
    fn insert_query(&self) -> &'static str {
        "TODO"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&self.network, &self.time, &self.number]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS propagation (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     peer_id             VARCHAR ( 46 )  NOT NULL,
///     hash                VARCHAR ( 66 )  NOT NULL,
///     message_name        VARCHAR ( 20 )  NOT NULL,
///     elapsed             BIGINT          NULL,
///     nth                 INT             NULL
/// );
///
/// SELECT create_hypertable('propagation', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Propagation {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub peer_id: String,
    pub hash: String,
    pub message_name: String,
}

impl Point for Propagation {
    fn name(&self) -> &'static str {
        "propagation"
    }
    fn insert_query(&self) -> &'static str {
        "INSERT INTO propagation(network, time, peer_id, hash, message_name)\
            VALUES ($1, $2, $3, $4, $5)"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.peer_id,
            &self.hash,
            &self.message_name,
        ]
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS propagation_percentile (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     percentile          INT             NOT NULL,
///     elapsed             BIGINT          NOT NULL,
///     hash                VARCHAR ( 66 )  NOT NULL,
///     message_name        VARCHAR ( 20 )  NOT NULL
/// );
///
/// SELECT create_hypertable('propagation_percentile', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct PropagationPercentile {
    pub network: String,
    pub time: chrono::NaiveDateTime,
    pub elapsed: i64,    // ms
    pub percentile: i32, // 50 | 80 | 90 | 95
    pub hash: String,
    pub message_name: String, // "t" | "b"
}

impl Point for PropagationPercentile {
    fn name(&self) -> &'static str {
        "propagation_percentile"
    }
    fn insert_query(&self) -> &'static str {
        "TODO"
    }
    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        vec![
            &self.network,
            &self.time,
            &self.elapsed,
            &self.percentile,
            &self.hash,
            &self.message_name,
        ]
    }
}
