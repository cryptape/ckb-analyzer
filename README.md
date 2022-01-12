# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaledb](https://docs.timescale.com/), then visualize using [Grafana](https://grafana.com/).

Visit the online dashboards at [https://ckbmonitor.bit.host/], and you can use the [maintained dashboards](https://github.com/keroro520/ckb-analyzer/tree/main/dashboards).

## Getting Started

### Setup TimescaleDB and Grafana services via docker-compose

```shell
$ cp docker/.env.example docker/.env

$ docker-compose -f docker/docker-compose.yaml up -d

$ source docker/.env && psql "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@127.0.0.1:${POSTGRES_PORT:-"5432"}" -f src/schema.sql
```

### Install CKBAnalyzer

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).

### Run CKBAnalyzer

Mostly environment variables are declared inside [`docker/.env.example`](./docker/.env.example). You can specify an environment file with `--envfile`.

```shell
ckb-analyzer \
    --node.rpc="http://127.0.0.1:8111" \
    --node.subscription="127.0.0.1:18114" \
    --envfile docker/.env
```

---

License: MIT
