# モニタリング・アラート

本ドキュメントは、k1s0 におけるモニタリングとアラートの設定を定義する。

## 1. アーキテクチャ概要

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│   Service   │────▶│ OTel Collector   │────▶│ Prometheus  │
│  (metrics)  │     │                  │     │             │
└─────────────┘     └──────────────────┘     └─────────────┘
                            │                       │
┌─────────────┐             │                       ▼
│   Service   │─────────────┤               ┌─────────────┐
│   (logs)    │             │               │   Grafana   │
└─────────────┘             │               │             │
                            ▼               └─────────────┘
┌─────────────┐     ┌──────────────────┐
│   Service   │────▶│     Jaeger /     │
│  (traces)   │     │     Tempo        │
└─────────────┘     └──────────────────┘
```

## 2. メトリクス一覧

### 2.1 サービス共通メトリクス（Prometheus 形式）

#### リクエストメトリクス

```prometheus
# リクエスト数（カウンター）
k1s0_{service}_request_total{
  protocol="http|grpc",
  method="GET|POST|...",
  route="/api/v1/users",
  status_code="200"
}

# リクエスト失敗数（カウンター）
k1s0_{service}_request_failures_total{
  protocol="http|grpc",
  method="GET|POST|...",
  route="/api/v1/users",
  status_code="500",
  error_code="db.connection_failed"
}

# リクエストレイテンシ（ヒストグラム）
k1s0_{service}_request_duration_seconds{
  protocol="http|grpc",
  method="GET|POST|...",
  route="/api/v1/users"
}
```

#### 依存関係メトリクス

```prometheus
# 外部依存の失敗数（カウンター）
k1s0_{service}_dependency_failures_total{
  dependency="postgres|redis|config-service",
  error_kind="connection|timeout|query"
}

# 外部依存のレイテンシ（ヒストグラム）
k1s0_{service}_dependency_duration_seconds{
  dependency="postgres|redis|config-service",
  operation="query|get|set"
}
```

#### システムメトリクス

```prometheus
# 設定取得失敗（カウンター）
k1s0_{service}_config_fetch_failures_total{
  source="yaml|db"
}

# アクティブ接続数（ゲージ）
k1s0_{service}_active_connections{
  type="http|grpc|db"
}

# ゴルーチン数（Go の場合）
k1s0_{service}_goroutines

# メモリ使用量
k1s0_{service}_memory_bytes{
  type="heap|stack"
}
```

### 2.2 ビジネスメトリクス例

```prometheus
# ユーザー登録数
k1s0_user_service_registrations_total{
  plan="free|premium"
}

# 注文処理数
k1s0_order_service_orders_total{
  status="created|completed|cancelled"
}
```

## 3. ログフォーマット

### 3.1 標準ログ形式（JSON）

```json
{
  "timestamp": "2026-01-28T10:30:00.123Z",
  "level": "INFO",
  "service_name": "user-service",
  "env": "prod",
  "trace_id": "abc123def456",
  "span_id": "span789",
  "message": "User created successfully",
  "user_id": 12345,
  "grpc.method": "/user.v1.UserService/CreateUser",
  "duration_ms": 45
}
```

### 3.2 エラーログ形式

```json
{
  "timestamp": "2026-01-28T10:30:00.123Z",
  "level": "ERROR",
  "service_name": "user-service",
  "env": "prod",
  "trace_id": "abc123def456",
  "span_id": "span789",
  "message": "Failed to create user",
  "error.kind": "dependency",
  "error.code": "db.connection_failed",
  "error.message": "Connection refused to postgres:5432",
  "grpc.status_code": 14
}
```

### 3.3 ログレベルガイドライン

| レベル | 用途 | 例 |
|--------|------|-----|
| `DEBUG` | 開発時のデバッグ情報 | 変数値、SQL クエリ |
| `INFO` | 正常な処理の記録 | リクエスト受信、処理完了 |
| `WARN` | 警告（処理は継続） | リトライ発生、非推奨 API 使用 |
| `ERROR` | エラー（処理失敗） | DB 接続失敗、バリデーションエラー |

### 3.4 ログ検索例（Grafana Loki）

```logql
# 特定サービスのエラーログ
{service_name="user-service"} |= "ERROR"

# 特定の trace_id に関連するログ
{env="prod"} | json | trace_id="abc123def456"

# エラーコード別集計
sum by (error_code) (
  count_over_time(
    {service_name="user-service"} | json | level="ERROR" [1h]
  )
)
```

## 4. トレーシング設定

### 4.1 OTel Collector 設定

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true
  prometheus:
    endpoint: "0.0.0.0:8889"

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
```

### 4.2 サービス側設定例（Rust）

```yaml
# config/prod.yaml
observability:
  otlp_endpoint: "http://otel-collector:4317"
  service_name: "user-service"
  environment: "prod"
  sampling_rate: 0.1  # 10% サンプリング
```

### 4.3 サンプリング戦略

