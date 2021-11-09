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
CREATE TABLE IF NOT EXISTS ckb.subscribed_new_transaction (
    time                TIMESTAMP       NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.subscribed_proposed_transaction (
    time                TIMESTAMP       NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.subscribed_rejected_transaction (
    time                TIMESTAMP       NOT NULL,
    reason              VARCHAR ( 60 )  NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.epoch (
    start_time          TIMESTAMP       NOT NULL,
    end_time            TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    length              BIGINT          NOT NULL,
    start_number        BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.retention_transaction (
    time                TIMESTAMP       NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb.cell (
    creating_time                   TIMESTAMP       NOT NULL,
    consuming_time                  TIMESTAMP,
    creating_number                 BIGINT          NOT NULL,
    tx_hash                         VARCHAR ( 66 )  NOT NULL,
    index                           INT             NOT NULL,
    lock_code_hash                  VARCHAR ( 66 )  NOT NULL,
    lock_args                       VARCHAR ( 100 ),
    type_code_hash                  VARCHAR ( 66 )
);
CREATE INDEX ckb_cell_out_point ON ckb.cell(tx_hash, index);

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
CREATE TABLE IF NOT EXISTS ckb_testnet.subscribed_new_transaction (
    time                TIMESTAMP       NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.subscribed_proposed_transaction (
    time                TIMESTAMP       NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.subscribed_rejected_transaction (
    time                TIMESTAMP       NOT NULL,
    reason              VARCHAR ( 60 )  NOT NULL,
    size                INT             NOT NULL,
    cycles              INT             NOT NULL,
    fee                 INT             NOT NULL,
    n_inputs            INT             NOT NULL,
    n_outputs           INT             NOT NULL,
    n_header_deps       INT             NOT NULL,
    n_cell_deps         INT             NOT NULL,
    proposal_id         VARCHAR ( 66 )  NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.epoch (
    start_time          TIMESTAMP       NOT NULL,
    end_time            TIMESTAMP       NOT NULL,
    number              BIGINT          NOT NULL,
    length              BIGINT          NOT NULL,
    start_number        BIGINT          NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.retention_transaction (
    time                TIMESTAMP       NOT NULL,
    hash                VARCHAR ( 66 )  NOT NULL
);
CREATE TABLE IF NOT EXISTS ckb_testnet.cell (
    creating_time                   TIMESTAMP       NOT NULL,
    consuming_time                  TIMESTAMP,
    creating_number                 BIGINT          NOT NULL,
    tx_hash                         VARCHAR ( 66 )  NOT NULL,
    index                           INT             NOT NULL,
    lock_code_hash                  VARCHAR ( 66 )  NOT NULL,
    lock_args                       VARCHAR ( 100 ),
    type_code_hash                  VARCHAR ( 66 )
);
CREATE INDEX ckb_testnet_cell_out_point ON ckb_testnet.cell(tx_hash, index);

SELECT create_hypertable('ckb.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb.block', 'time', migrate_data => true);
SELECT create_hypertable('ckb.epoch', 'start_time', migrate_data => true);
SELECT create_hypertable('ckb.tx_pool_info', 'time', migrate_data => true);
SELECT create_hypertable('ckb.block_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb.subscribed_new_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb.subscribed_proposed_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb.subscribed_rejected_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb.retention_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb.cell', 'creating_time', migrate_data => true);

SELECT create_hypertable('ckb_testnet.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.block', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.epoch', 'start_time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.tx_pool_info', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.block_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.subscribed_new_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.subscribed_proposed_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.subscribed_rejected_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.retention_transaction', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.cell', 'creating_time', migrate_data => true);
