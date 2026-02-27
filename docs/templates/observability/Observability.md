# テンプレート仕様 — Observability

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Observability（可観測性）** リソースのテンプレート仕様を定義する。Prometheus の ServiceMonitor（メトリクス収集定義）と PrometheusRule（アラート定義）を、サービスの `tier` に応じて自動生成する。

可観測性設計の全体像は [可観測性設計](../../observability/overview/可観測性設計.md) を参照。

## 生成対象

| kind       | ServiceMonitor | PrometheusRule (alerts) |
| ---------- | -------------- | ----------------------- |
| `server`   | 生成する       | 生成する                |
| `bff`      | 生成する       | 生成する                |
| `client`   | 生成しない     | 生成しない              |
| `library`  | 生成しない     | 生成しない              |
| `database` | 生成しない     | 生成しない              |

## 配置パス

生成されるリソースファイルは `infra/observability/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル          | 配置パス                                                     |
| ----------------- | ------------------------------------------------------------ |
| ServiceMonitor    | `infra/observability/{{ service_name }}/servicemonitor.yaml`  |
| PrometheusRule    | `infra/observability/{{ service_name }}/alerts.yaml`          |

## テンプレートファイル一覧

テンプレートは `CLI/templates/observability/` 配下に配置する。

| テンプレートファイル               | 生成先                                                       | 説明                              |
| ---------------------------------- | ------------------------------------------------------------ | --------------------------------- |
| `servicemonitor.yaml.tera`         | `infra/observability/{{ service_name }}/servicemonitor.yaml`  | Prometheus ServiceMonitor 定義    |
| `alerts.yaml.tera`                 | `infra/observability/{{ service_name }}/alerts.yaml`          | PrometheusRule（アラートルール）  |

### ディレクトリ構成

```
CLI/
└── templates/
    └── observability/
        ├── servicemonitor.yaml.tera
        └── alerts.yaml.tera
```

## 使用するテンプレート変数

Observability テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | ServiceMonitor | PrometheusRule | 用途                                     |
| -------------------- | ------ | -------------- | -------------- | ---------------------------------------- |
| `service_name`       | String | 用             | 用             | リソース名、セレクター、アラートラベル   |
| `service_name_snake` | String | -              | 用             | アラートルール名のプレフィクス           |
| `namespace`          | String | 用             | 用             | リソースの配置先 Namespace               |
| `tier`               | String | 用             | 用             | メトリクス収集間隔、アラート閾値の決定   |
| `server_port`        | Number | 用             | -              | メトリクスエンドポイントのポート番号     |

### Tier 別メトリクス収集設定

| Tier       | 収集間隔 (interval) | スクレイプタイムアウト |
| ---------- | ------------------- | ---------------------- |
| `system`   | 15s                 | 10s                    |
| `business` | 30s                 | 15s                    |
| `service`  | 30s                 | 15s                    |

### Tier 別アラート閾値

| アラート               | system    | business  | service   |
| ---------------------- | --------- | --------- | --------- |
| 高エラー率 (threshold) | 1%        | 5%        | 5%        |
| 高エラー率 (for)       | 1m        | 5m        | 5m        |
| 高エラー率 (severity)  | critical  | warning   | warning   |
| 高レイテンシ P99       | 500ms     | 1s        | 2s        |
| 高レイテンシ (for)     | 5m        | 10m       | 10m       |
| 高レイテンシ (severity)| warning   | warning   | info      |
| Pod 再起動 (threshold) | 3         | 5         | 5         |
| Pod 再起動 (for)       | 5m        | 10m       | 10m       |
| Pod 再起動 (severity)  | critical  | warning   | warning   |

---

## ServiceMonitor テンプレート（servicemonitor.yaml.tera）

Prometheus の ServiceMonitor リソースを定義する。メトリクスの収集間隔、エンドポイント、ラベルセレクターを設定する。

```tera
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ service_name }}
  namespaceSelector:
    matchNames:
      - {{ namespace }}
  endpoints:
    - port: http-metrics
      path: /metrics
{% if tier == "system" %}
      interval: 15s
      scrapeTimeout: 10s
{% else %}
      interval: 30s
      scrapeTimeout: 15s
{% endif %}
      metricRelabelings:
        - sourceLabels: [__name__]
          regex: "go_.*"
          action: drop
