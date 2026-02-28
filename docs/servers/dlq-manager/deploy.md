# system-dlq-manager-server デプロイ設計

> **ガイド**: 設計背景・実装例は [deploy.guide.md](./deploy.guide.md) を参照。

system-dlq-manager-server の Dockerfile・テスト・CI/CD パイプライン・設定ファイル・Helm values を定義する。概要・API 定義・アーキテクチャは [system-dlq-manager-server.md](server.md) を参照。

---

## Dockerfile

[Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) のテンプレートに従う。ビルドコンテキストは `regions/system`（ライブラリ依存解決のため）。

### Dockerfile 構成のポイント

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.88-bookworm`（マルチステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-dlq-manager`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API のみ、gRPC なし） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

> Dockerfile 全文は [deploy.guide.md](./deploy.guide.md#dockerfile) を参照。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/entity | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP） | `axum::test` + `tokio::test` |
| infrastructure/config | 単体テスト | `#[cfg(test)]` |
| infrastructure/database | 単体テスト | `#[cfg(test)]` |
| infrastructure/kafka | 単体テスト（モック） | `mockall` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |

### ユニットテスト（48 テスト）

| テスト対象 | テスト数 | 内容 |
| --- | --- | --- |
| domain/entity/dlq_message | 13 | ステータス遷移、リトライ判定、Display/from_str、UUID 生成 |
| infrastructure/config | 4 | 設定デシリアライズ、デフォルト値、DB/Kafka 設定 |
| infrastructure/database | 2 | 接続 URL 生成、設定デシリアライズ |
| infrastructure/kafka/mod | 2 | KafkaConfig デシリアライズ、デフォルト値 |
| infrastructure/kafka/producer | 2 | MockDlqEventPublisher 正常/エラー |
| usecase/list_messages | 4 | 空一覧、結果あり、ページネーション、リポジトリエラー |
| usecase/get_message | 2 | 取得成功、未存在 |
| usecase/retry_message | 4 | 正常リトライ、未存在、リトライ不可、上限超過 |
| usecase/delete_message | 2 | 正常削除、エラー |
| usecase/retry_all | 3 | 空トピック、メッセージあり、非リトライ対象スキップ |
| adapter/handler/dlq_handler | 10 | healthz/readyz、一覧、詳細取得（成功/404/400）、削除、リトライ、一括リトライ |

### 統合テスト

`tests/integration_test.rs` に配置。InMemory リポジトリを使用した REST API のエンドツーエンド動作を検証する（12 テストケース）。

| テストケース | 内容 |
| --- | --- |
| `test_healthz_returns_ok` | ヘルスチェック正常 |
| `test_readyz_returns_ok` | レディネスチェック正常 |
| `test_list_messages_empty_topic` | 空トピックの一覧取得 |
| `test_list_messages_returns_stored_message` | メッセージありの一覧取得 |
| `test_get_message_returns_404_when_not_found` | 未存在メッセージ取得時 404 |
| `test_get_message_returns_message` | メッセージ取得成功 |
| `test_get_message_returns_400_for_invalid_id` | 不正 UUID 時 400 |
| `test_retry_message_returns_404_when_not_found` | 未存在メッセージリトライ時 404 |
| `test_retry_message_resolves_pending_message` | PENDING メッセージのリトライ成功 |
| `test_delete_message_returns_ok` | メッセージ削除成功 |
| `test_retry_all_returns_retried_count` | 一括リトライの件数返却 |

---

## CI/CD パイプライン

### CI（`.github/workflows/dlq-manager-ci.yaml`）

PR 時に `regions/system/server/rust/dlq-manager/**` の変更を検出してトリガーする。

```
detect-changes → lint-rust → test-rust → build-rust → security-scan
```

| ジョブ | 内容 |
| --- | --- |
| `detect-changes` | `dorny/paths-filter@v3` でパス変更検出 |
| `lint-rust` | `cargo fmt --check` + `cargo clippy -D warnings` |
| `test-rust` | `cargo test --all` |
| `build-rust` | `cargo build --release` |
| `security-scan` | Trivy ファイルシステムスキャン（HIGH/CRITICAL） |

**Concurrency**: `ci-${{ github.ref }}-dlq-manager`（同一ブランチの古い実行をキャンセル）

### CD（`.github/workflows/dlq-manager-deploy.yaml`）

main ブランチへの push 時にトリガー。

```
build-and-push → deploy-dev → deploy-staging → deploy-prod（手動承認）
```

| ジョブ | 内容 |
| --- | --- |
| `build-and-push` | Docker Buildx ビルド、Harbor プッシュ、Cosign 署名、Trivy イメージスキャン、SBOM 生成 |
| `deploy-dev` | Cosign 署名検証 → Helm upgrade（dev 環境） |
| `deploy-staging` | Cosign 署名検証 → Helm upgrade（staging 環境） |
| `deploy-prod` | Cosign 署名検証 → Helm upgrade（prod 環境、手動承認必須） |

### イメージタグ戦略

| タグ | 形式 |
| --- | --- |
| バージョン | `{VERSION}`（git describe から取得） |
| バージョン + SHA | `{VERSION}-{SHORT_SHA}` |
| latest | `latest` |

**レジストリ**: `harbor.internal.example.com/k1s0-system/dlq-manager`

---

## ポートマッピング

| 環境 | ホストポート | コンテナポート | プロトコル |
| --- | --- | --- | --- |
| docker-compose | 8086 | 8080 | REST API |
| Kubernetes | 80（Service） | 8080（Pod） | REST API |

DLQ Manager は REST API のみを提供し、gRPC エンドポイントは持たない。

### エンドポイント

| パス | 用途 |
| --- | --- |
| `/api/v1/dlq/*` | DLQ メッセージ管理 REST API |
| `/healthz` | ヘルスチェック（Liveness Probe） |
| `/readyz` | レディネスチェック（Readiness Probe） |
| `/metrics` | Prometheus メトリクス |

---

## 依存サービス

| サービス | 用途 | 必須 |
| --- | --- | --- |
| PostgreSQL | DLQ メッセージの永続化 | No（未設定時は InMemory リポジトリで動作） |
| Kafka | DLQ トピック購読・元トピックへの再発行 | No（未設定時は REST API のみ動作、再処理時は再発行をスキップ） |

### 本番設定

| 項目 | 値 |
| --- | --- |
| `server.port` | 8080 |
| `database.host` | `postgres.k1s0-system.svc.cluster.local` |
| `database.name` | `k1s0_dlq` |
| `database.ssl_mode` | `disable` |
| `kafka.brokers` | `kafka-0.messaging.svc.cluster.local:9092` |
| `kafka.consumer_group` | `dlq-manager.default` |
| `kafka.security_protocol` | `PLAINTEXT` |

---

## Helm Chart

Helm Chart は `infra/helm/services/system/dlq-manager/` に配置。k1s0-common チャート（v0.1.0）に依存。

### Chart 構成

```
infra/helm/services/system/dlq-manager/
  Chart.yaml
  Chart.lock
  values.yaml           # デフォルト values
  values-dev.yaml       # dev 環境オーバーライド
  values-staging.yaml   # staging 環境オーバーライド
  values-prod.yaml      # prod 環境オーバーライド
  charts/
    k1s0-common-0.1.0.tgz
  templates/
    _helpers.tpl
    configmap.yaml
    deployment.yaml
    hpa.yaml
    pdb.yaml
    service.yaml
    serviceaccount.yaml
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/dlq-manager/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

> Helm values・設定ファイル・Kubernetes Probes・Kong ルーティングの例は [deploy.guide.md](./deploy.guide.md) を参照。

---

## 関連ドキュメント

- [system-dlq-manager-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-dlq-manager-server-implementation.md](implementation.md) -- Rust 実装詳細
- [system-library-dlq-client.md](../../libraries/messaging/dlq-client.md) -- DLQ クライアントライブラリ設計
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- DLQ パターンの基本方針
