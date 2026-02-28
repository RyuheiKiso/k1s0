# system-graphql-gateway デプロイ設計

> **ガイド**: Dockerfile・Helm values 詳細・CI/CD パイプライン・Kong ルーティングは [deploy.guide.md](./deploy.guide.md) を参照。

概要・API 定義・アーキテクチャは [system-graphql-gateway設計.md](server.md) を参照。

---

## 環境変数一覧

| 環境変数 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `CONFIG_PATH` | - | `config/config.yaml` | 設定ファイルのパス |
| `ENVIRONMENT` | - | `production` | 実行環境（`development` / `staging` / `production`） |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | `http://localhost:4317` | OpenTelemetry Collector エンドポイント |
| `RUST_LOG` | - | `info` | tracing フィルタ（例: `info,k1s0_graphql_gateway_server=debug`） |
| `SERVER_PORT` | - | `8080` | HTTP リスニングポート |

Vault Agent Injector によって `/vault/secrets/` に書き出されたシークレットは、設定ファイル内のパスで参照する。graphql-gateway 自体はシークレットを直接保持しない。

---

## Docker イメージ

| 項目 | 値 |
| --- | --- |
| ビルドステージ | `rust:1.88-bookworm` |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot` |
| ビルドコンテキスト | `regions/system` |
| ビルドコマンド | `cargo build --release -p k1s0-graphql-gateway` |
| 公開ポート | 8080 |
| 実行ユーザー | `nonroot:nonroot` |

---

## リソース制限

| 環境 | CPU requests | CPU limits | Memory requests | Memory limits |
| --- | --- | --- | --- | --- |
| 本番 | 100m | 500m | 128Mi | 256Mi |
| ステージング | 50m | 200m | 64Mi | 128Mi |

---

## HPA 設定

| パラメータ | 値 |
| --- | --- |
| minReplicas | 2 |
| maxReplicas | 10 |
| targetCPUUtilizationPercentage | 70 |

---

## ヘルスチェック設定

| Probe | Path | initialDelay | period | timeout | failureThreshold |
| --- | --- | --- | --- | --- | --- |
| Liveness | `/healthz` | 10s | 10s | 5s | 3 |
| Readiness | `/readyz` | 5s | 5s | 3s | 3 |

---

## 関連ドキュメント

- [system-graphql-gateway設計.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-graphql-gateway-implementation.md](implementation.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [API設計.md](../../architecture/api/API設計.md) -- Kong ルーティング設計
