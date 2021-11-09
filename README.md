# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

CKBAnalyzer is still working in progress rapidly.

## Storage

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaledb](https://docs.timescale.com/).

## Visualization

We visualize data using [Grafana](https://grafana.com/). Our Grafana dashboards are maintained in [dashboard/](https://github.com/keroro520/ckb-analyzer/tree/main/dashboard/grafana) directory.

## Install

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).

## Getting Started

### Setup TimescaleDB and Grafana services via docker-compose

```shell
$ cd docker

$ cp .env.example .env

$ docker-compose up -d

$ source .env

$ psql "postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:5432/${POSTGRES_DB}" -f ../src/schema.sql
```

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
