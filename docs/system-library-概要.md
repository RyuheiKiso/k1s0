# system-library 概要

system tier が提供する共通ライブラリの設計を定義する。
全ライブラリは Go / Rust / TypeScript / Dart / C# / Swift の 6 言語で平等に実装する。
Swift 実装は Swift Package Manager（SPM）を用い、iOS / macOS ネイティブクライアントおよび Swift サーバーサイド（Vapor 等）向けの共有ライブラリとして提供する。Swift 6 Concurrency（actor / Sendable）に準拠し、型安全な非同期 API を設計する。

## ライブラリ一覧

| ライブラリ | 用途 | 利用者 | 詳細設計 |
|-----------|------|--------|---------|
| config | YAML 設定読み込み・環境別オーバーライド・バリデーション | 全サーバー・クライアント | [system-library-config設計](system-library-config設計.md) |
| telemetry | OpenTelemetry 初期化・構造化ログ・分散トレース・メトリクス | 全サーバー・クライアント | [system-library-telemetry設計](system-library-telemetry設計.md) |
| authlib | JWT 検証（サーバー用）/ OAuth2 PKCE トークン管理（クライアント用） | 全サーバー・クライアント | [system-library-authlib設計](system-library-authlib設計.md) |
| k1s0-messaging | Kafka イベント発行・購読の抽象化（EventProducer トレイト・EventEnvelope） | 全サーバー（Kafka イベント発行） | [system-library-messaging設計](system-library-messaging設計.md) |
| k1s0-kafka | Kafka 接続設定・管理・ヘルスチェック（KafkaConfig・TLS 対応） | k1s0-messaging を使うサーバー | [system-library-kafka設計](system-library-kafka設計.md) |
| k1s0-correlation | 分散トレーシング用相関 ID・トレース ID 管理（UUID v4・32 文字 hex） | 全サーバー・クライアント | [system-library-correlation設計](system-library-correlation設計.md) |
| k1s0-outbox | トランザクショナルアウトボックスパターン（指数バックオフリトライ） | Kafka 発行を必要とするサーバー | [system-library-outbox設計](system-library-outbox設計.md) |
| k1s0-schemaregistry | Confluent Schema Registry クライアント（Avro/Json/Protobuf 対応） | Kafka プロデューサー・コンシューマー | [system-library-schemaregistry設計](system-library-schemaregistry設計.md) |
| k1s0-serviceauth | サービス間 OAuth2 Client Credentials 認証（トークンキャッシュ・SPIFFE） | サービス間 gRPC/HTTP 通信を行うサーバー | [system-library-serviceauth設計](system-library-serviceauth設計.md) |
| k1s0-saga | SagaサーバーREST/gRPCクライアントSDK | サービス間Saga起動・状態確認 | [system-library-saga設計](system-library-saga設計.md) |
| k1s0-dlq-client | Kafka DLQ メッセージ管理クライアント | DLQメッセージ再処理・モニタリング | [system-library-dlq-client設計](system-library-dlq-client設計.md) |

---

## テスト方針

全ライブラリで TDD を適用する。

| 言語 | ユニットテスト | モック | 統合テスト |
|------|---------------|--------|-----------|
| Go | testify + assert/require | gomock | testcontainers-go |
| Rust | #[cfg(test)] + assert | mockall | wiremock |
| TypeScript | vitest + expect | MSW | vitest |
| Dart | test + expect | mocktail | test |
| C# | xUnit + Assert | NSubstitute | WireMock.Net + Testcontainers |
| Swift | Swift Testing (@Suite, @Test) | Swift Concurrency + protocol | Swift Testing |

### テストカバレッジ目標

| 対象 | カバレッジ |
|------|-----------|
| config ライブラリ | 90% 以上 |
| telemetry ライブラリ | 80% 以上 |
| authlib ライブラリ | 90% 以上 |
| k1s0-messaging | 85% 以上 |
| k1s0-kafka | 80% 以上 |
| k1s0-correlation | 90% 以上 |
| k1s0-outbox | 85% 以上 |
| k1s0-schemaregistry | 85% 以上 |
| k1s0-serviceauth | 90% 以上 |
| k1s0-saga | 85% 以上 |
| k1s0-dlq-client | 85% 以上 |

C# 実装のカバレッジ計測には `coverlet` + `dotnet test --collect:"XPlat Code Coverage"` を使用し、各ライブラリのカバレッジ目標は上記テーブルと同一とする。

---

## 関連ドキュメント

- [config設計](config設計.md) — config.yaml スキーマ・環境別管理
- [可観測性設計](可観測性設計.md) — OpenTelemetry・構造化ログ・メトリクス
- [認証認可設計](認証認可設計.md) — OAuth2.0・JWT・RBAC
- [tier-architecture](tier-architecture.md) — Tier 間の依存関係
- [コーディング規約](コーディング規約.md) — 命名規則・Linter 設定
- [ディレクトリ構成図](ディレクトリ構成図.md) — ライブラリのディレクトリ構成
- [system-server設計](system-server設計.md) — auth サーバーの詳細設計
- [system-database設計](system-database設計.md) — auth-db スキーマ設計
