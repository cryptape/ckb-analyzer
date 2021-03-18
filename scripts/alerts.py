#!/usr/bin/env python

# Generate alerts document
#
# Usage Example:
#
# ```shell
# GRAFANA_ADDRESS="http://127.0.0.1:3000" GRAFANA_TOKEN=<API TOKEN> python3 scripts/alerts.py

from __future__ import print_function
import requests
import copy
import os

print('# [Alerts]({}/alerting/list)'.format(GRAFANA_ADDRESS), end='\n\n')
print('Genrated by https://github.com/keroro520/ckb-analyzer/blob/main/scripts/alerts.py')
print('''
> References:
>   * https://grafana.com/docs/grafana/latest/alerting/create-alerts
---
''')


# https://grafana.com/docs/grafana/latest/http_api/alerting
GRAFANA_ADDRESS = os.environ['GRAFANA_ADDRESS'].strip('/')
GRAFANA_TOKEN = os.environ['GRAFANA_TOKEN'].strip()
GRAFANA_HEADERS = {
    'Authorization': 'Bearer {}'.format(GRAFANA_TOKEN),
    'Content-Type': 'application/json'
}

grafana_alerts_url = '{}/api/alerts'.format(GRAFANA_ADDRESS)
alerts = requests.get(grafana_alerts_url, headers=GRAFANA_HEADERS).json()

# Get the alerting panels info. We want to generate alert description
# later by reusing the panel description. Furthermore, The alert panel
# links will be generated as well.
#
# https://grafana.com/docs/grafana/latest/http_api/dashboard/
dashboards = {} # #{dashboard_uid => dashboard_json}
dashboard_urls = {} # #{dashboard_id => dashboard_url}
for alert in alerts:
    dashboard_uid = alert['dashboardUid']
    if dashboard_uid in dashboards:
        continue
    grafana_get_dashboard_url = '{}/api/dashboards/uid/{}'.format(
            GRAFANA_ADDRESS, dashboard_uid
    )
    resp = requests.get(grafana_get_dashboard_url, headers=GRAFANA_HEADERS).json()
    dashboard, dashboard_url = resp['dashboard'], resp['meta']['url']
    dashboard_id = dashboard['id']

    dashboards[dashboard_uid] = copy.deepcopy(dashboard)
    dashboard_urls[dashboard_id] = dashboard_url

panels = {}
for dashboard in dashboards.values():
    dashboard_id = dashboard['id']
    for panel in dashboard['panels']:
        panel_id = panel['id']
        panels[(dashboard_id, panel_id)] = copy.deepcopy(panel)

## Output Format
# # [Alerts](../alerting/list)
#
# * Name:
#   Evaluate: every .. for ..
#   Description: 
#   Link:
detail_alerts = []
for alert in alerts:
    alert_id = alert['id']
    grafana_get_alert_url = '{}/api/alerts/{}'.format(GRAFANA_ADDRESS, alert_id)
    detail_alert = requests.get(grafana_get_alert_url, headers=GRAFANA_HEADERS).json()
    detail_alerts.append(detail_alert)
detail_alerts = sorted(detail_alerts, key=lambda alert: alert['PanelId'])


for alert in detail_alerts:
    settings = alert['Settings']
    alert_name = settings['name']
    dashboard_id = alert['DashboardId']
    panel_id = alert['PanelId']
    panel = panels[(dashboard_id, panel_id)]
    panel_link = '{}{}?viewPanel={}'.format(GRAFANA_ADDRESS, dashboard_urls[dashboard_id], panel_id)
    description = panel.get('description', '').replace('\n', ' ')

    print('* Name: [{}]({})'.format(alert_name, panel_link), end='\n\n')
    print('  Evaluate: Every {} for {}'.format(settings['frequency'], settings['for']), end='\n\n')
    print('  Description: {}'.format(description), end='\n\n')
