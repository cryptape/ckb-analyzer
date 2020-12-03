# ckb-analyzer

ckb-analyzer is an agent for collecting metrics from ckb, then writing the processed metrics info InfluxDB.  we can visalize these metrics on Grafana or other visalization tools. Currently we collect:

- main chain metrics,, mainly include epochs, blocks, and uncles
- real-time the network metrics, include block propagation, transaction propagation, and high latency records
- the network topology

ckb-analyzer is still work in progress.

### FAQ

* Ckb itself also exposes metrics. So why create ckb-analyzer?

  Some metrics are not convenient to expose from ckb, like historical chain metrics and complexe analyzing tasks. With ckb-analyzer, we can display historical chain infomation by extracting the historicla blocks, and do some complexity tasks outside ckb, which prevent adding too much complexity into ckb.

* Why use InfluxDB?

  Pushing metrics actively via HTTP to InfluxDB is much useful!

### Snapshot Example

Please reference our Grafana dashboard files at [`dashboards`](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards)

* Network Prober: https://snapshot.raintank.io/dashboard/snapshot/5Yql0Y9q1EW20hHZjBzgXXbU1DSYuqaR
* MainChain: https://snapshot.raintank.io/dashboard/snapshot/THchboPCVZBQ827X0jRtdlFzi20I4Gvk

### How to Use

ckb-analyzer reads several environment variables:

* `CKB_ANALYZER_CONFIG` specify the configuration file path
* `CKB_RPC_USERNAME` specify the authorization username to ckb rpc service, default is `""`
* `CKB_RPC_PASSWORD` specify the authorization password to ckb rpc service, default is `""`
* `INFLUXDB_USERNAME` specify the influxdb username, default is `""`
* `INFLUXDB_PASSWORD` specify the influxdb password, default is `""`

Command example:

```shell
CKB_ANALYZER_CONFIG=config/local.toml cargo run --release
```
