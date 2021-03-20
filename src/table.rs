// TODO How to store u64 in Postgres?
// TODO Can we use "TIME" to represent duration?
// TODO Define ToSQL types, instead of String

use std::fmt::Debug;

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

impl Block {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO block (network, time, number, interval, n_transactions, n_proposals, n_uncles, hash, miner, version)\
            VALUES ('{}', '{}', {}, {}, {}, {}, {}, '{}', '{}', {})",
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
        )
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

impl Uncle {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO uncle (network, time, number, lag_to_canonical, n_transactions, n_proposals, hash, miner, version)\
            VALUES ('{}', '{}', {}, {}, {}, {}, '{}', '{}', {})",
            &self.network,
            &self.time,
            &self.number,
            &self.lag_to_canonical,
            &self.n_transactions,
            &self.n_proposals,
            &self.hash,
            &self.miner,
            &self.version,
        )
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

impl TwoPCCommitment {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO two_pc_commitment (network, time, number, delay) \
            VALUES ('{}', '{}', {}, {})",
            self.network, &self.time, &self.number, &self.delay
        )
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

impl Epoch {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO epoch (network, time, number, length, duration, n_uncles) \
            VALUES ('{}', '{}', {}, {}, {}, {})",
            &self.network, &self.time, &self.number, &self.length, &self.duration, &self.n_uncles,
        )
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

impl Reorganization {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO reorganization (network, time, attached_length, old_tip_number, \
            new_tip_number, ancestor_number, old_tip_hash, new_tip_hash, ancestor_hash)\
            VALUES ('{}', '{}', {}, {}, {}, {}, '{}', '{}', '{}')",
            &self.network,
            &self.time,
            &self.attached_length,
            &self.old_tip_number,
            &self.new_tip_number,
            &self.ancestor_number,
            &self.old_tip_hash,
            &self.new_tip_hash,
            &self.ancestor_hash,
        )
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS transaction (
///     network             VARCHAR ( 10 )  NOT NULL,
///     enter_time          TIMESTAMP,
///     commit_time         TIMESTAMP,
///     remove_time         TIMESTAMP,
///     hash                CHAR ( 66 )     NOT NULL
/// );
///
/// SELECT create_hypertable('transaction', 'time');
/// ```
#[derive(Clone, Debug)]
pub struct Transaction {
    pub network: String,
    pub enter_time: Option<chrono::NaiveDateTime>,
    pub commit_time: Option<chrono::NaiveDateTime>,
    pub remove_time: Option<chrono::NaiveDateTime>,
    pub hash: String,
}

impl Transaction {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO transaction (network, enter_time, hash)\
            VALUES ('{}', '{}', '{}')",
            self.network,
            self.enter_time.unwrap(),
            self.hash,
        )
    }
    pub fn update_query(&self) -> String {
        if let Some(ref commit_time) = self.commit_time {
            format!("UPDATE transaction SET commit_time='{}' WHERE hash='{}'", commit_time, self.hash)
        } else if let Some(ref remove_time) = self.remove_time {
            format!("UPDATE transaction SET remove_time='{}' WHERE hash='{}'", remove_time, self.hash)
        } else {
            unreachable!()
        }
    }
}

/// ```
/// CREATE TABLE IF NOT EXISTS heartbeat (
///     network             VARCHAR ( 10 )  NOT NULL,
///     time                TIMESTAMP       NOT NULL,
///     peer_id             VARCHAR ( 46 )  NOT NULL,
///     host                VARCHAR ( 46 )  NOT NULL,
///     connected_duration  BIGINT          NOT NULL,
///     client_version      VARCHAR ( 200 ) NOT NULL,
///     country             VARCHAR ( 5 )   NULL
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
    pub country: String,
}

impl Heartbeat {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO heartbeat(network, time, peer_id, host, connected_duration, client_version, country)\
            VALUES ('{}', '{}', '{}', '{}', {}, '{}', '{}')",
            &self.network,
            &self.time,
            &self.peer_id,
            &self.host,
            &self.connected_duration,
            &self.client_version,
            &self.country,
        )
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

impl Propagation {
    pub fn insert_query(&self) -> String {
        format!(
            "INSERT INTO propagation(network, time, peer_id, hash, message_name)\
            VALUES ('{}', '{}', '{}', '{}', '{}')",
            &self.network, &self.time, &self.peer_id, &self.hash, &self.message_name,
        )
    }
}

// /// # Create trigger
// /// ```sql
// /// CREATE OR REPLACE FUNCTION label_nth_propagation() RETURNS trigger AS $$
// /// DECLARE
// ///     first_time TIMESTAMP;
// ///     inserted_count INT;
// /// BEGIN
// ///     SELECT first(time, time) INTO first_time FROM propagation WHERE hash = NEW.hash;
// ///
// ///     IF first_time IS NULL THEN
// ///         NEW.elapsed = 0;
// ///         NEW.nth = 1;
// ///     ELSE
// ///         SELECT COUNT(*) INTO inserted_count FROM propagation WHERE hash = NEW.hash;
// ///         NEW.elapsed = EXTRACT(MILLISECONDS FROM (NEW.time - first_time));
// ///         NEW.nth := inserted_count + 1;
// ///     END IF;
// ///
// ///     RETURN NEW;
// /// END;
// /// $$ LANGUAGE plpgsql;
// ///
// /// CREATE TRIGGER label_nth_propagation_trigger BEFORE INSERT ON propagation
// /// FOR EACH ROW
// /// EXECUTE PROCEDURE label_nth_propagation();
// /// ```

// /// ```
// /// CREATE TABLE IF NOT EXISTS propagation_percentile (
// ///     network             VARCHAR ( 10 )  NOT NULL,
// ///     time                TIMESTAMP       NOT NULL,
// ///     percentile          INT             NOT NULL,
// ///     elapsed             BIGINT          NOT NULL,
// ///     hash                VARCHAR ( 66 )  NOT NULL,
// ///     message_name        VARCHAR ( 20 )  NOT NULL
// /// );
// ///
// /// SELECT create_hypertable('propagation_percentile', 'time');
// /// ```
// #[derive(Clone, Debug)]
// pub struct PropagationPercentile {
//     pub network: String,
//     pub time: chrono::NaiveDateTime,
//     pub elapsed: i64,    // ms
//     pub percentile: i32, // 50 | 80 | 90 | 95
//     pub hash: String,
//     pub message_name: String, // "t" | "b"
// }
