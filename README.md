# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaledb](https://docs.timescale.com/), then visualize using [Grafana](https://grafana.com/). Free to use our maintained [dashboards](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards).

## Getting Started

### Setup TimescaleDB and Grafana services via docker-compose

```shell
$ cp docker/.env.example docker/.env

$ docker-compose -f docker/docker-compose.yaml up -d
```

### Install CKBAnalyzer

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).

### Run CKBAnalyzer

The following environment variables are required by CKBAnalyzer. They are mostly declared inside [`docker/.env.example`](./docker/.env.example). You can specify an environment file with `--envfile`.

| variable | required | description |
| :--- | :--- | :--- |
| `IPINFO_IO_TOKEN` | false | [ipinfo.io](https://ipinfo.ip) authentication token, is used to look up the geographical location by nodes' ip. |
| `PGHOST` | true | Postgres host |
| `PGPORT` | true | Postgres port |
| `PGDATABASE` | true | Postgres database |
| `PGUSER` | true | Postgres username |
| `PGPASSWORD` | true | Postgres password |

```shell
ckb-analyzer \
    --node.rpc="http://127.0.0.1:8111" \
    --node.subscription="127.0.0.1:18114" \
    --envfile docker/.env
```

---

License: MIT
