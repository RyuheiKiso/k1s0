# k1s0 Observability Stack

## 概要

k1s0 サービスの分散トレース・ログ・メトリクスを統合的に収集・可視化するための Observability スタックです。
OpenTelemetry Collector を中心に、Jaeger / Loki / Prometheus / Grafana を構成しています。

```
Services → OTEL Collector → Traces  → Jaeger      ─┐
                          → Logs    → Loki         ├→ Grafana
                          → Metrics → Prometheus   ─┘
```

## 前提条件

- **Docker Desktop** (または Docker Engine + Docker Compose) が必要です
- サービス開発自体 (ビルド・テスト・lint) に Docker は不要です。Observability スタックの起動時のみ使用します。

## クイックスタート

```bash
cd observability
docker compose up -d
```

## アクセス先

| サービス | URL | 用途 |
|---------|-----|------|
| Grafana | http://localhost:3000 | ダッシュボード (admin/admin) |
| Jaeger | http://localhost:16686 | トレース検索 |
| Prometheus | http://localhost:9090 | メトリクスクエリ |

## サービスからの接続

OTEL Collector は以下のエンドポイントでテレメトリデータを受け付けます。

| プロトコル | エンドポイント |
|-----------|---------------|
| gRPC (OTLP) | `localhost:4317` |
| HTTP (OTLP) | `localhost:4318` |

各言語の k1s0-observability ライブラリを使用すると、設定ファイルだけで接続が完了します。
言語別の設定例は `otel/instrumentation/` ディレクトリを参照してください。

**対応言語:** Rust, Go, C#, Python, Kotlin

```yaml
# config/default.yaml に追加する例
observability:
  exporter:
    protocol: grpc
    endpoint: "http://localhost:4317"
  traces:
    enabled: true
  metrics:
    enabled: true
  logs:
    enabled: true
```

## ダッシュボード一覧

| ダッシュボード | 説明 |
|---------------|------|
| k1s0 System Overview | サービス数・リクエスト率・エラー率・レイテンシの全体俯瞰 |
| HighErrorRate アラート | 5xx エラー率が 5% を超えた場合の通知 |
| HighLatency アラート | P99 レイテンシが 2 秒を超えた場合の通知 |
| ServiceDown アラート | サービスインスタンスのダウン検知 |
| OTEL Collector 自己監視 | Collector 自身のメトリクス (`:8888`) |

## 停止

```bash
# データを保持したまま停止
docker compose down

# ボリュームごと削除 (データ初期化)
docker compose down -v
```

## Kubernetes / Helm デプロイ

本番環境へのデプロイには `deploy/` ディレクトリの構成を使用します。

### Kustomize

```bash
# dev 環境へデプロイ
kubectl apply -k deploy/kubernetes/overlays/dev/

# prod 環境へデプロイ
kubectl apply -k deploy/kubernetes/overlays/prod/
```

`deploy/kubernetes/base/` に各コンポーネント（OTEL Collector, Jaeger, Loki, Prometheus, Grafana）のベースマニフェストがあり、`overlays/` で環境ごとにリソース制限やレプリカ数をオーバーライドします。

### Helm

```bash
# Helm values ファイルを使用したデプロイ
helm install observability <chart> -f deploy/helm/values/<env>.yaml
```

`deploy/helm/values/` に環境別の values ファイルが配置されています。

## 環境別設定

OTEL Collector の設定は環境ごとにオーバーライドされます (`otel/collector/config/`)。

| 環境 | サンプリング率 | メモリ制限 | ヘルスチェック除外 | ログレベル |
|------|--------------|-----------|-----------------|-----------|
| default | 全量 | 80% (割合) | なし | detailed |
| dev | 100% | 256 MiB | なし | debug |
| stg | default に準拠 | default に準拠 | default に準拠 | default に準拠 |
| prod | 10% | 1024 MiB | あり (`/health`, `/ready`, `/live`) | warn |

## ディレクトリ構造

```
observability/
├── grafana/
│   ├── dashboards/                # JSON ダッシュボード定義
│   │   └── overview.json
│   └── provisioning/
│       ├── alerting/
│       │   └── rules.yaml         # アラートルール
│       ├── dashboards/
│       │   └── provider.yaml      # ダッシュボードプロバイダ設定
│       └── datasources/
│           ├── jaeger.yaml
│           ├── loki.yaml
│           └── prometheus.yaml
├── jaeger/
│   └── config/
│       ├── default.yaml
│       ├── dev.yaml
│       └── prod.yaml
├── otel/
│   ├── collector/
│   │   ├── Dockerfile
│   │   └── config/
│   │       ├── default.yaml       # ベース設定
│   │       ├── dev.yaml
│   │       ├── stg.yaml
│   │       └── prod.yaml
│   └── instrumentation/           # 言語別の接続設定例
│       ├── csharp/
│       ├── go/
│       ├── kotlin/
│       ├── python/
│       └── rust/
└── deploy/
    ├── helm/
    │   └── values/
    └── kubernetes/
        ├── base/                  # Kustomize ベース
        │   ├── grafana/
        │   ├── jaeger/
        │   ├── loki/
        │   ├── otel-collector/
        │   └── prometheus/
        └── overlays/              # 環境別オーバーレイ
            ├── dev/
            ├── stg/
            └── prod/
```
