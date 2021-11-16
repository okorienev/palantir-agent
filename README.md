# palantir-agent

APM agent that accepts client events (UDP + Google Protocol Buffers), transofrms them into stored internally timeseries data and periodically synchronizes it with VictoriaMetrics (Prometheus-compatible TSDB)
