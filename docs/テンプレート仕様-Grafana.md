# テンプレート仕様 — Grafana

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Grafana ダッシュボード** テンプレートの仕様を定義する。Kubernetes ConfigMap として JSON ダッシュボード定義を生成し、Grafana のプロビジョニング機能で自動インポートする。Overview（RED メトリクス）、Service Detail（個別サービス詳細）、SLO（可用性・エラーバジェット・バーンレート）の3種類のダッシュボードを、サービスの `tier` に応じて自動生成する。

可観測性設計の全体像は [可観測性設計](可観測性設計.md) を参照。

## 生成対象

| kind       | Overview Dashboard | Service Detail Dashboard | SLO Dashboard |
| ---------- | ------------------ | ------------------------ | ------------- |
| `server`   | 生成する           | 生成する                 | 生成する      |
| `bff`      | 生成する           | 生成する                 | 生成する      |
| `client`   | 生成しない         | 生成しない               | 生成しない    |
| `library`  | 生成しない         | 生成しない               | 生成しない    |
| `database` | 生成しない         | 生成しない               | 生成しない    |

> **Infrastructure / Kafka / Kong / Istio / Database ダッシュボード** はクラスタ共有リソースであり、サービス個別に生成するものではない。これらは別途 Helm Chart でデプロイする。

## 配置パス

生成されるリソースファイルは `infra/grafana/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル                      | 配置パス                                                              |
| ----------------------------- | --------------------------------------------------------------------- |
| Overview Dashboard ConfigMap  | `infra/grafana/{{ service_name }}/dashboard-overview.yaml`            |
| Service Detail Dashboard ConfigMap | `infra/grafana/{{ service_name }}/dashboard-service-detail.yaml` |
| SLO Dashboard ConfigMap       | `infra/grafana/{{ service_name }}/dashboard-slo.yaml`                 |

## テンプレートファイル一覧

テンプレートは `CLI/templates/grafana/` 配下に配置する。

| テンプレートファイル                     | 生成先                                                              | 説明                                  |
| ---------------------------------------- | ------------------------------------------------------------------- | ------------------------------------- |
| `dashboard-overview.yaml.tera`           | `infra/grafana/{{ service_name }}/dashboard-overview.yaml`          | Overview ダッシュボード ConfigMap     |
| `dashboard-service-detail.yaml.tera`     | `infra/grafana/{{ service_name }}/dashboard-service-detail.yaml`    | Service Detail ダッシュボード ConfigMap |
| `dashboard-slo.yaml.tera`               | `infra/grafana/{{ service_name }}/dashboard-slo.yaml`               | SLO ダッシュボード ConfigMap          |

### ディレクトリ構成

```
CLI/
└── templates/
    └── grafana/
        ├── dashboard-overview.yaml.tera
        ├── dashboard-service-detail.yaml.tera
        └── dashboard-slo.yaml.tera
