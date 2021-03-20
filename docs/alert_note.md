## Implementation Notes

Grafana alert does not support template variables, which means that it's impossible to create alert rules at node-level based on template variables. Although Prometheus alert manager supports template variables, our data is stored in InfluxDB.

Another inconvenience is that some info of series we interest, such as the hostname and network name, will not be shown in notification content.

I don't want to change another alert system yet. Therfore I create an alerting dashboard that consists of many panels, one panel to one node's metric, to one alert. So that a node-level alert carries the corresponding image and serie info.
