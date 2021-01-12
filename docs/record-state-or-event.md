# State or Event

We interest in two kinds of application metrics: state and event.

* An application state tells us what this application currently states, such as health, memory used, the total count of something, and so forth

* An application event tells us that there once happened something, such as the canonical chain reorganized, a transaction is removed as double-spending, and so forth.

  Events history is an important part of the application history. Some events history helps us recalling and reproducing some issue context.

  **To make it clear: as decentralized network operators, we do not just want to "monitor" the current application, but also "watch" the history.**

---

Prometheus uses a pulling model. It collects metrics periodically via HTTP from configured nodes; nodes expose metrics by providing an HTTP service. This model is efficient at collecting state metrics, but not good at historical events. Because the HTTP service only maintains the current(recent) information but not historical events.

Most state metrics can easily transform into event metrics.

Therefore I intend to:
  1. Nodes record real-time structured events into log-file
  2. An agent watches the log-file, extracts structured events, and reports into InfluxDB
