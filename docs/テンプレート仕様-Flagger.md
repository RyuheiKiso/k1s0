# テンプレート仕様 — Flagger

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Flagger カナリアリリースリソース** のテンプレート仕様を定義する。Flagger の Canary CRD（カナリアデプロイ制御）と MetricTemplate（カスタムメトリクス定義）を、サービスの `tier` に応じて自動生成する。

カナリアリリースの段階的ロールアウト設計は [サービスメッシュ設計](サービスメッシュ設計.md) を、CI/CD パイプラインとの連携は [CI-CD設計](CI-CD設計.md) を参照。

## 生成対象

| kind       | Canary | MetricTemplate |
| ---------- | ------ | -------------- |
| `server`   | 生成する | 生成する     |
| `bff`      | 生成する | 生成する     |
| `client`   | 生成しない | 生成しない |
| `library`  | 生成しない | 生成しない |
| `database` | 生成しない | 生成しない |

## 配置パス

生成されるリソースファイルは `infra/flagger/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル        | 配置パス                                              |
| --------------- | ----------------------------------------------------- |
| Canary          | `infra/flagger/{{ service_name }}/canary.yaml`        |
| MetricTemplate  | `infra/flagger/{{ service_name }}/metric-template.yaml` |

## テンプレートファイル一覧

テンプレートは `CLI/templates/flagger/` 配下に配置する。

| テンプレートファイル          | 生成先                                                  | 説明                              |
| ----------------------------- | ------------------------------------------------------- | --------------------------------- |
| `canary.yaml.tera`            | `infra/flagger/{{ service_name }}/canary.yaml`          | Flagger Canary CRD 定義          |
| `metric-template.yaml.tera`   | `infra/flagger/{{ service_name }}/metric-template.yaml` | Flagger MetricTemplate 定義      |

### ディレクトリ構成

```
CLI/
└── templates/
    └── flagger/
        ├── canary.yaml.tera
        └── metric-template.yaml.tera
```

## 使用するテンプレート変数

Flagger テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名         | 型     | Canary | MetricTemplate | 用途                                           |
| -------------- | ------ | ------ | -------------- | ---------------------------------------------- |
| `service_name` | String | 用     | 用             | リソース名、targetRef、メトリクスクエリ        |
| `tier`         | String | 用     | 用             | progressDeadline、threshold、メトリクス閾値の決定 |
| `namespace`    | String | 用     | 用             | リソースの配置先 Namespace                     |
| `server_port`  | Number | 用     | -              | Canary Service のポート番号                    |

### Tier 別カナリア分析設定

| 設定                     | system | business | service |
| ------------------------ | ------ | -------- | ------- |
| progressDeadlineSeconds  | 300    | 600      | 600     |
| interval                 | 30s    | 30s      | 30s     |
| threshold（失敗閾値）    | 3      | 5        | 5       |
| maxWeight                | 50     | 50       | 50      |
| stepWeight               | 10     | 10       | 10      |

### Tier 別メトリクス閾値

| メトリクス           | system | business | service |
| -------------------- | ------ | -------- | ------- |
| request-success-rate | 99     | 95       | 95      |
| request-duration     | 500ms  | 1000ms   | 1000ms  |

---

## Canary テンプレート（canary.yaml.tera）

Flagger の Canary CRD を定義する。対象 Deployment へのカナリアリリースの分析条件、ステップウェイト、メトリクス閾値を設定する。

```tera
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: {{ service_name }}
  namespace: {{ namespace }}
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ service_name }}
{% if tier == "system" %}
  progressDeadlineSeconds: 300
{% else %}
  progressDeadlineSeconds: 600
{% endif %}
  service:
    port: {{ server_port }}
    targetPort: {{ server_port }}
    gateways:
      - mesh
  analysis:
    interval: 30s
{% if tier == "system" %}
    threshold: 3
{% else %}
    threshold: 5
{% endif %}
    maxWeight: 50
    stepWeight: 10
    metrics:
      - name: request-success-rate
        templateRef:
          name: {{ service_name }}-success-rate
          namespace: {{ namespace }}
        thresholdRange:
{% if tier == "system" %}
          min: 99
{% else %}
          min: 95
{% endif %}
        interval: 30s
      - name: request-duration
        templateRef:
          name: {{ service_name }}-request-duration
          namespace: {{ namespace }}
        thresholdRange:
{% if tier == "system" %}
          max: 500
{% else %}
          max: 1000
{% endif %}
        interval: 30s
    webhooks:
      - name: rollback-alert
        type: rollback
        url: http://alertmanager.observability.svc.cluster.local:9093/api/v1/alerts
