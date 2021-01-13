# ckb-analyzer

ckb-analyzer is an agent for collecting metrics from ckb, then writing the processed metrics
info InfluxDB. We can visualize these metrics on Grafana or other visualization tools.

ckb-analyzer is still working in progress rapidly.

## Install

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases) or

```shell
cargo install ckb-analyzer
```

## Usage

ckb-analyzer reads several environment variables:

* `CKB_ANALYZER_CONFIG` specify the configuration file path
* `CKB_RPC_USERNAME` specify the authorization username to ckb rpc service, default is `""`
* `CKB_RPC_PASSWORD` specify the authorization password to ckb rpc service, default is `""`
* `INFLUXDB_USERNAME` specify the influxdb username, default is `""`
* `INFLUXDB_PASSWORD` specify the influxdb password, default is `""`

Command example:

```shell
CKB_ANALYZER_CONFIG=config/test.toml ckb-analyzer
```

## Topics and measurements

* [ ] canonical chain
  - [x] block committed transactions and proposed transactions
  - [x] transactions per seconds
  - [x] block time interval
  - [x] epoch uncles count and uncles rate
  - [x] epoch duration and length
  - [ ] epoch adjustment

* [x] canonical chain reorganization
  * [x] traffic
  * [x] scale

* [ ] node's canonical chain growth

* [ ] node's uncle blocks (some may not be included in canonical chain uncles)
  - [ ] traffic

* [ ] miner
  - [ ] the miner node of a specified block (ip or lock args)

* [ ] transaction transition (mainly focus the transaction traffic and state transition latency)
  - [x] pending
  - [x] pending too long
  - [x] propose
  - [x] propose too long
  - [x] commit
  - [x] remove (with reason, reject, conflict, and so forth)
  - [ ] reorganize

* [ ] node's tx-pool state
  - [ ] pending/proposed pool size/cycles

* [x] processed cost (via ckb internal metrics service)
  - [x] verify block
  - [x] verify transaction

* [x] transaction and block propagation across the network
  - [x] the number of connected peers
  - [x] propagation elapsed
  - [x] high latency propagation

* [x] logged events
  - [x] error and warning events
  - [x] sufficient events, recognize via regex patterning; better structure these logs

## Debug suites

* [ ] persist recent transactions (debug suite)

* [ ] reproduce context

## Monitoring alerts

* [ ] datasource issues
  - [ ] no update for a long time

* [ ] miner issues
  - [ ] chain does not grow up for too long
  - [ ] node receives too many uncle blocks

* [ ] chain growth issues
  - [ ] block time interval is shorter/longer then threshold
  - [ ] a big epoch adjustment

* [ ] transaction transition issues
  - [ ] too many transactions at a certain state

* [ ] logged issues

## Dashboards

Please reference our Grafana dashboard files at [`dashboards`](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards)

## FAQ

* ckb itself exposes metrics. Then why create ckb-analyzer?

  Some metrics are not convenient to expose from ckb, like historical chain metrics and complex
  analyzing tasks. With ckb-analyzer, we can display historical chain information by extracting
  the historical blocks and do some complexity tasks outside ckb, which prevent adding too much
  complexity into ckb.

* Why use InfluxDB?

  Pushing metrics actively via HTTP to InfluxDB is much useful!

License: MIT
