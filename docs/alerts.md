# [Alerts](http://13.212.112.4:3000/alerting/list)

Genrated by https://github.com/keroro520/ckb-analyzer/blob/main/scripts/alerts.py

> References:
>   * https://grafana.com/docs/grafana/latest/alerting/create-alerts
---

* Name: [[Aggron] Chain Growth](http://13.212.112.4:3000/d/vcN5hTfGz/aggron-alerts?viewPanel=30)

  Evaluate: Every 5m for 10m

  Description:   Target: Tip number

  Alert When: No update in a certain time

  Possible Cause: The canonical chain suspend


* Name: [[Aggron] Block Time Interval](http://13.212.112.4:3000/d/vcN5hTfGz/aggron-alerts?viewPanel=32)

  Evaluate: Every 1m for 5m

  Description:   Target: Block time interval

  Alert When: The average block time interval is greater/less than thresholds

  Possible Cause: Mining power changes rapidly


* Name: [[Aggron] Uncle Count](http://13.212.112.4:3000/d/vcN5hTfGz/aggron-alerts?viewPanel=34)

  Evaluate: Every 5m for 10m

  Description:   Target: Uncle count

  Alert When: The uncle count in the recent 20m is greater than the threshold

  Possible Cause: Mining power changes rapidly; network traffic jam


* Name: [[Aggron] Transaction Suspend in Pending Pool](http://13.212.112.4:3000/d/vcN5hTfGz/aggron-alerts?viewPanel=36)

  Evaluate: Every 1m for 5m

  Description:   Target: Pending Transaction

  Alert When: There are transactions that have been pending for a long time

  Possible Cause: Pool bug

* Name: [[Lina] Chain Growth](http://13.212.112.4:3000/d/1rK5hPfGe/lina-alerts?viewPanel=30)

  Evaluate: Every 5m for 10m

  Description:   Target: Tip number

  Alert When: No update in a certain time

  Possible Cause: The canonical chain suspend


* Name: [[Lina] Block Time Interval](http://13.212.112.4:3000/d/1rK5hPfGe/lina-alerts?viewPanel=32)

  Evaluate: Every 1m for 5m

  Description:   Target: Block time interval

  Alert When: The average block time interval is greater/less than thresholds

  Possible Cause: Mining power changes rapidly


* Name: [[Lina] Uncle Count](http://13.212.112.4:3000/d/1rK5hPfGe/lina-alerts?viewPanel=34)

  Evaluate: Every 5m for 10m

  Description:   Target: Uncle count

  Alert When: The uncle count in the recent 20m is greater than the threshold

  Possible Cause: Mining power changes rapidly; network traffic jam


* Name: [[Lina] Transaction Suspend in Pending Pool](http://13.212.112.4:3000/d/1rK5hPfGe/lina-alerts?viewPanel=36)

  Evaluate: Every 1m for 5m

  Description:   Target: Pending Transaction

  Alert When: There are transactions that have been pending for a long time

  Possible Cause: Pool bug