```

## 使用するテンプレート変数

Grafana テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | Overview | Service Detail | SLO | 用途                                     |
| -------------------- | ------ | -------- | -------------- | --- | ---------------------------------------- |
| `service_name`       | String | 用       | 用             | 用  | ダッシュボードタイトル、PromQL フィルタ  |
| `service_name_snake` | String | 用       | 用             | 用  | ConfigMap 名のプレフィクス               |
| `namespace`          | String | 用       | 用             | 用  | PromQL の namespace フィルタ             |
| `tier`               | String | 用       | 用             | 用  | SLO 目標値、閾値ラインの決定            |
| `server_port`        | Number | -        | 用             | -   | エンドポイント情報の表示                 |

### Tier 別 SLO 目標値

| Tier       | 可用性目標  | P99 レイテンシ目標 | エラーバジェット（30日） |
| ---------- | ----------- | ------------------ | ------------------------ |
| `system`   | 99.95%      | < 200ms            | 21.6 分                  |
| `business` | 99.9%       | < 500ms            | 43.2 分                  |
| `service`  | 99.9%       | < 1s               | 43.2 分                  |

---

## Overview ダッシュボード テンプレート（dashboard-overview.yaml.tera）

サービスの RED メトリクス（Rate, Errors, Duration）を一覧表示する Overview ダッシュボードを ConfigMap として定義する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-dashboard-overview
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
    grafana_dashboard: "1"
data:
  overview.json: |
    {
      "dashboard": {
        "title": "{{ service_name }} - Overview",
        "tags": ["{{ tier }}", "overview", "{{ service_name }}"],
        "templating": {
          "list": [
            {
              "name": "interval",
              "type": "interval",
              "options": [
                {"text": "1m", "value": "1m"},
                {"text": "5m", "value": "5m"},
                {"text": "15m", "value": "15m"}
              ],
              "current": {"text": "5m", "value": "5m"}
            }
          ]
        },
        "panels": [
          {
            "title": "Request Rate",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 8, "x": 0, "y": 0},
            "targets": [
              {
                "expr": "rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[$interval])",
                "legendFormat": "{% raw %}{{method}} {{path}}{% endraw %}"
              }
            ]
          },
          {
            "title": "Error Rate",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 8, "x": 8, "y": 0},
            "targets": [
              {
                "expr": "rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\", status=~\"5..\"}[$interval]) / rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[$interval])",
                "legendFormat": "error_rate"
              }
            ],
            "thresholds": [
{% if tier == "system" %}
              {"value": 0.01, "colorMode": "critical"}
{% else %}
              {"value": 0.05, "colorMode": "critical"}
{% endif %}
            ]
          },
          {
            "title": "P50 / P95 / P99 Latency",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 8, "x": 16, "y": 0},
            "targets": [
              {
                "expr": "histogram_quantile(0.50, rate(http_request_duration_seconds_bucket{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[$interval]))",
                "legendFormat": "P50"
              },
              {
                "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[$interval]))",
                "legendFormat": "P95"
              },
              {
                "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[$interval]))",
                "legendFormat": "P99"
              }
            ]
          },
          {
            "title": "Pod CPU Usage",
            "type": "gauge",
            "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
            "targets": [
              {
                "expr": "rate(container_cpu_usage_seconds_total{namespace=\"{{ namespace }}\", pod=~\"{{ service_name }}.*\"}[5m])",
                "legendFormat": "{% raw %}{{pod}}{% endraw %}"
              }
            ]
          },
          {
            "title": "Pod Memory Usage",
            "type": "gauge",
            "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
            "targets": [
              {
                "expr": "container_memory_working_set_bytes{namespace=\"{{ namespace }}\", pod=~\"{{ service_name }}.*\"} / container_spec_memory_limit_bytes{namespace=\"{{ namespace }}\", pod=~\"{{ service_name }}.*\"}",
                "legendFormat": "{% raw %}{{pod}}{% endraw %}"
              }
            ]
          }
        ]
      }
    }
```

### ポイント

- `grafana_dashboard: "1"` ラベルにより、Grafana のサイドカープロビジョナーが ConfigMap を自動検出してダッシュボードをインポートする
- RED メトリクス（Request Rate / Error Rate / Latency）を横並びで配置し、サービスの健全性を一目で把握できる
- エラーレートの閾値ラインは Tier に応じて変更する（system: 1%、business/service: 5%）
- Pod の CPU / メモリ使用率を Gauge パネルで表示し、リソース状況を可視化する

---

## Service Detail ダッシュボード テンプレート（dashboard-service-detail.yaml.tera）

個別サービスの詳細メトリクスを表示する Service Detail ダッシュボードを ConfigMap として定義する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-dashboard-service-detail
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
    grafana_dashboard: "1"
