# Monitoring Alerts

I list the planned alerts and note how to archive them.

## Alert List

* Datasource no data or values are null
* Network
  - [ ] Forks
  - [x] Canonical tip number does not change
  - [x] Avarage block interval is large
* Node
  - Mines too many uncle blocks
  - [x] Transaction being pending or proposed too long
  - [x] Any error/warning logs occurs

## Implementation Notes

Grafana alert does not support template variables, which means that it's impossible to create alert rules at node-level based on template variables. Although Prometheus alert manager supports template variables, our data is stored in InfluxDB.

Another inconvenience is that some info of series we interest, such as the hostname and network name, will not be shown in notification content.

I don't want to change another alert system yet. Therfore I create an alerting dashboard that consists of many panels, one panel to one node's metric, to one alert. So that a node-level alert carries the corresponding image and serie info.
