# テンプレート仕様 — Alertmanager

## 概要

k1s0 CLI ひな形生成の Alertmanager テンプレート仕様。Kubernetes ConfigMap として Alertmanager の設定ファイルを生成し、severity 別のルーティング、Microsoft Teams への Webhook 通知、環境別のアラート抑制をサービスの `tier` に応じて自動構成する。Webhook URL は Kubernetes Secret（Vault 管理）から参照する。

可観測性設計の全体像は [可観測性設計](../../architecture/observability/可観測性設計.md) の Alertmanager → Teams 通知設計を参照。

## 生成対象

| kind       | Alertmanager ConfigMap |
| ---------- | ---------------------- |
| `server`   | 生成する               |
| `bff`      | 生成する               |

## 配置パス

生成されるリソースファイルは `infra/alertmanager/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル                    | 配置パス                                                              |
| --------------------------- | --------------------------------------------------------------------- |
| Alertmanager ConfigMap      | `infra/alertmanager/{{ service_name }}/alertmanager-config.yaml`      |

## テンプレートファイル一覧

テンプレートは `CLI/templates/alertmanager/` 配下に配置する。

| テンプレートファイル                   | 生成先                                                              | 説明                                  |
| -------------------------------------- | ------------------------------------------------------------------- | ------------------------------------- |
| `alertmanager-config.yaml.tera`        | `infra/alertmanager/{{ service_name }}/alertmanager-config.yaml`    | Alertmanager ConfigMap 定義           |

### ディレクトリ構成

```
CLI/
└── templates/
    └── alertmanager/
        └── alertmanager-config.yaml.tera
```

## 使用するテンプレート変数

Alertmanager テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | Alertmanager ConfigMap | 用途                                     |
| -------------------- | ------ | ---------------------- | ---------------------------------------- |
| `service_name`       | String | 用                     | リソース名、ルーティングのマッチラベル   |
| `service_name_snake` | String | 用                     | ConfigMap 名のプレフィクス               |
| `namespace`          | String | 用                     | リソースの配置先 Namespace               |
| `tier`               | String | 用                     | ルーティング設定、repeat_interval の決定 |

### 環境別アラート抑制

| 環境    | critical 通知 | warning 通知 | 備考                       |
| ------- | ------------- | ------------ | -------------------------- |
| dev     | 無効          | 無効         | 開発者がローカルで確認     |
| staging | 有効          | 無効         | 重大な問題のみ通知         |
| prod    | 有効          | 有効         | 全アラートを通知           |

### Tier 別ルーティング設定

| Tier       | critical repeat_interval | warning repeat_interval | group_wait |
| ---------- | ------------------------ | ----------------------- | ---------- |
| `system`   | 30m                      | 2h                      | 15s        |
| `business` | 1h                       | 4h                      | 30s        |
| `service`  | 1h                       | 4h                      | 30s        |

---

## Alertmanager ConfigMap テンプレート（alertmanager-config.yaml.tera）

Alertmanager の設定を ConfigMap として定義する。severity 別のルーティング、receivers（prometheus-msteams Webhook）、環境別の抑制ルールを構成する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-alertmanager-config
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
data:
  alertmanager.yaml: |
    global:
      resolve_timeout: 5m

    route:
      group_by: ['alertname', 'namespace', 'service']
{% if tier == "system" %}
      group_wait: 15s
{% else %}
      group_wait: 30s
{% endif %}
      group_interval: 5m
      repeat_interval: 4h
      receiver: teams-default
      routes:
        # critical アラート → teams-critical チャネル
        - match:
            severity: critical
            service: {{ service_name }}
          receiver: teams-critical
{% if tier == "system" %}
          repeat_interval: 30m
{% else %}
          repeat_interval: 1h
{% endif %}
          continue: false

        # warning アラート → teams-warning チャネル
        - match:
            severity: warning
            service: {{ service_name }}
          receiver: teams-warning
{% if tier == "system" %}
          repeat_interval: 2h
{% else %}
          repeat_interval: 4h
{% endif %}
          continue: false

    receivers:
      - name: teams-default
        webhook_configs:
          - url: 'http://prometheus-msteams.observability.svc.cluster.local:2000/default'
            send_resolved: true

      - name: teams-critical
        webhook_configs:
          - url: 'http://prometheus-msteams.observability.svc.cluster.local:2000/critical'
            send_resolved: true

      - name: teams-warning
        webhook_configs:
          - url: 'http://prometheus-msteams.observability.svc.cluster.local:2000/warning'
            send_resolved: true

      # dev/staging 環境での抑制用 null receiver
      - name: 'null'

    # 環境別アラート抑制ルール
    inhibit_rules:
      # dev 環境: 全アラートを抑制
      - source_match:
          environment: dev
        target_match_re:
          severity: critical|warning
        equal: ['namespace']

      # staging 環境: warning を抑制（critical のみ通知）
      - source_match:
          environment: staging
        target_match:
          severity: warning
        equal: ['namespace']
```