data:
  service-detail.json: |
    {
      "dashboard": {
        "title": "{{ service_name }} - Service Detail",
        "tags": ["{{ tier }}", "service-detail", "{{ service_name }}"],
        "panels": [
          {
            "title": "gRPC Request Rate",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
            "targets": [
              {
                "expr": "rate(grpc_server_handled_total{grpc_service=~\".*{{ service_name }}.*\", namespace=\"{{ namespace }}\"}[5m])",
                "legendFormat": "{% raw %}{{grpc_method}} [{{grpc_code}}]{% endraw %}"
              }
            ]
          },
          {
            "title": "gRPC Latency (P99)",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
            "targets": [
              {
                "expr": "histogram_quantile(0.99, rate(grpc_server_handling_seconds_bucket{grpc_service=~\".*{{ service_name }}.*\", namespace=\"{{ namespace }}\"}[5m]))",
                "legendFormat": "{% raw %}{{grpc_method}}{% endraw %}"
              }
            ]
          },
          {
            "title": "HTTP Request Rate by Status",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
            "targets": [
              {
                "expr": "rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[5m])",
                "legendFormat": "{% raw %}{{method}} {{path}} [{{status}}]{% endraw %}"
              }
            ]
          },
          {
            "title": "DB Query Duration",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
            "targets": [
              {
                "expr": "histogram_quantile(0.99, rate(db_query_duration_seconds_bucket{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[5m]))",
                "legendFormat": "{% raw %}{{query_name}} ({{table}}){% endraw %}"
              }
            ]
          },
          {
            "title": "Kafka Messages Produced",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
            "targets": [
              {
                "expr": "rate(kafka_messages_produced_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[5m])",
                "legendFormat": "{% raw %}{{topic}}{% endraw %}"
              }
            ]
          },
          {
            "title": "Kafka Messages Consumed",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16},
            "targets": [
              {
                "expr": "rate(kafka_messages_consumed_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[5m])",
                "legendFormat": "{% raw %}{{topic}} ({{consumer_group}}){% endraw %}"
              }
            ]
          },
          {
            "title": "Pod Restarts",
            "type": "stat",
            "gridPos": {"h": 4, "w": 8, "x": 0, "y": 24},
            "targets": [
              {
                "expr": "increase(kube_pod_container_status_restarts_total{namespace=\"{{ namespace }}\", pod=~\"{{ service_name }}.*\"}[1h])",
                "legendFormat": "{% raw %}{{pod}}{% endraw %}"
              }
            ]
          },
          {
            "title": "Pod Ready",
            "type": "stat",
            "gridPos": {"h": 4, "w": 8, "x": 8, "y": 24},
            "targets": [
              {
                "expr": "kube_pod_status_ready{namespace=\"{{ namespace }}\", pod=~\"{{ service_name }}.*\", condition=\"true\"}",
                "legendFormat": "{% raw %}{{pod}}{% endraw %}"
              }
            ]
          },
          {
            "title": "Container Port",
            "type": "stat",
            "gridPos": {"h": 4, "w": 8, "x": 16, "y": 24},
            "targets": [
              {
                "expr": "vector({{ server_port }})",
                "legendFormat": "server_port"
              }
            ]
          }
        ]
      }
    }
```

### ポイント

- gRPC と HTTP の両方のメトリクスを表示し、サービスの通信方式に応じた可視化を提供する
- DB クエリの P99 レイテンシをクエリ名・テーブル別に表示し、ボトルネックの特定を容易にする
- Kafka の Produce / Consume レートをトピック別に表示し、メッセージングの健全性を監視する
- Pod のリスタート回数と Ready 状態を Stat パネルで表示し、Pod の安定性を把握する

---

## SLO ダッシュボード テンプレート（dashboard-slo.yaml.tera）

SLI/SLO のバーンレートとエラーバジェットを表示する SLO ダッシュボードを ConfigMap として定義する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-dashboard-slo
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
    grafana_dashboard: "1"
data:
  slo.json: |
    {
      "dashboard": {
        "title": "{{ service_name }} - SLO",
        "tags": ["{{ tier }}", "slo", "{{ service_name }}"],
        "panels": [
          {
            "title": "Availability (30d)",
            "type": "gauge",
            "gridPos": {"h": 8, "w": 8, "x": 0, "y": 0},
            "targets": [
              {
                "expr": "sum(rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\", status!~\"5..\"}[30d])) / sum(rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[30d]))",
                "legendFormat": "availability"
              }
            ],
            "fieldConfig": {
              "defaults": {
                "thresholds": {
                  "steps": [
                    {"value": 0, "color": "red"},
{% if tier == "system" %}
                    {"value": 0.9995, "color": "green"}
{% else %}
                    {"value": 0.999, "color": "green"}
{% endif %}
                  ]
                },
                "unit": "percentunit",
{% if tier == "system" %}
                "min": 0.999,
{% else %}
                "min": 0.998,
{% endif %}
                "max": 1
              }
            }
          },
          {
            "title": "Error Budget Remaining",
            "type": "gauge",
            "gridPos": {"h": 8, "w": 8, "x": 8, "y": 0},
            "targets": [
              {
                "expr": "slo:error_budget:remaining{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}",
                "legendFormat": "error_budget"
              }
            ],
            "fieldConfig": {
              "defaults": {
                "thresholds": {
                  "steps": [
                    {"value": 0, "color": "red"},
                    {"value": 0.25, "color": "orange"},
                    {"value": 0.5, "color": "yellow"},
                    {"value": 0.75, "color": "green"}
                  ]
                },
                "unit": "percentunit",
                "min": 0,
                "max": 1
              }
            }
          },
          {
            "title": "Burn Rate (1h)",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 8, "x": 16, "y": 0},
            "targets": [
              {
                "expr": "1 - (sum(rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\", status!~\"5..\"}[1h])) / sum(rate(http_requests_total{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[1h])))",
                "legendFormat": "burn_rate"
              }
            ],
            "thresholds": [
{% if tier == "system" %}
              {"value": 0.0005, "colorMode": "warning"},
              {"value": 0.01, "colorMode": "critical"}
{% else %}
              {"value": 0.001, "colorMode": "warning"},
              {"value": 0.05, "colorMode": "critical"}
{% endif %}
            ]
          },
          {
            "title": "P99 Latency vs SLO Target",
            "type": "timeseries",
            "gridPos": {"h": 8, "w": 24, "x": 0, "y": 8},
            "targets": [
              {
                "expr": "histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{service=\"{{ service_name }}\", namespace=\"{{ namespace }}\"}[5m]))",
                "legendFormat": "P99 actual"
              },
              {
{% if tier == "system" %}
                "expr": "vector(0.2)",
                "legendFormat": "SLO target (200ms)"
{% elif tier == "business" %}
                "expr": "vector(0.5)",
                "legendFormat": "SLO target (500ms)"
{% elif tier == "service" %}
                "expr": "vector(1.0)",
                "legendFormat": "SLO target (1s)"
{% endif %}
              }
            ]
          }
        ]
      }
    }
```

