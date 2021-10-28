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

SELECT create_hypertable('ckb.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb.block', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.peer', 'time', migrate_data => true);
SELECT create_hypertable('ckb_testnet.block', 'time', migrate_data => true);
