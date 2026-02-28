# docker-compose 可観測性サービス設計

docker-compose における Prometheus・Grafana・Loki・Jaeger の詳細設定を定義する。基本方針・プロファイル設計は [docker-compose設計.md](docker-compose設計.md) を参照。設計の全体像は [可観測性設計](../../architecture/observability/可観測性設計.md) を参照。

---

## Observability サービス詳細設定

### Prometheus scrape 設定

```yaml
# infra/docker/prometheus/prometheus.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - /etc/prometheus/recording_rules.yaml
  - /etc/prometheus/alerting_rules.yaml

scrape_configs:
  # Prometheus 自身
  - job_name: prometheus
    static_configs:
      - targets: ["localhost:9090"]

  # auth-server (Rust)
  - job_name: auth-server-rust
    static_configs:
      - targets: ["auth-rust:8080"]
        labels:
          service: auth-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # config-server (Rust)
  - job_name: config-server-rust
    static_configs:
      - targets: ["config-rust:8080"]
        labels:
          service: config-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # saga-server (Rust)
  - job_name: saga-server-rust
    static_configs:
      - targets: ["saga-rust:8080"]
        labels:
          service: saga-server
          tier: system
          lang: rust
    metrics_path: /metrics

  # dlq-manager (Rust)
  - job_name: dlq-manager-rust
    static_configs:
      - targets: ["dlq-manager-rust:8080"]
        labels:
          service: dlq-manager
          tier: system
          lang: rust
    metrics_path: /metrics

  # bff-proxy (Go)
  - job_name: bff-proxy-go
    static_configs:
      - targets: ["bff-proxy:8080"]
        labels:
          service: bff-proxy
          tier: system
          lang: go
    metrics_path: /metrics

  # Kong API Gateway
  - job_name: kong
    static_configs:
      - targets: ["kong:8001"]
    metrics_path: /metrics
```

### Grafana 自動プロビジョニング

#### データソース

```yaml
# infra/docker/grafana/provisioning/datasources/datasources.yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    editable: false
    jsonData:
      derivedFields:
        - datasourceUid: jaeger
          matcherRegex: '"trace_id":"([a-f0-9]+)"'
          name: TraceID
          url: '$${__value.raw}'

  - name: Jaeger
    type: jaeger
    access: proxy
    uid: jaeger
    url: http://jaeger:16686
    editable: false
```

#### ダッシュボードプロビジョニング

```yaml
# infra/docker/grafana/provisioning/dashboards/dashboard.yml
apiVersion: 1

providers:
  - name: k1s0
    orgId: 1
    folder: k1s0
    type: file
    disableDeletion: false
    editable: true
    updateIntervalSeconds: 30
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: false
```

### Loki 設定

```yaml
# ローカル開発用（シングルインスタンス）
# 設定ファイル: infra/docker/loki/loki-config.yaml をマウント
# Kubernetes 環境では Promtail（DaemonSet）がログを収集し Loki に転送するが、
# ローカルでは各コンテナの stdout を直接 docker compose logs で確認する。
# Loki はダッシュボード経由でのログ検索用途で提供する。
```

### Jaeger 設定

```yaml
# OTLP プロトコルで各サービスからトレースデータを受信する
# - OTLP gRPC: 4317（サービスからの送信先）
# - OTLP HTTP: 4318（HTTP 経由の送信先）
# - UI: 16686（Jaeger UI）
```

| 項目 | 設定 |
| --- | --- |
| OTLP gRPC | `jaeger:4317` |
| OTLP HTTP | `jaeger:4318` |
| UI | `localhost:16686` |

---

## 関連ドキュメント

- [docker-compose設計.md](docker-compose設計.md) -- 基本方針・プロファイル設計
- [docker-compose-システムサービス設計.md](compose-システムサービス設計.md) -- auth-server・config-server・System プロファイル
- [docker-compose-インフラサービス設計.md](compose-インフラサービス設計.md) -- PostgreSQL・Keycloak・Kafka・Redis・Kong の詳細設定
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