### ポイント

- **route**:
  - `group_by` で `alertname`、`namespace`、`service` をグルーピングし、同一アラートの重複通知を防止する
  - system Tier は `group_wait: 15s` で即座に通知を開始し、障害検知の迅速化を図る
  - `continue: false` で最初にマッチしたルートで処理を終了する
- **receivers**:
  - `prometheus-msteams` は observability Namespace にデプロイされ、Alertmanager の Webhook を受けて Microsoft Teams チャネルに通知する
  - Webhook URL は prometheus-msteams が Kubernetes Secret `prometheus-msteams-webhook` から環境変数経由で参照する（テンプレート内にはハードコードしない）
  - `send_resolved: true` でアラート解消時にも通知する
- **inhibit_rules**:
  - dev 環境では全アラート（critical/warning）を抑制し、開発者がローカルで確認する運用とする
  - staging 環境では warning を抑制し、critical のみ通知する
  - prod 環境では抑制ルールが適用されず、全アラートが通知される
- **Webhook URL 管理**: Webhook URL は Kubernetes Secret に格納し、Vault で管理する。テンプレートは prometheus-msteams の Service エンドポイントのみを参照する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                | 選択肢                              | 生成への影響                                                  |
| ------------------- | ----------------------------------- | ------------------------------------------------------------- |
| Tier (`tier`)       | `system`                            | group_wait=15s、critical repeat=30m、warning repeat=2h       |
| Tier (`tier`)       | `business` / `service`              | group_wait=30s、critical repeat=1h、warning repeat=4h        |
| kind (`kind`)       | `server` / `bff` 以外              | Alertmanager 設定を生成しない                                 |

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
- `infra/alertmanager/auth-service/alertmanager-config.yaml` -- group_wait=15s、critical repeat=30m、warning repeat=2h、dev/staging 抑制ルール

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
- `infra/alertmanager/inventory-server/alertmanager-config.yaml` -- group_wait=30s、critical repeat=1h、warning repeat=4h、dev/staging 抑制ルール

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
- `infra/alertmanager/order-bff/alertmanager-config.yaml` -- group_wait=30s、critical repeat=1h、warning repeat=4h、dev/staging 抑制ルール

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [可観測性設計](../../architecture/observability/可観測性設計.md) -- 可観測性の全体設計（Alertmanager → Teams 通知設計）
- [テンプレート仕様-Observability](Observability.md) -- Observability テンプレート仕様（PrometheusRule でアラートを定義）
- [テンプレート仕様-Grafana](Grafana.md) -- Grafana ダッシュボードテンプレート仕様
- [テンプレート仕様-OpenTelemetry](OpenTelemetry.md) -- OpenTelemetry Collector テンプレート仕様
- [テンプレート仕様-Loki](Loki.md) -- Loki/Promtail テンプレート仕様
- [テンプレート仕様-Config](../data/Config.md) -- Config テンプレート仕様
