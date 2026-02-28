# テンプレート仕様 — Loki

## 概要

k1s0 CLI ひな形生成の Promtail/Loki テンプレート仕様。Kubernetes ConfigMap として Promtail の設定ファイルを生成し、構造化ログの収集・ラベル付け・パイプライン処理をサービスの `tier` に応じて自動構成する。JSON 構造化ログから `level`、`trace_id`、`service` を抽出し、Loki でクエリ可能にする。

可観測性設計の全体像は [可観測性設計](../../architecture/observability/可観測性設計.md) の D-109（構造化ログ設計）を参照。

## 生成対象

| kind       | Promtail ConfigMap |
| ---------- | ------------------- |
| `server`   | 生成する            |
| `bff`      | 生成する            |

## 配置パス

生成されるリソースファイルは `infra/loki/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル              | 配置パス                                                  |
| --------------------- | --------------------------------------------------------- |
| Promtail ConfigMap    | `infra/loki/{{ service_name }}/promtail-config.yaml`      |

## テンプレートファイル一覧

テンプレートは `CLI/templates/loki/` 配下に配置する。

| テンプレートファイル              | 生成先                                                  | 説明                               |
| --------------------------------- | ------------------------------------------------------- | ---------------------------------- |
| `promtail-config.yaml.tera`      | `infra/loki/{{ service_name }}/promtail-config.yaml`    | Promtail ConfigMap 定義            |

### ディレクトリ構成

```
CLI/
└── templates/
    └── loki/
        └── promtail-config.yaml.tera
```

## 使用するテンプレート変数

Loki テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | Promtail ConfigMap | 用途                                     |
| -------------------- | ------ | ------------------- | ---------------------------------------- |
| `service_name`       | String | 用                  | リソース名、scrape_configs のフィルタ    |
| `service_name_snake` | String | 用                  | ConfigMap 名のプレフィクス               |
| `namespace`          | String | 用                  | リソースの配置先 Namespace、ラベル       |
| `tier`               | String | 用                  | ラベル付与、ログ保持期間の決定           |

### Tier 別ログ保持期間

| Tier       | dev 環境 | staging 環境 | prod 環境 |
| ---------- | -------- | ------------ | --------- |
| `system`   | 7日      | 30日         | 90日      |
| `business` | 7日      | 30日         | 90日      |
| `service`  | 7日      | 30日         | 90日      |

> ログ保持期間は Tier ではなく環境に依存する。Loki の `retention_period` で制御する（dev=168h、staging=720h、prod=2160h）。

### 環境別ログレベル

| 環境    | デフォルトレベル | フォーマット |
| ------- | ---------------- | ------------ |
| dev     | debug            | text         |
| staging | info             | json         |
| prod    | warn             | json         |

---

## Promtail ConfigMap テンプレート（promtail-config.yaml.tera）

Promtail の設定を ConfigMap として定義する。Kubernetes Pod のログを収集し、JSON パース・ラベル付与・フィルタリングを行う。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-promtail-config
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
data:
  promtail.yaml: |
    server:
      http_listen_port: 3101

    positions:
      filename: /tmp/positions.yaml

    clients:
      - url: http://loki.observability.svc.cluster.local:3100/loki/api/v1/push

    scrape_configs:
      - job_name: {{ service_name }}-pods
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - {{ namespace }}

        relabel_configs:
          # Namespace ラベル
          - source_labels: [__meta_kubernetes_namespace]
            target_label: namespace

          # アプリケーション名ラベル
          - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_name]
            target_label: app

          # Tier ラベル
          - source_labels: [__meta_kubernetes_pod_label_tier]
            target_label: tier

          # Pod 名ラベル
          - source_labels: [__meta_kubernetes_pod_name]
            target_label: pod

          # 対象サービスのみ収集
          - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_name]
            regex: {{ service_name }}
            action: keep

        pipeline_stages:
          # JSON 構造化ログのパース
          - json:
              expressions:
                level: level
                trace_id: trace_id
                span_id: span_id
                service: service
                request_id: request_id
                method: method
                path: path
                status: status
                duration_ms: duration_ms

          # 抽出フィールドをラベルに変換
          - labels:
              level:
              trace_id:
              service:

          # タイムスタンプのパース
          - timestamp:
              source: timestamp
              format: RFC3339Nano

          # ログレベルフィルタリング
{% if tier == "system" %}
          # system Tier: debug 以上を全て収集
          - match:
              selector: '{app="{{ service_name }}"}'
              stages: []
{% elif tier == "business" %}
          # business Tier: info レベル未満をドロップ
          - match:
              selector: '{app="{{ service_name }}", level="debug"}'
              action: drop
{% elif tier == "service" %}
          # service Tier: info レベル未満をドロップ
          - match:
              selector: '{app="{{ service_name }}", level="debug"}'
              action: drop
{% endif %}

          # 出力フォーマット
          - output:
              source: message
```