### ポイント

- **可用性ゲージ**: 30日間の可用性を表示し、SLO 目標値（system: 99.95%、business/service: 99.9%）を閾値として色分けする
- **エラーバジェット残量**: Recording Rule `slo:error_budget:remaining` の値をゲージで表示し、0%/25%/50%/75% の段階で色分けする
- **バーンレート**: 直近1時間のエラーバーンレートをタイムシリーズで表示し、SLO の消費速度を可視化する
- **P99 レイテンシ対 SLO 目標**: 実測値と SLO 目標値を同一パネルに描画し、目標との乖離を視覚的に把握する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                | 選択肢                              | 生成への影響                                                  |
| ------------------- | ----------------------------------- | ------------------------------------------------------------- |
| Tier (`tier`)       | `system`                            | SLO 目標値 99.95%、閾値ライン厳格、P99 目標 200ms            |
| Tier (`tier`)       | `business`                          | SLO 目標値 99.9%、P99 目標 500ms                             |
| Tier (`tier`)       | `service`                           | SLO 目標値 99.9%、P99 目標 1s                                |
| kind (`kind`)       | `server` / `bff` 以外              | Grafana ダッシュボードを生成しない                            |

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
- `infra/grafana/auth-service/dashboard-overview.yaml` -- エラーレート閾値1%、RED メトリクスパネル
- `infra/grafana/auth-service/dashboard-service-detail.yaml` -- gRPC/HTTP/DB/Kafka の詳細メトリクス
- `infra/grafana/auth-service/dashboard-slo.yaml` -- 可用性目標99.95%、P99目標200ms、バーンレート

### business Tier のサーバーの場合

入力:
```json
{
  "service_name": "inventory-server",
  "service_name_snake": "inventory_server",
  "tier": "business",
  "namespace": "k1s0-business",
  "server_port": 80
}
```

生成されるファイル:
- `infra/grafana/inventory-server/dashboard-overview.yaml` -- エラーレート閾値5%、RED メトリクスパネル
- `infra/grafana/inventory-server/dashboard-service-detail.yaml` -- gRPC/HTTP/DB/Kafka の詳細メトリクス
- `infra/grafana/inventory-server/dashboard-slo.yaml` -- 可用性目標99.9%、P99目標500ms、バーンレート

### service Tier の BFF の場合

入力:
```json
{
  "service_name": "order-bff",
  "service_name_snake": "order_bff",
  "tier": "service",
  "namespace": "k1s0-service",
  "server_port": 3000
}
```

生成されるファイル:
- `infra/grafana/order-bff/dashboard-overview.yaml` -- エラーレート閾値5%、RED メトリクスパネル
- `infra/grafana/order-bff/dashboard-service-detail.yaml` -- HTTP/Kafka の詳細メトリクス
- `infra/grafana/order-bff/dashboard-slo.yaml` -- 可用性目標99.9%、P99目標1s、バーンレート

---

## 関連ドキュメント

- [可観測性設計](可観測性設計.md) -- 可観測性の全体設計（Grafana ダッシュボード構成・パネル定義）
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様（ServiceMonitor・PrometheusRule）
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-Config](テンプレート仕様-Config.md) -- Config テンプレート仕様
