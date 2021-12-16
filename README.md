# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaledb](https://docs.timescale.com/), then visualize using [Grafana](https://grafana.com/). Free to use our maintained [dashboards](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards).

## Getting Started

### Setup TimescaleDB and Grafana services via docker-compose

```shell
$ cp docker/.env.example docker/.env

$ docker-compose -f docker/docker-compose.yaml up -d

$ source docker/.env && psql -f src/schema.sql
```

### Install CKBAnalyzer

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).


### Run CKBAnalyzer

Environment variables:

| variable | required | description |
| :--- | :--- | :--- |
| `IPINFO_IO_TOKEN` | false | [ipinfo.io](https://ipinfo.ip) authentication token, is used by [`PeerScanner`](./src/topics/peer_scanner.rs) to look up the geographical location by nodes' ip. |
| `CKB_ANALYZER_POSTGRES` | true | Postgres login key |

```shell
CKB_ANALYZER_POSTGRES="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}" \
ckb-analyzer --node.rpc="http://127.0.0.1:8111" --node.subscription="127.0.0.1:18114"
```

---

License: MIT