### ポイント

- **scrape_configs**: `kubernetes_sd_configs` で Pod を自動検出し、`relabel_configs` で対象サービスのみにフィルタリングする
- **relabel_configs**:
  - `namespace`、`app`、`tier`、`pod` の4つのラベルを付与し、Loki でのクエリ（`{namespace="k1s0-system", app="auth-service"}`）を容易にする
  - `action: keep` で対象サービスの Pod のみを収集対象とする
- **pipeline_stages**:
  - `json` ステージで構造化ログの主要フィールド（level, trace_id, span_id, service, request_id 等）を抽出する
  - `labels` ステージで `level`、`trace_id`、`service` をインデックスラベルに変換する（Loki のクエリパフォーマンスに寄与）
  - `timestamp` ステージでログのタイムスタンプを RFC3339Nano 形式でパースする
  - `match` ステージで Tier に応じたログレベルフィルタリングを行う。system Tier は全レベル収集、business/service Tier は debug をドロップする
- **Loki への送信先**: `observability` Namespace にデプロイされた Loki の push API エンドポイントを使用する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                | 選択肢                              | 生成への影響                                                  |
| ------------------- | ----------------------------------- | ------------------------------------------------------------- |
| Tier (`tier`)       | `system`                            | 全ログレベルを収集（debug 含む）                             |
| Tier (`tier`)       | `business` / `service`              | debug レベルをドロップ                                       |
| kind (`kind`)       | `server` / `bff` 以外              | Promtail 設定を生成しない                                     |

---

## 生成例

### system Tier のサーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "service_name_snake": "auth_service",
  "tier": "system",
  "namespace": "k1s0-system"
}
```

生成されるファイル:
- `infra/loki/auth-service/promtail-config.yaml` -- debug 以上の全ログレベルを収集、namespace/app/tier/pod ラベル付与、JSON パイプライン

### business Tier のサーバーの場合

入力:
```json
{
  "service_name": "inventory-server",
  "service_name_snake": "inventory_server",
  "tier": "business",
  "namespace": "k1s0-business"
}
```

生成されるファイル:
- `infra/loki/inventory-server/promtail-config.yaml` -- debug ドロップ（info 以上を収集）、namespace/app/tier/pod ラベル付与

### service Tier の BFF の場合

入力:
```json
{
  "service_name": "order-bff",
  "service_name_snake": "order_bff",
  "tier": "service",
  "namespace": "k1s0-service"
}
```

生成されるファイル:
- `infra/loki/order-bff/promtail-config.yaml` -- debug ドロップ（info 以上を収集）、namespace/app/tier/pod ラベル付与

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [可観測性設計](../../architecture/observability/可観測性設計.md) -- 可観測性の全体設計（D-109 構造化ログ設計）
- [テンプレート仕様-Observability](Observability.md) -- Observability テンプレート仕様（ServiceMonitor・PrometheusRule）
- [テンプレート仕様-OpenTelemetry](OpenTelemetry.md) -- OpenTelemetry Collector テンプレート仕様（トレース相関）
- [テンプレート仕様-Grafana](Grafana.md) -- Grafana ダッシュボードテンプレート仕様
- [テンプレート仕様-Alertmanager](Alertmanager.md) -- Alertmanager テンプレート仕様
- [テンプレート仕様-Kong](../middleware/Kong.md) -- Kong テンプレート仕様（ロギングプラグイン連携）
- [テンプレート仕様-サーバー](../server/サーバー.md) -- サーバーテンプレート（構造化ログ実装）
