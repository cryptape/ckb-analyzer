CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE DATABASE ckbraw;

\c ckbraw;

CREATE TABLE IF NOT EXISTS peer (
    id                  SERIAL,
    network             VARCHAR ( 20 )  NOT NULL,
    time                TIMESTAMP       NOT NULL,
    version             VARCHAR ( 200 ) NOT NULL,
    ip                  VARCHAR ( 46 )  NOT NULL,
    country             VARCHAR ( 20 )  NULL
);

SELECT create_hypertable('peer', 'time', migrate_data => true);
