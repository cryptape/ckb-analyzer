-- CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE SCHEMA IF NOT EXISTS ckb;
CREATE SCHEMA IF NOT EXISTS ckb_testnet;

CREATE TABLE IF NOT EXISTS ckb.peer (
    id                  SERIAL,
    time                TIMESTAMP       NOT NULL,
    version             VARCHAR ( 200 ) NOT NULL,
    ip                  VARCHAR ( 46 )  NOT NULL,
    country             VARCHAR ( 20 )  NULL
);
CREATE TABLE IF NOT EXISTS ckb.block (
    time                TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    n_transactions      INT             NOT NULL,
    n_proposals         INT             NOT NULL,
    n_uncles            INT             NOT NULL,
    cellbase_message    VARCHAR ( 50 )  NULL,
    interval            BIGINT          NULL
);
CREATE TABLE IF NOT EXISTS ckb.tx_pool_info (
    time                TIMESTAMP       NOT NULL,
    total_tx_cycles     BIGINT          NOT NULL,
    total_tx_size       BIGINT          NOT NULL,
    pending             BIGINT          NOT NULL,
    proposed            BIGINT          NOT NULL,
    orphan              BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.block_transaction (
    time                TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    size                INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    total_data_size     INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);

CREATE TABLE IF NOT EXISTS ckb_testnet.peer (
    id                  SERIAL,
    time                TIMESTAMP       NOT NULL,
    version             VARCHAR ( 200 ) NOT NULL,
    ip                  VARCHAR ( 46 )  NOT NULL,
    country             VARCHAR ( 20 )  NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.block (
    time                TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    n_transactions      INT             NOT NULL,
    n_proposals         INT             NOT NULL,
    n_uncles            INT             NOT NULL,
    cellbase_message    VARCHAR ( 50 )  NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.tx_pool_info (
    time                TIMESTAMP       NOT NULL,
    total_tx_cycles     BIGINT          NOT NULL,
    total_tx_size       BIGINT          NOT NULL,
    pending             BIGINT          NOT NULL,
    proposed            BIGINT          NOT NULL,
    orphan              BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.block_transaction (
    time                TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    size                INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    total_data_size     INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);

SELECT create_hypertable('ckb.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb.block', 'time', migrate_data => true);
SELECT create_hypertable('ckb.tx_pool_info', 'time', migrate_data => true);
SELECT create_hypertable('ckb.block_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.block', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.tx_pool_info', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.block_transaction', 'time', migrate_data => true);
