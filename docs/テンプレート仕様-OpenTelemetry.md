# テンプレート仕様 — OpenTelemetry

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **OpenTelemetry Collector** 設定テンプレートの仕様を定義する。Kubernetes ConfigMap として OpenTelemetry Collector の設定ファイルを生成し、分散トレーシングのレシーバー、プロセッサー、エクスポーターをサービスの `tier` に応じて自動構成する。

可観測性設計の全体像は [可観測性設計](可観測性設計.md) の D-110（分散トレーシング設計）を参照。

## 生成対象

| kind       | Collector ConfigMap |
| ---------- | ------------------- |
| `server`   | 生成する            |
| `bff`      | 生成する            |
| `client`   | 生成しない          |
| `library`  | 生成しない          |
| `database` | 生成しない          |

## 配置パス

生成されるリソースファイルは `infra/otel/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル              | 配置パス                                                    |
| --------------------- | ----------------------------------------------------------- |
| Collector ConfigMap   | `infra/otel/{{ service_name }}/collector-config.yaml`       |

## テンプレートファイル一覧

テンプレートは `CLI/crates/k1s0-cli/templates/opentelemetry/` 配下に配置する。

| テンプレートファイル              | 生成先                                                    | 説明                                    |
| --------------------------------- | --------------------------------------------------------- | --------------------------------------- |
| `collector-config.yaml.tera`      | `infra/otel/{{ service_name }}/collector-config.yaml`     | OpenTelemetry Collector ConfigMap 定義  |

### ディレクトリ構成

```
CLI/
└── templates/
    └── otel/
        └── collector-config.yaml.tera
```

## 使用するテンプレート変数

OpenTelemetry テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | Collector ConfigMap | 用途                                     |
| -------------------- | ------ | ------------------- | ---------------------------------------- |
| `service_name`       | String | 用                  | リソース名、resource attributes          |
| `service_name_snake` | String | 用                  | ConfigMap 名のプレフィクス               |
| `namespace`          | String | 用                  | リソースの配置先 Namespace               |
| `tier`               | String | 用                  | サンプリングレート、バッチサイズの決定   |

### Tier 別サンプリングレート

| Tier       | サンプリングレート | 説明                                     |
| ---------- | ------------------ | ---------------------------------------- |
| `system`   | 1.0（100%）        | システム基盤は全トレースを収集           |
| `business` | 0.5（50%）         | ビジネスロジックは半数をサンプリング     |
| `service`  | 0.1（10%）         | 業務サービスは10%をサンプリング          |

### Tier 別バッチ設定

| Tier       | batch timeout | batch send_batch_size | memory_limiter limit_mib |
| ---------- | ------------- | --------------------- | ------------------------ |
| `system`   | 5s            | 512                   | 512                      |
| `business` | 10s           | 256                   | 256                      |
| `service`  | 10s           | 128                   | 128                      |

---

## Collector ConfigMap テンプレート（collector-config.yaml.tera）

OpenTelemetry Collector の設定を ConfigMap として定義する。receivers / processors / exporters / service の4セクションで構成する。

```tera
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ service_name_snake }}-otel-collector
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
data:
  collector.yaml: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
          http:
            endpoint: 0.0.0.0:4318

    processors:
      batch:
{% if tier == "system" %}
        timeout: 5s
        send_batch_size: 512
{% elif tier == "business" %}
        timeout: 10s
        send_batch_size: 256
{% elif tier == "service" %}
        timeout: 10s
        send_batch_size: 128
{% endif %}

      memory_limiter:
        check_interval: 5s
{% if tier == "system" %}
        limit_mib: 512
        spike_limit_mib: 128
{% elif tier == "business" %}
        limit_mib: 256
        spike_limit_mib: 64
{% elif tier == "service" %}
        limit_mib: 128
        spike_limit_mib: 32
{% endif %}

      resource:
        attributes:
          - key: service.name
            value: {{ service_name }}
            action: upsert
          - key: service.namespace
            value: {{ namespace }}
            action: upsert
          - key: deployment.tier
            value: {{ tier }}
            action: upsert

      probabilistic_sampler:
{% if tier == "system" %}
        sampling_percentage: 100
{% elif tier == "business" %}
        sampling_percentage: 50
{% elif tier == "service" %}
        sampling_percentage: 10
{% endif %}

    exporters:
      jaeger:
        endpoint: jaeger-collector.observability.svc.cluster.local:14250
        tls:
          insecure: true

      prometheus:
        endpoint: 0.0.0.0:8889
        namespace: {{ service_name_snake }}

    service:
      pipelines:
        traces:
          receivers: [otlp]
          processors: [memory_limiter, probabilistic_sampler, batch, resource]
          exporters: [jaeger]
        metrics:
          receivers: [otlp]
          processors: [memory_limiter, batch, resource]
          exporters: [prometheus]
```

### ポイント

- **receivers**: OTLP プロトコルで gRPC（4317）と HTTP（4318）の両方を受信する。アプリケーションは OTLP エクスポーターでトレース・メトリクスを送信する
- **processors**:
  - `memory_limiter`: メモリ使用量を制限し、Collector の OOM を防止する。Tier に応じて上限を変更する
  - `probabilistic_sampler`: Tier に応じたサンプリングレートでトレースを間引く。system Tier は全トレース、service Tier は10%に絞る
  - `batch`: バッチ処理でエクスポーターへの送信を効率化する。system Tier はタイムアウトを短く設定する
  - `resource`: サービス名・Namespace・Tier をリソース属性として付与する
- **exporters**:
  - `jaeger`: トレースを Jaeger Collector に送信する。observability Namespace にデプロイされた Jaeger を使用する
  - `prometheus`: メトリクスを Prometheus 形式でエクスポートする（ポート8889）
- **pipeline 順序**: `memory_limiter` → `probabilistic_sampler` → `batch` → `resource` の順でプロセッサーを適用する。メモリ制限を最初に行い、サンプリングで間引いてからバッチ処理する

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                | 選択肢                              | 生成への影響                                                  |
| ------------------- | ----------------------------------- | ------------------------------------------------------------- |
| Tier (`tier`)       | `system`                            | サンプリング100%、バッチ5s/512、メモリ512MiB                 |
| Tier (`tier`)       | `business`                          | サンプリング50%、バッチ10s/256、メモリ256MiB                 |
| Tier (`tier`)       | `service`                           | サンプリング10%、バッチ10s/128、メモリ128MiB                 |
| kind (`kind`)       | `server` / `bff` 以外              | OpenTelemetry Collector 設定を生成しない                      |

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
- `infra/otel/auth-service/collector-config.yaml` -- サンプリング100%、バッチ5s/512、メモリ上限512MiB、Jaeger + Prometheus エクスポート

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
- `infra/otel/inventory-server/collector-config.yaml` -- サンプリング50%、バッチ10s/256、メモリ上限256MiB

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
- `infra/otel/order-bff/collector-config.yaml` -- サンプリング10%、バッチ10s/128、メモリ上限128MiB

---

## 関連ドキュメント

- [可観測性設計](可観測性設計.md) -- 可観測性の全体設計（D-110 分散トレーシング設計）
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様（ServiceMonitor・PrometheusRule）
- [テンプレート仕様-Grafana](テンプレート仕様-Grafana.md) -- Grafana ダッシュボードテンプレート仕様
- [テンプレート仕様-Loki](テンプレート仕様-Loki.md) -- Loki/Promtail テンプレート仕様
- [テンプレート仕様-Config](テンプレート仕様-Config.md) -- Config テンプレート仕様（トレース設定連携）
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) -- サーバーテンプレート（OTel 初期化コード）
