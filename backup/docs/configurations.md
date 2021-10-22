# Configurations

Ckb-analyzer reads the config file path from the environment variable `CKB_ANALYZER_CONFIG`. The config file should contain sections below:

* `topics`
  A list of topics to be execute. Available topics are:
    * `"CanonicalChainState"`
    * `"Reorganization"`
    * `"TxTransition"`
    * `"PatternLogs"`
    * `"NetworkPropagation"`

* `influxdb`
    * `url`, the url of InfluxDB, e.g. `"http://127.0.0.1:8086"`
    * `database`, the database name of InfluxDB, e.g. `"ckb"`

* `node`
    * `host`, the target ckb node's host, e.g. `"127.0.0.1"`
    * `rpc_port`, the target ckb node's RPC port, e.g. `8121`
    * `subscription_port`, the target ckb node's port to subscription API, e.g. `18114`
    * `data_dir`, the target ckb node's data directory. e.g. `"/home/ckb/nodes/node-8111/default/"`
    * `bootnodes`, the list of bootnodes, e.g. `["/ip4/47.111.169.36/tcp/8111/p2p/QmNQ4jky6uVqLDrPU7snqxARuNGWNLgSrTnssbRuy3ij2W"]`