```

### ポイント

- `selector` でサービス固有の Pod を対象にメトリクスを収集する
- system Tier は収集間隔を短く（15秒）設定し、システム基盤の状態変化を迅速に検知する
- business/service Tier は収集間隔を30秒に設定し、メトリクスストレージの負荷を抑える
- `metricRelabelings` で Go ランタイムの内部メトリクス（`go_.*`）をドロップし、カーディナリティを削減する
- メトリクスのパスは `/metrics`（Prometheus 標準）を使用する

---

## PrometheusRule テンプレート（alerts.yaml.tera）

PrometheusRule リソースを定義する。高エラー率、高レイテンシ、Pod 再起動の3種類のアラートルールを生成する。

```tera
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: {{ service_name }}-alerts
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
spec:
  groups:
    - name: {{ service_name_snake }}.rules
      rules:
        # High Error Rate
        - alert: {{ service_name_snake }}_high_error_rate
          expr: |
            (
              sum(rate(http_requests_total{service="{{ service_name }}", code=~"5.."}[5m]))
              /
              sum(rate(http_requests_total{service="{{ service_name }}"}[5m]))
{% if tier == "system" %}
            ) > 0.01
          for: 1m
{% elif tier == "business" %}
            ) > 0.05
          for: 5m
{% elif tier == "service" %}
            ) > 0.05
          for: 5m
{% endif %}
          labels:
{% if tier == "system" %}
            severity: critical
{% else %}
            severity: warning
{% endif %}
            service: {{ service_name }}
            tier: {{ tier }}
          annotations:
            summary: "High error rate on {{ service_name }}"
            description: "Error rate is above threshold for {{ service_name }} in {{ namespace }}"

        # High Latency (P99)
        - alert: {{ service_name_snake }}_high_latency
          expr: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_seconds_bucket{service="{{ service_name }}"}[5m])) by (le)
{% if tier == "system" %}
            ) > 0.5
          for: 5m
          labels:
            severity: warning
{% elif tier == "business" %}
            ) > 1.0
          for: 10m
          labels:
            severity: warning
{% elif tier == "service" %}
            ) > 2.0
          for: 10m
          labels:
            severity: info
{% endif %}
            service: {{ service_name }}
            tier: {{ tier }}
          annotations:
            summary: "High P99 latency on {{ service_name }}"
            description: "P99 latency is above threshold for {{ service_name }} in {{ namespace }}"

        # Pod Restart
        - alert: {{ service_name_snake }}_pod_restart
          expr: |
{% if tier == "system" %}
            increase(kube_pod_container_status_restarts_total{namespace="{{ namespace }}", pod=~"{{ service_name }}.*"}[1h]) > 3
          for: 5m
          labels:
            severity: critical
{% elif tier == "business" %}
            increase(kube_pod_container_status_restarts_total{namespace="{{ namespace }}", pod=~"{{ service_name }}.*"}[1h]) > 5
          for: 10m
          labels:
            severity: warning
{% elif tier == "service" %}
            increase(kube_pod_container_status_restarts_total{namespace="{{ namespace }}", pod=~"{{ service_name }}.*"}[1h]) > 5
          for: 10m
          labels:
            severity: warning
{% endif %}
            service: {{ service_name }}
            tier: {{ tier }}
          annotations:
            summary: "Pod restarts detected for {{ service_name }}"
            description: "Pod {{ service_name }} in {{ namespace }} has restarted multiple times"
```

### ポイント

- **高エラー率**: HTTP 5xx エラーの割合を監視する。system Tier は閾値1%・severity critical とし、システム基盤の異常を厳格に検知する
- **高レイテンシ（P99）**: レスポンスタイムの P99 パーセンタイルを監視する。system Tier は500ms、business は1秒、service は2秒を閾値とする
- **Pod 再起動**: 1時間あたりの Pod 再起動回数を監視する。system Tier は3回で critical、business/service は5回で warning とする
- 全アラートに `service` と `tier` ラベルを付与し、アラートの分類・ルーティングを容易にする

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                | 選択肢                              | 生成への影響                                                  |
| ------------------- | ----------------------------------- | ------------------------------------------------------------- |
| Tier (`tier`)       | `system`                            | 収集間隔15秒、厳格な閾値、severity critical                  |
| Tier (`tier`)       | `business` / `service`              | 収集間隔30秒、標準的な閾値、severity warning/info            |
| kind (`kind`)       | `server` / `bff` 以外              | Observability リソースを生成しない                            |

---

## 生成例

### system Tier のサーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "service_name_snake": "auth_service",
  "tier": "system",
  "namespace": "k1s0-system",
  "server_port": 80
}
```

生成されるファイル:
- `infra/observability/auth-service/servicemonitor.yaml` -- interval=15s、scrapeTimeout=10s
- `infra/observability/auth-service/alerts.yaml` -- エラー率1%/critical、レイテンシ500ms、Pod再起動3回/critical

### service Tier のサーバーの場合

入力:
```json
{
  "service_name": "order-server",
  "service_name_snake": "order_server",
  "tier": "service",
  "namespace": "k1s0-service",
  "server_port": 80
}
```

生成されるファイル:
- `infra/observability/order-server/servicemonitor.yaml` -- interval=30s、scrapeTimeout=15s
- `infra/observability/order-server/alerts.yaml` -- エラー率5%/warning、レイテンシ2s/info、Pod再起動5回/warning

---

## 関連ドキュメント

- [可観測性設計](../../observability/overview/可観測性設計.md) -- 可観測性の全体設計（メトリクス・ログ・トレース）
- [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-ServiceMesh](../middleware/ServiceMesh.md) -- ServiceMesh テンプレート仕様
- [テンプレート仕様-Helm](../infrastructure/Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-Config](../data/Config.md) -- Config テンプレート仕様（トレース設定連携）
- [テンプレート仕様-Kong](../middleware/Kong.md) -- Kong テンプレート仕様（ロギングプラグイン連携）
- [テンプレート仕様-CICD](../infrastructure/CICD.md) -- CI/CD テンプレート仕様