```

### ポイント

- `targetRef` で対象の Deployment を指定し、Flagger がカナリア用の Deployment を自動生成する
- system Tier は `progressDeadlineSeconds` を300秒（5分）に設定し、システム基盤のデプロイを迅速に完了させる
- business/service Tier は `progressDeadlineSeconds` を600秒（10分）に設定し、余裕のある分析期間を確保する
- `maxWeight: 50` により、カナリアへのトラフィックは最大50%まで段階的に増加する（10% -> 20% -> 30% -> 40% -> 50%）
- system Tier は `threshold: 3`（連続3回の分析失敗でロールバック）、business/service Tier は `threshold: 5` とする
- メトリクスは MetricTemplate を参照し、Prometheus からカスタムクエリで取得する
- ロールバック時は Alertmanager に通知を送信する

---

## MetricTemplate テンプレート（metric-template.yaml.tera）

Flagger の MetricTemplate を定義する。Prometheus からカスタムメトリクスを取得するためのクエリテンプレートを設定する。request-success-rate（成功率）と request-duration（レイテンシ）の2種類を生成する。

```tera
apiVersion: flagger.app/v1beta1
kind: MetricTemplate
metadata:
  name: {{ service_name }}-success-rate
  namespace: {{ namespace }}
spec:
  provider:
    type: prometheus
    address: http://prometheus.observability.svc.cluster.local:9090
  query: |
    sum(rate(
      http_requests_total{
        service="{{ service_name }}",
        namespace="{{ namespace }}",
        code!~"5.."
      }[{{ interval }}]
    ))
    /
    sum(rate(
      http_requests_total{
        service="{{ service_name }}",
        namespace="{{ namespace }}"
      }[{{ interval }}]
    )) * 100
---
apiVersion: flagger.app/v1beta1
kind: MetricTemplate
metadata:
  name: {{ service_name }}-request-duration
  namespace: {{ namespace }}
spec:
  provider:
    type: prometheus
    address: http://prometheus.observability.svc.cluster.local:9090
  query: |
    histogram_quantile(0.99,
      sum(rate(
        http_request_duration_seconds_bucket{
          service="{{ service_name }}",
          namespace="{{ namespace }}"
        }[{{ interval }}]
      )) by (le)
    ) * 1000
```

### ポイント

- **request-success-rate**: HTTP 5xx 以外のリクエスト割合を百分率で算出する。system Tier は99%以上、business/service Tier は95%以上を閾値とする
- **request-duration**: P99 レイテンシをミリ秒単位で算出する。system Tier は500ms以下、business/service Tier は1000ms以下を閾値とする
- Prometheus のアドレスは `http://prometheus.observability.svc.cluster.local:9090` を使用する（可観測性スタックの標準構成に準拠）
- `{{ interval }}` は Flagger が分析間隔に基づいて自動的に置換するビルトイン変数である

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件            | 選択肢                             | 生成への影響                                                |
| --------------- | ---------------------------------- | ----------------------------------------------------------- |
| Tier (`tier`)   | `system`                           | progressDeadline=300s、threshold=3、成功率99%、レイテンシ500ms |
| Tier (`tier`)   | `business` / `service`             | progressDeadline=600s、threshold=5、成功率95%、レイテンシ1000ms |
| kind (`kind`)   | `server` / `bff` 以外             | Flagger リソースを生成しない                                |

---

## 生成例

### system Tier のサーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "tier": "system",
  "namespace": "k1s0-system",
  "server_port": 80
}
```

生成されるファイル:
- `infra/flagger/auth-service/canary.yaml` -- progressDeadline=300s、threshold=3、成功率99%、レイテンシ500ms
- `infra/flagger/auth-service/metric-template.yaml` -- request-success-rate + request-duration の MetricTemplate 2種

### business Tier のサーバーの場合

入力:
```json
{
  "service_name": "accounting-api",
  "tier": "business",
  "namespace": "k1s0-business",
  "server_port": 80
}
```

生成されるファイル:
- `infra/flagger/accounting-api/canary.yaml` -- progressDeadline=600s、threshold=5、成功率95%、レイテンシ1000ms
- `infra/flagger/accounting-api/metric-template.yaml` -- request-success-rate + request-duration の MetricTemplate 2種

### service Tier の BFF の場合

入力:
```json
{
  "service_name": "order-bff",
  "tier": "service",
  "namespace": "k1s0-service",
  "server_port": 80
}
```

生成されるファイル:
- `infra/flagger/order-bff/canary.yaml` -- progressDeadline=600s、threshold=5、成功率95%、レイテンシ1000ms
- `infra/flagger/order-bff/metric-template.yaml` -- request-success-rate + request-duration の MetricTemplate 2種

---

## 関連ドキュメント

- [サービスメッシュ設計](サービスメッシュ設計.md) -- カナリアリリースの段階的ロールアウト設計・Flagger Canary リソース定義
- [CI-CD設計](CI-CD設計.md) -- CI/CD パイプラインとデプロイ戦略
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-ServiceMesh](テンプレート仕様-ServiceMesh.md) -- ServiceMesh テンプレート仕様（VirtualService・DestinationRule のカナリアサブセット）
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様（Prometheus メトリクス連携）
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) -- Helm テンプレート仕様
- [テンプレート仕様-CICD](テンプレート仕様-CICD.md) -- CI/CD テンプレート仕様
- [可観測性設計](可観測性設計.md) -- Prometheus メトリクス収集・アラート設計
