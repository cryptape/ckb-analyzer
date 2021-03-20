# [Alerts](http://13.212.112.4:3000/alerting/list)


> References:
>   * https://grafana.com/docs/grafana/latest/alerting/create-alerts
---

* Name: [Tip Change (network=aggron) Alert](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=3)

  Evaluate: every 1m for 5m

  Description: Tip number change indicates the change growth.  When the tip number doesn't change for a long time, the chain may suspend.

* Name: [Block Time Interval Alert (network = aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=4)

  Evaluate: every 1m for 2m

  Description: Block time interval relates to the network mining power and the network propagation. A continuous long block time interval is a warning.

* Name: [Transaction Being Pending Too Long Alert (network = aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=5)

  Evaluate: every 1m for 0m

  Description: There is a transaction being pending for a long time.  Transactions should be packaged within new blocks or removed because of invalidation. Therefore if there are any pending-too-long transactions, tx-pool may have trouble.

* Name: [Transaction Being Proposed Too Long Alert (network = aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=6)

  Evaluate: every 1m for 0m

  Description: There is a transaction being proposed for a long time.  Transactions should be packaged within new blocks or removed because of invalidation. Therefore if there are any proposed-too-long transactions, tx-pool may have trouble.

* Name: [Error/Warning Logs Alert (network=aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=7)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Tip Change (network=lina) Alert](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=9)

  Evaluate: every 1m for 5m

  Description: Tip number change indicates the change growth.  When the tip number doesn't change for a long time, the chain may suspend.

* Name: [Block Time Interval Alert (network = lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=10)

  Evaluate: every 1m for 2m

  Description: 

* Name: [Transaction Being Pending Too Long (network = lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=11)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Transaction Being Proposed Too Long Alert (network = lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=12)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Error/Warning Logs Alert (network=lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=13)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Tip Change Alert (network=wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=15)

  Evaluate: every 1m for 5m

  Description: Tip number change indicates the change growth.  When the tip number doesn't change for a long time, the chain may suspend.

* Name: [Block Time Interval Alert (network = wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=16)

  Evaluate: every 1m for 2m

  Description: 

* Name: [Transaction Being Pending Too Long Alert (network = wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=17)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Transaction Being Proposed Too Long Alert (network = wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=18)

  Evaluate: every 1m for 0m

  Description: 

* Name: [Error/Warning Logs Alert (network=wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=19)

  Evaluate: every 1m for 0m

  Description: 

* Name: [No Data Alert (network = aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=21)

  Evaluate: every 30m for 60m

  Description: 

* Name: [No Data Alert (network = lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=22)

  Evaluate: every 30m for 60m

  Description: 

* Name: [No Data Alert (network = wano)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=23)

  Evaluate: every 30m for 60m

  Description: 

* Name: [Uncle Count in 20m Alert (network = aggron)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=27)

  Evaluate: every 1m for 10m

  Description: 

* Name: [Uncle Count in 20m Alert (network = lina)](http://13.212.112.4:3000/d/vcN5hTfGz/alerting?viewPanel=28)

  Evaluate: every 1m for 10m

  Description: 

