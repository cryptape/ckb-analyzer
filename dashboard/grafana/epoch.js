{
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": "-- Grafana --",
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "target": {
          "limit": 100,
          "matchAny": false,
          "tags": [],
          "type": "dashboard"
        },
        "type": "dashboard"
      }
    ]
  },
  "editable": true,
  "fiscalYearStartMonth": 0,
  "gnetId": null,
  "graphTooltip": 1,
  "id": 9,
  "iteration": 1635854834779,
  "links": [],
  "liveNow": false,
  "panels": [
    {
      "datasource": "PostgreSQL-ckb",
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "viz": false
            },
            "lineInterpolation": "stepBefore",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 9,
        "w": 12,
        "x": 0,
        "y": 0
      },
      "id": 2,
      "options": {
        "legend": {
          "calcs": [
            "lastNotNull",
            "mean"
          ],
          "displayMode": "table",
          "placement": "right"
        },
        "tooltip": {
          "mode": "single"
        }
      },
      "targets": [
        {
          "format": "time_series",
          "group": [],
          "metricColumn": "none",
          "rawQuery": true,
          "rawSql": "SELECT\n  start_time AS \"time\",\n  length AS Length\nFROM $network.epoch\nWHERE\n  $__timeFilter(start_time)\nORDER BY 1",
          "refId": "A",
          "select": [
            [
              {
                "params": [
                  "length"
                ],
                "type": "column"
              }
            ]
          ],
          "table": "$network.epoch",
          "timeColumn": "start_time",
          "timeColumnType": "timestamp",
          "where": [
            {
              "name": "$__timeFilter",
              "params": [],
              "type": "macro"
            }
          ]
        }
      ],
      "title": "Length",
      "type": "timeseries"
    },
    {
      "datasource": "PostgreSQL-ckb",
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "viz": false
            },
            "lineInterpolation": "stepBefore",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          },
          "unit": "s"
        },
        "overrides": []
      },
      "gridPos": {
        "h": 9,
        "w": 12,
        "x": 12,
        "y": 0
      },
      "id": 3,
      "options": {
        "legend": {
          "calcs": [
            "lastNotNull",
            "mean"
          ],
          "displayMode": "table",
          "placement": "right"
        },
        "tooltip": {
          "mode": "single"
        }
      },
      "targets": [
        {
          "format": "time_series",
          "group": [],
          "metricColumn": "none",
          "rawQuery": true,
          "rawSql": "WITH epoch_duration AS (\n    SELECT start_time,\n           (end_time - start_time) AS \"duration\"\n    FROM ckb.epoch\n    ORDER BY start_time\n)\n\nSELECT\n    e.start_time AS \"time\",\n    EXTRACT(EPOCH FROM e.duration) AS \"Duration\"\nFROM epoch_duration e;\n",
          "refId": "A",
          "select": [
            [
              {
                "params": [
                  "length"
                ],
                "type": "column"
              }
            ]
          ],
          "table": "$network.epoch",
          "timeColumn": "start_time",
          "timeColumnType": "timestamp",
          "where": [
            {
              "name": "$__timeFilter",
              "params": [],
              "type": "macro"
            }
          ]
        }
      ],
      "title": "Duration",
      "type": "timeseries"
    }
  ],
  "schemaVersion": 31,
  "style": "dark",
  "tags": [],
  "templating": {
    "list": [
      {
        "allValue": null,
        "current": {
          "selected": true,
          "text": "ckb_testnet",
          "value": "ckb_testnet"
        },
        "description": "CKB network name",
        "error": null,
        "hide": 0,
        "includeAll": false,
        "label": "network",
        "multi": false,
        "name": "network",
        "options": [
          {
            "selected": false,
            "text": "ckb",
            "value": "ckb"
          },
          {
            "selected": true,
            "text": "ckb_testnet",
            "value": "ckb_testnet"
          }
        ],
        "query": "ckb,ckb_testnet",
        "queryValue": "",
        "skipUrlSync": false,
        "type": "custom"
      }
    ]
  },
  "time": {
    "from": "now-24h",
    "to": "now"
  },
  "timepicker": {},
  "timezone": "",
  "title": "Epoch",
  "uid": "0fv3RaKnz",
  "version": 1
}