| 環境 | サンプリング率 | 理由 |
|------|---------------|------|
| dev | 100% | デバッグのため全トレース取得 |
| stg | 50% | 負荷テスト時のオーバーヘッド軽減 |
| prod | 10% | コスト最適化、エラー時は 100% |

## 5. アラートルール

### 5.1 重大度定義

| 重大度 | 対応時間 | 通知先 |
|--------|---------|--------|
| Critical | 即時対応 | PagerDuty + Slack #alerts |
| Warning | 1 時間以内 | Slack #alerts |
| Info | 翌営業日 | Slack #ops |

### 5.2 Prometheus アラートルール例

```yaml
# alerts.yaml
groups:
  - name: k1s0-service-alerts
    rules:
      # サービスダウン
      - alert: ServiceDown
        expr: up{job=~"k1s0-.*"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service {{ $labels.job }} is down"
          description: "Service {{ $labels.job }} has been down for more than 1 minute."

      # 高エラーレート
      - alert: HighErrorRate
        expr: |
          sum(rate(k1s0_request_failures_total[5m])) by (service_name)
          /
          sum(rate(k1s0_request_total[5m])) by (service_name)
          > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate on {{ $labels.service_name }}"
          description: "Error rate is {{ $value | humanizePercentage }} for the last 5 minutes."

      # 高レイテンシ
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            sum(rate(k1s0_request_duration_seconds_bucket[5m])) by (le, service_name)
          ) > 1.0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency on {{ $labels.service_name }}"
          description: "P99 latency is {{ $value | humanizeDuration }}."

      # DB 接続失敗
      - alert: DatabaseConnectionFailure
        expr: |
          increase(k1s0_dependency_failures_total{dependency="postgres"}[5m]) > 10
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Database connection failures on {{ $labels.service_name }}"
          description: "{{ $value }} database connection failures in the last 5 minutes."

      # Pod 再起動頻発
      - alert: FrequentPodRestarts
        expr: |
          increase(kube_pod_container_status_restarts_total{namespace="k1s0"}[1h]) > 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Frequent pod restarts: {{ $labels.pod }}"
          description: "Pod {{ $labels.pod }} has restarted {{ $value }} times in the last hour."

      # メモリ使用率
      - alert: HighMemoryUsage
        expr: |
          container_memory_usage_bytes{namespace="k1s0"}
          /
          container_spec_memory_limit_bytes{namespace="k1s0"}
          > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage: {{ $labels.pod }}"
          description: "Memory usage is {{ $value | humanizePercentage }}."
```

### 5.3 SLO ベースアラート

```yaml
# slo-alerts.yaml
groups:
  - name: k1s0-slo-alerts
    rules:
      # 可用性 SLO 違反（99.9%）
      - alert: AvailabilitySLOBreach
        expr: |
          (
            1 - (
              sum(rate(k1s0_request_failures_total[30m]))
              /
              sum(rate(k1s0_request_total[30m]))
            )
          ) < 0.999
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Availability SLO breach"
          description: "Availability is {{ $value | humanizePercentage }}, below 99.9% SLO."

      # レイテンシ SLO 違反（P99 < 500ms）
      - alert: LatencySLOBreach
        expr: |
          histogram_quantile(0.99,
            sum(rate(k1s0_request_duration_seconds_bucket[30m])) by (le)
          ) > 0.5
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Latency SLO breach"
          description: "P99 latency is {{ $value | humanizeDuration }}, exceeds 500ms SLO."
```

## 6. ダッシュボード

### 6.1 必須ダッシュボード

| ダッシュボード | 内容 |
|--------------|------|
| Service Overview | 全サービスの健全性サマリ |
| Service Detail | 個別サービスの詳細メトリクス |
| SLO Dashboard | SLO/SLI の達成状況 |
| Error Analysis | エラーの分析・トレンド |
| Dependencies | 外部依存の健全性 |

### 6.2 Grafana ダッシュボード設定

ダッシュボード定義は `deploy/monitoring/dashboards/` に格納:

```
deploy/monitoring/dashboards/
├── service-overview.json
├── service-detail.json
├── slo-dashboard.json
├── error-analysis.json
└── dependencies.json
```

## 7. オンコール対応

### 7.1 アラート受信時のフロー

1. アラート受信
2. [トラブルシューティングガイド](troubleshooting.md) を確認
3. 該当する [Runbook](runbooks/) を実行
4. 解決できない場合はエスカレーション

### 7.2 エスカレーションパス

| レベル | 担当 | 連絡先 |
|--------|------|--------|
| L1 | オンコール担当 | PagerDuty |
| L2 | サービスオーナー | Slack #escalation |
| L3 | インフラチーム | Slack #infra |

## 関連ドキュメント

- [観測性規約](../conventions/observability.md)
- [エラーハンドリング規約](../conventions/error-handling.md)
- [トラブルシューティング](troubleshooting.md)
- [SLA 定義](sla.md)
