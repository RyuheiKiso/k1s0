# system-config-server デプロイ設計

system-config-server のキャッシュ戦略・テスト方針・Dockerfile・Helm values を定義する。概要・API 定義・アーキテクチャは [system-config-server.md](server.md) を参照。

> **ガイド**: 設計背景・実装例は [deploy.guide.md](./deploy.guide.md) を参照。

---

## キャッシュ戦略

設定値の取得は高頻度で呼び出されるため、インメモリキャッシュによるレイテンシ削減を行う。

### キャッシュ方針

| 項目 | 値 |
| --- | --- |
| キャッシュ方式 | インメモリ（Go: ristretto, Rust: moka） |
| TTL | 設定可能（デフォルト 60 秒） |
| 最大エントリ数 | 設定可能（デフォルト 10,000） |
| キャッシュキー | `{namespace}:{key}` 形式 |
| 無効化タイミング | PUT / DELETE 実行時に即座に無効化 |
| キャッシュミス | DB から取得後にキャッシュに格納 |

---

## データベースマイグレーション

設定値と変更ログの2テーブルを PostgreSQL（config-db）に格納する。詳細なスキーマ定義と全マイグレーションファイルは [system-config-database.md](database.md) 参照。

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | ツール |
| --- | --- | --- |
| domain/service | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `axum::test` + `tokio::test` |
| infrastructure/persistence | 統合テスト（DB） | `testcontainers` |
| infrastructure/cache | 単体テスト | `tokio::test` |

---

## デプロイ

### Dockerfile 構成

| 項目 | 詳細 |
| --- | --- |
| ビルドステージ | `rust:1.88-bookworm`（マルチステージビルド） |
| ランタイムステージ | `gcr.io/distroless/cc-debian12:nonroot`（最小イメージ） |
| 追加パッケージ | `protobuf-compiler`（proto 生成）、`cmake` + `build-essential`（rdkafka ビルド） |
| libz コピー | distroless には zlib が含まれないため、ビルドステージから手動コピー |
| ビルドコマンド | `cargo build --release -p k1s0-config-server`（ワークスペースから特定パッケージを指定） |
| ビルドコンテキスト | `regions/system`（`COPY . .` でシステム全体のライブラリ依存を含める） |
| 公開ポート | 8080（REST API）、50051（gRPC） |
| 実行ユーザー | `nonroot:nonroot`（セキュリティベストプラクティス） |

---

## 関連ドキュメント

- [system-config-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-config-server-implementation.md](implementation.md) -- Rust 実装詳細
- [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](../../infrastructure/kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- Kong ルーティング設計
