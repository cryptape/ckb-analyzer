# CKBAnalyzer

The purpose of CKBAnalyzer is to facilitate observation of the CKB network.

The program does several things:
  - Crawl the CKB network to gather the nodes information
  - Store the main-chain blocks

CKBAnalyzer is still working in progress rapidly.

## Storage

CKBAnalyzer acts as a metrics agent and stores the data into [Timescaldb](https://docs.timescale.com/).

## Visualization

We visualize data using [Grafana](https://grafana.com/). Our Grafana dashboards are maintained in this repository.

## Install

Download from [releases](https://github.com/keroro520/ckb-analyzer/releases).

## Usage

### Environment Variables

* `IPINFO_IO_TOKEN`, optional, [ipinfo.io](https://ipinfo.ip) authentication token, is used by [`PeerScanner`](./src/topics/peer_scanner.rs) to look up the geographical location by nodes' ip.

* `CKB_ANALYZER_POSTGRES`, required, Postgres login key.

### Help

```shell
USAGE:
    ckb-analyzer [OPTIONS] --node.rpc <URL> --node.subscription <URL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --node.rpc <URL>
        --node.subscription <URL>
        --topics <TOPIC>...           [default: PeerCrawler,PeerScanner]  [possible values: PeerCrawler, PeerScanner]
```

### Example

```shell
CKB_ANALYZER_POSTGRES="postgres://postgres:postgres@localhost:5432/ckbraw" \
ckb-analyzer --node.rpc="http://127.0.0.1:8111" --node.subscription="http://127.0.0.1:18114"
```

License: MIT

