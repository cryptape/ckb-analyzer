# ckb-analyzer

ckb-analyzer is an agent for collecting metrics from ckb, then writing the processed metrics info InfluxDB.  we can visalize these metrics on Grafana or other visalization tools. Currently we collect:

- main chain metrics,, mainly include epochs, blocks, and uncles
- real-time the network metrics, include block propagation, transaction propagation, and high latency records
- the network topology

### FAQ

* Ckb itself also exposes metrics. So why create ckb-analyzer?

  Some metrics are not convenient to expose from ckb, like historical chain metrics and complexe analyzing tasks. With ckb-analyzer, we can display historical chain infomation by extracting the historicla blocks, and do some complexity tasks outside ckb, which prevent adding too much complexity into ckb.

* Why use InfluxDB?

  Pushing metrics actively via HTTP to InfluxDB is much useful!

### Snapshot Example

Contract [us](keroroxx520@gmail.com) if you want the dashboard json file.

* network prober: https://snapshot.raintank.io/dashboard/snapshot/5clx4H72cGkEt1Tmiz8jkkJ8M3C7IE5O

* chain: https://snapshot.raintank.io/dashboard/snapshot/aR840AN8IJ9xehuP5vxgbVvifi3lMkaC
