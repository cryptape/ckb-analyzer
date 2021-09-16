CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE DATABASE ckb;

\c ckb;

CREATE TABLE IF NOT EXISTS lina_block (
    network         VARCHAR ( 10 )  NOT NULL,
    time            TIMESTAMP       NOT NULL,
    number          BIGINT          NOT NULL,
    interval        BIGINT          NOT NULL,
    n_transactions  INT             NOT NULL,
    n_proposals     INT             NOT NULL,
    n_uncles        INT             NOT NULL,
    hash            CHAR ( 66 )     NOT NULL,
    miner           CHAR ( 66 )     NOT NULL,
    version         INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_uncle (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP   NOT NULL,
    number              BIGINT      NOT NULL,
    lag_to_canonical    BIGINT      NOT NULL,
    n_transactions      INT         NOT NULL,
    n_proposals         INT         NOT NULL,
    hash                CHAR ( 66 ) NOT NULL,
    miner               CHAR ( 66 ) NOT NULL,
    version             INT         NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_two_pc_commitment (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    delay       INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_epoch (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    length      INT             NOT NULL,
    duration    INT             NOT NULL,
    n_uncles    INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_reorganization (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    attached_length     INT             NOT NULL,
    old_tip_number      BIGINT          NOT NULL,
    new_tip_number      BIGINT          NOT NULL,
    ancestor_number     BIGINT          NOT NULL,
    old_tip_hash        CHAR ( 66 )     NOT NULL,
    new_tip_hash        CHAR ( 66 )     NOT NULL,
    ancestor_hash       CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    enter_time          TIMESTAMP       NOT NULL,
    commit_time         TIMESTAMP,
    remove_time         TIMESTAMP,
    hash                CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_heartbeat (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    host                VARCHAR ( 46 )  NOT NULL,
    connected_duration  BIGINT          NOT NULL,
    client_version      VARCHAR ( 200 ) NOT NULL,
    country             VARCHAR ( 5 )   NULL
);
CREATE TABLE IF NOT EXISTS lina_propagation (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL,
    message_name        VARCHAR ( 20 )  NOT NULL,
    elapsed             BIGINT          NULL,
    nth                 INT             NULL
);
CREATE TABLE IF NOT EXISTS lina_subscribe_new_tip_header(
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    block_timestamp     TIMESTAMP       NOT NULL,
    block_hash          VARCHAR ( 66 )  NOT NULL,
    block_number        BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_subscribe_new_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS lina_subscribe_proposed_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);


CREATE TABLE IF NOT EXISTS aggron_block (
    network         VARCHAR ( 10 )  NOT NULL,
    time            TIMESTAMP       NOT NULL,
    number          BIGINT          NOT NULL,
    interval        BIGINT          NOT NULL,
    n_transactions  INT             NOT NULL,
    n_proposals     INT             NOT NULL,
    n_uncles        INT             NOT NULL,
    hash            CHAR ( 66 )     NOT NULL,
    miner           CHAR ( 66 )     NOT NULL,
    version         INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_uncle (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP   NOT NULL,
    number              BIGINT      NOT NULL,
    lag_to_canonical    BIGINT      NOT NULL,
    n_transactions      INT         NOT NULL,
    n_proposals         INT         NOT NULL,
    hash                CHAR ( 66 ) NOT NULL,
    miner               CHAR ( 66 ) NOT NULL,
    version             INT         NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_two_pc_commitment (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    delay       INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_epoch (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    length      INT             NOT NULL,
    duration    INT             NOT NULL,
    n_uncles    INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_reorganization (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    attached_length     INT             NOT NULL,
    old_tip_number      BIGINT          NOT NULL,
    new_tip_number      BIGINT          NOT NULL,
    ancestor_number     BIGINT          NOT NULL,
    old_tip_hash        CHAR ( 66 )     NOT NULL,
    new_tip_hash        CHAR ( 66 )     NOT NULL,
    ancestor_hash       CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    enter_time          TIMESTAMP       NOT NULL,
    commit_time         TIMESTAMP,
    remove_time         TIMESTAMP,
    hash                CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_heartbeat (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    host                VARCHAR ( 46 )  NOT NULL,
    connected_duration  BIGINT          NOT NULL,
    client_version      VARCHAR ( 200 ) NOT NULL,
    country             VARCHAR ( 5 )   NULL
);
CREATE TABLE IF NOT EXISTS aggron_propagation (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL,
    message_name        VARCHAR ( 20 )  NOT NULL,
    elapsed             BIGINT          NULL,
    nth                 INT             NULL
);
CREATE TABLE IF NOT EXISTS aggron_subscribe_new_tip_header(
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    block_timestamp     TIMESTAMP       NOT NULL,
    block_hash          VARCHAR ( 66 )  NOT NULL,
    block_number        BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_subscribe_new_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS aggron_subscribe_proposed_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);


CREATE TABLE IF NOT EXISTS staging_block (
    network         VARCHAR ( 10 )  NOT NULL,
    time            TIMESTAMP       NOT NULL,
    number          BIGINT          NOT NULL,
    interval        BIGINT          NOT NULL,
    n_transactions  INT             NOT NULL,
    n_proposals     INT             NOT NULL,
    n_uncles        INT             NOT NULL,
    hash            CHAR ( 66 )     NOT NULL,
    miner           CHAR ( 66 )     NOT NULL,
    version         INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_uncle (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP   NOT NULL,
    number              BIGINT      NOT NULL,
    lag_to_canonical    BIGINT      NOT NULL,
    n_transactions      INT         NOT NULL,
    n_proposals         INT         NOT NULL,
    hash                CHAR ( 66 ) NOT NULL,
    miner               CHAR ( 66 ) NOT NULL,
    version             INT         NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_two_pc_commitment (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    delay       INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_epoch (
    network     VARCHAR ( 10 )  NOT NULL,
    time        TIMESTAMP       NOT NULL,
    number      BIGINT          NOT NULL,
    length      INT             NOT NULL,
    duration    INT             NOT NULL,
    n_uncles    INT             NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_reorganization (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    attached_length     INT             NOT NULL,
    old_tip_number      BIGINT          NOT NULL,
    new_tip_number      BIGINT          NOT NULL,
    ancestor_number     BIGINT          NOT NULL,
    old_tip_hash        CHAR ( 66 )     NOT NULL,
    new_tip_hash        CHAR ( 66 )     NOT NULL,
    ancestor_hash       CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    enter_time          TIMESTAMP       NOT NULL,
    commit_time         TIMESTAMP,
    remove_time         TIMESTAMP,
    hash                CHAR ( 66 )     NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_heartbeat (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    host                VARCHAR ( 46 )  NOT NULL,
    connected_duration  BIGINT          NOT NULL,
    client_version      VARCHAR ( 200 ) NOT NULL,
    country             VARCHAR ( 5 )   NULL
);
CREATE TABLE IF NOT EXISTS staging_propagation (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    peer_id             VARCHAR ( 46 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL,
    message_name        VARCHAR ( 20 )  NOT NULL,
    elapsed             BIGINT          NULL,
    nth                 INT             NULL
);
CREATE TABLE IF NOT EXISTS staging_subscribe_new_tip_header(
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    block_timestamp     TIMESTAMP       NOT NULL,
    block_hash          VARCHAR ( 66 )  NOT NULL,
    block_number        BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_subscribe_new_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS staging_subscribe_proposed_transaction (
    network             VARCHAR ( 10 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    hostname            VARCHAR ( 46 )  NOT NULL,
    transaction_hash    VARCHAR ( 66 )  NOT NULL
);


SELECT create_hypertable('lina_block', 'time');
SELECT create_hypertable('lina_uncle', 'time');
SELECT create_hypertable('lina_two_pc_commitment', 'time');
SELECT create_hypertable('lina_epoch', 'time');
SELECT create_hypertable('lina_reorganization', 'time');
SELECT create_hypertable('lina_transaction', 'enter_time');
SELECT create_hypertable('lina_heartbeat', 'time');
SELECT create_hypertable('lina_propagation', 'time');
SELECT create_hypertable('lina_subscribe_new_tip_header', 'time');
SELECT create_hypertable('lina_subscribe_new_transaction', 'time');
SELECT create_hypertable('lina_subscribe_proposed_transaction', 'time');

SELECT create_hypertable('aggron_block', 'time');
SELECT create_hypertable('aggron_uncle', 'time');
SELECT create_hypertable('aggron_two_pc_commitment', 'time');
SELECT create_hypertable('aggron_epoch', 'time');
SELECT create_hypertable('aggron_reorganization', 'time');
SELECT create_hypertable('aggron_transaction', 'enter_time');
SELECT create_hypertable('aggron_heartbeat', 'time');
SELECT create_hypertable('aggron_propagation', 'time');
SELECT create_hypertable('aggron_subscribe_new_tip_header', 'time');
SELECT create_hypertable('aggron_subscribe_new_transaction', 'time');
SELECT create_hypertable('aggron_subscribe_proposed_transaction', 'time');

SELECT create_hypertable('staging_block', 'time');
SELECT create_hypertable('staging_uncle', 'time');
SELECT create_hypertable('staging_two_pc_commitment', 'time');
SELECT create_hypertable('staging_epoch', 'time');
SELECT create_hypertable('staging_reorganization', 'time');
SELECT create_hypertable('staging_transaction', 'enter_time');
SELECT create_hypertable('staging_heartbeat', 'time');
SELECT create_hypertable('staging_propagation', 'time');
SELECT create_hypertable('staging_subscribe_new_tip_header', 'time');
SELECT create_hypertable('staging_subscribe_new_transaction', 'time');
SELECT create_hypertable('staging_subscribe_proposed_transaction', 'time');




CREATE USER ckb_analyzer WITH
    NOCREATEDB
    NOCREATEROLE
    PASSWORD 'azBbP3tNH3FgQ9y2ifVM9eBZa';
CREATE USER grafana WITH
    NOCREATEDB
    NOCREATEROLE
    PASSWORD 'la02vlffhkap9ufeoihaFkjhfa';


GRANT SELECT, INSERT, UPDATE ON TABLE lina_block TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_uncle TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_two_pc_commitment TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_epoch TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_reorganization TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_heartbeat TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_peers TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_propagation TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_propagation_percentile TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_subscribe_new_tip_header TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_subscribe_new_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE lina_subscribe_proposed_transaction TO ckb_analyzer;

GRANT SELECT ON TABLE lina_block TO grafana;
GRANT SELECT ON TABLE lina_uncle TO grafana;
GRANT SELECT ON TABLE lina_two_pc_commitment TO grafana;
GRANT SELECT ON TABLE lina_epoch TO grafana;
GRANT SELECT ON TABLE lina_reorganization TO grafana;
GRANT SELECT ON TABLE lina_transaction TO grafana;
GRANT SELECT ON TABLE lina_heartbeat TO grafana;
GRANT SELECT ON TABLE lina_peers TO grafana;
GRANT SELECT ON TABLE lina_propagation TO grafana;
GRANT SELECT ON TABLE lina_propagation_percentile TO grafana;
GRANT SELECT ON TABLE lina_subscribe_new_tip_header TO grafana;
GRANT SELECT ON TABLE lina_subscribe_new_transaction TO grafana;
GRANT SELECT ON TABLE lina_subscribe_proposed_transaction TO grafana;

GRANT SELECT, INSERT, UPDATE ON TABLE aggron_block TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_uncle TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_two_pc_commitment TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_epoch TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_reorganization TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_heartbeat TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_peers TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_propagation TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_propagation_percentile TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_subscribe_new_tip_header TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_subscribe_new_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE aggron_subscribe_proposed_transaction TO ckb_analyzer;

GRANT SELECT ON TABLE aggron_block TO grafana;
GRANT SELECT ON TABLE aggron_uncle TO grafana;
GRANT SELECT ON TABLE aggron_two_pc_commitment TO grafana;
GRANT SELECT ON TABLE aggron_epoch TO grafana;
GRANT SELECT ON TABLE aggron_reorganization TO grafana;
GRANT SELECT ON TABLE aggron_transaction TO grafana;
GRANT SELECT ON TABLE aggron_heartbeat TO grafana;
GRANT SELECT ON TABLE aggron_peers TO grafana;
GRANT SELECT ON TABLE aggron_propagation TO grafana;
GRANT SELECT ON TABLE aggron_propagation_percentile TO grafana;
GRANT SELECT ON TABLE aggron_subscribe_new_tip_header TO grafana;
GRANT SELECT ON TABLE aggron_subscribe_new_transaction TO grafana;
GRANT SELECT ON TABLE aggron_subscribe_proposed_transaction TO grafana;


GRANT SELECT, INSERT, UPDATE ON TABLE staging_block TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_uncle TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_two_pc_commitment TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_epoch TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_reorganization TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_heartbeat TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_peers TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_propagation TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_propagation_percentile TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_subscribe_new_tip_header TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_subscribe_new_transaction TO ckb_analyzer;
GRANT SELECT, INSERT, UPDATE ON TABLE staging_subscribe_proposed_transaction TO ckb_analyzer;

GRANT SELECT ON TABLE staging_block TO grafana;
GRANT SELECT ON TABLE staging_uncle TO grafana;
GRANT SELECT ON TABLE staging_two_pc_commitment TO grafana;
GRANT SELECT ON TABLE staging_epoch TO grafana;
GRANT SELECT ON TABLE staging_reorganization TO grafana;
GRANT SELECT ON TABLE staging_transaction TO grafana;
GRANT SELECT ON TABLE staging_heartbeat TO grafana;
GRANT SELECT ON TABLE staging_peers TO grafana;
GRANT SELECT ON TABLE staging_propagation TO grafana;
GRANT SELECT ON TABLE staging_propagation_percentile TO grafana;
GRANT SELECT ON TABLE staging_subscribe_new_tip_header TO grafana;
GRANT SELECT ON TABLE staging_subscribe_new_transaction TO grafana;
GRANT SELECT ON TABLE staging_subscribe_proposed_transaction TO grafana;
