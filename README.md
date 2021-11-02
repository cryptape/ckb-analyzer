# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

CKBAnalyzer is still working in progress rapidly.

## Storage

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaldb](https://docs.timescale.com/).

## Visualization

We visualize data using [Grafana](https://grafana.com/). Our Grafana dashboards are maintained in [dashboard/](https://github.com/keroro520/ckb-analyzer/tree/main/dashboard/grafana) directory.

## Install

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).

## Usage

### Database Schema

[`schema.sql`](https://github.com/keroro520/ckb-analyzer/blob/main/src/schema.sql)

### Environment Variables

| variable | required | description |
| :--- | :--- | :--- |
| `IPINFO_IO_TOKEN` | false | [ipinfo.io](https://ipinfo.ip) authentication token, is used by [`PeerScanner`](./src/topics/peer_scanner.rs) to look up the geographical location by nodes' ip. |
| `CKB_ANALYZER_POSTGRES` | true | Postgres login key |

### Help

```shell
USAGE:
    ckb-analyzer [OPTIONS] --node.rpc <URL> --node.subscription <SOCKET_ADDR>
```

### Example

```shell
CKB_ANALYZER_POSTGRES="postgres://postgres:postgres@localhost:5432/ckb" \
ckb-analyzer --node.rpc="http://127.0.0.1:8111" --node.subscription="127.0.0.1:18114"
```

License: MIT

