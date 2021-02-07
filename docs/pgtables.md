## Notes

* Timescale [`create_hypertable`](https://docs.timescale.com/latest/api#create_hypertable) requires the table contains **no unique column**.

## Tables

```
CREATE TABLE IF NOT EXISTS block (
    time            TIMESTAMP       NOT NULL,
    number          BIGINT          NOT NULL,
    n_transactions  INT             NOT NULL,
    n_proposals     INT             NOT NULL,
    n_uncles        INT             NOT NULL,
    hash            CHAR ( 34 )     NOT NULL,
    miner           CHAR ( 34 )     NOT NULL
);
SELECT create_hypertable('block', 'time');
```
