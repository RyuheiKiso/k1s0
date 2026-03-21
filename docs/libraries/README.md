# ライブラリ設計書

k1s0 system tier が提供する共通ライブラリの設計書一覧。
全ライブラリは Go / Rust / TypeScript / Dart の4言語で平等に実装する。

詳細は [_common/概要.md](./_common/概要.md) を参照。

## 共通ドキュメント

| ドキュメント | 内容 |
|------------|------|
| [_common/概要.md](./_common/概要.md) | 全ライブラリ一覧・テスト方針・カバレッジ目標 |
| [_common/多言語提供方針.md](./_common/多言語提供方針.md) | Tier定義・全ライブラリ分類・新規追加ルール・互換性ポリシー |
| [_common/共通実装パターン.md](./_common/共通実装パターン.md) | 言語横断の共通実装パターン |
| [_common/イベント駆動設計.md](./_common/イベント駆動設計.md) | イベント駆動アーキテクチャ共通設計 |
| [_common/バリデーション共通ルール.md](./_common/バリデーション共通ルール.md) | 全言語共通バリデーションルール |
| [_common/server-common.md](./_common/server-common.md) | サーバー共通ライブラリ設計 |
| [_common/building-blocks.md](./_common/building-blocks.md) | Building Blocks パターン設計 |

---

## 認証・セキュリティ（auth-security）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| auth | JWT 検証（サーバー用）/ OAuth2 PKCE トークン管理（クライアント用） | [auth-security/auth.md](./auth-security/auth.md) |
| k1s0-serviceauth | サービス間 OAuth2 Client Credentials 認証（トークンキャッシュ・SPIFFE） | [auth-security/serviceauth.md](./auth-security/serviceauth.md) |
| k1s0-vault-client | Vault シークレット管理クライアント（シークレット取得・リース管理） | [auth-security/vault-client.md](./auth-security/vault-client.md) |
| k1s0-encryption | AES-GCM / RSA / Argon2id 暗号化・復号化ユーティリティ | [auth-security/encryption.md](./auth-security/encryption.md) |
| secret-store（Go） | Vault・環境変数・ファイル・InMemory 統一シークレット取得 | [data/statestore.md](./data/statestore.md) |

---

## 設定（config）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| config | YAML 設定読み込み・環境別オーバーライド・バリデーション | [config/config.md](./config/config.md) |
| k1s0-featureflag | フィーチャーフラグクライアント SDK（InMemory実装・テスト/ローカル開発用） | [config/featureflag.md](./config/featureflag.md) |
| binding（Go） | HTTP・S3・InMemory の統一 Input/Output バインディング | [config/binding.md](./config/binding.md) |

---

## データ管理（data）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| k1s0-cache | Redis 分散キャッシュ抽象化（get/set/delete・Cluster/Sentinel対応） | [data/cache.md](./data/cache.md) |
| k1s0-pagination | カーソルベース / オフセットベースページネーション・ソート対応 | [data/pagination.md](./data/pagination.md) |
| k1s0-migration | DB マイグレーション管理（バージョン管理・ロールバック・履歴） | [data/migration.md](./data/migration.md) |
| k1s0-migration-evolution | マイグレーション進化設計 | [data/migration-evolution.md](./data/migration-evolution.md) |
| k1s0-distributed-lock | Redis / PostgreSQL ベース分散ロック（TTL付きリース） | [data/distributed-lock.md](./data/distributed-lock.md) |
| k1s0-eventstore | イベントソーシング向けイベント永続化・再生基盤（Append-onlyストリーム） | [data/eventstore.md](./data/eventstore.md) |
| k1s0-schemaregistry | Confluent Schema Registry クライアント（Avro/Json/Protobuf） | [data/schemaregistry.md](./data/schemaregistry.md) |
| statestore（Go） | Redis・InMemory 統一 KV ストア（ETag 楽観ロック） | [data/statestore.md](./data/statestore.md) |

---

## メッセージング（messaging）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| k1s0-messaging | Kafka イベント発行・購読抽象化（EventProducer トレイト・EventEnvelope） | [messaging/messaging.md](./messaging/messaging.md) |
| k1s0-kafka | Kafka 接続設定・管理・ヘルスチェック（KafkaConfig・TLS対応） | [messaging/kafka.md](./messaging/kafka.md) |
| k1s0-outbox | トランザクショナルアウトボックスパターン（指数バックオフリトライ） | [messaging/outbox.md](./messaging/outbox.md) |
| k1s0-dlq-client | Kafka DLQ メッセージ管理クライアント | [messaging/dlq-client.md](./messaging/dlq-client.md) |
| k1s0-event-bus | ドメインイベントバス（in-process パブサブ・非同期ハンドラー） | [messaging/event-bus.md](./messaging/event-bus.md) |
| k1s0-websocket | WebSocket 接続管理・自動再接続・Ping/Pong ハートビート | [messaging/websocket.md](./messaging/websocket.md) |
| k1s0-notification-client | メール / SMS / Push 通知の統一インターフェース送信クライアント | [messaging/notification-client.md](./messaging/notification-client.md) |
| k1s0-webhook-client | HMAC 署名・リトライ・配信ログ付き Webhook 送信クライアント | [messaging/webhook-client.md](./messaging/webhook-client.md) |
| pubsub（Go） | Kafka・Redis・InMemory 統一 Pub/Sub インターフェース | [messaging/pubsub.md](./messaging/pubsub.md) |

---

## 可観測性（observability）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| telemetry | OpenTelemetry 初期化・構造化ログ・分散トレース・メトリクス | [observability/telemetry.md](./observability/telemetry.md) |
| k1s0-correlation | 分散トレーシング用相関 ID・トレース ID 管理（UUID v4・32文字hex） | [observability/correlation.md](./observability/correlation.md) |
| k1s0-tracing | W3C TraceContext 伝播・Span 作成・Baggage 管理 | [observability/tracing.md](./observability/tracing.md) |
| k1s0-health | liveness / readiness / startup プローブ・依存サービス状態集約 | [observability/health.md](./observability/health.md) |
| k1s0-audit-client | 監査ログ送信クライアント（audit-server への構造化ログ・バッチ送信） | [observability/audit-client.md](./observability/audit-client.md) |
| telemetry-macros（Go） | 関数ラッパー（Trace/TraceValue/InstrumentDB/KafkaTracingMiddleware） | [observability/telemetry-macros.md](./observability/telemetry-macros.md) |

---

## レジリエンス（resilience）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| k1s0-retry | 指数バックオフリトライ・OpenTelemetry メトリクス連携 | [resilience/retry.md](./resilience/retry.md) |
| k1s0-circuit-breaker | Open/HalfOpen/Closed 状態管理・閾値ベース自動遮断 | [resilience/circuit-breaker.md](./resilience/circuit-breaker.md) |
| k1s0-bulkhead | 最大同時実行数制限・待機タイムアウト付き拒否制御 | [resilience/bulkhead.md](./resilience/bulkhead.md) |
| k1s0-idempotency | Idempotency-Key ヘッダー処理・TTL 付きレスポンスキャッシュ | [resilience/idempotency.md](./resilience/idempotency.md) |
| k1s0-resiliency | retry + circuit-breaker + bulkhead + タイムアウト統合ポリシー | [resilience/resiliency.md](./resilience/resiliency.md) |
| k1s0-saga | Saga サーバー REST/gRPC クライアント SDK | [resilience/saga.md](./resilience/saga.md) |

---

## クライアント SDK（client-sdk）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| k1s0-graphql-client | GraphQL クエリ・ミューテーション・サブスクリプション（WebSocket）クライアント | [client-sdk/graphql-client.md](./client-sdk/graphql-client.md) |
| k1s0-session-client | session-server へのセッション作成・取得・更新・失効クライアント | [client-sdk/session-client.md](./client-sdk/session-client.md) |
| k1s0-file-client | S3 / GCS / Ceph 対応ファイルストレージクライアント・マルチパートアップロード | [client-sdk/file-client.md](./client-sdk/file-client.md) |
| k1s0-quota-client | リソース使用量追跡・制限チェック・超過通知クライアント | [client-sdk/quota-client.md](./client-sdk/quota-client.md) |
| k1s0-ratelimit-client | スライディングウィンドウ / トークンバケット分散カウンタークライアント | [client-sdk/ratelimit-client.md](./client-sdk/ratelimit-client.md) |
| k1s0-scheduler-client | ジョブ登録・cron スケジュール・実行履歴管理クライアント | [client-sdk/scheduler-client.md](./client-sdk/scheduler-client.md) |
| k1s0-search-client | Elasticsearch / OpenSearch 全文検索クライアント・クエリビルダー | [client-sdk/search-client.md](./client-sdk/search-client.md) |
| k1s0-tenant-client | テナント作成・メンバー管理・プロビジョニング状態確認クライアント | [client-sdk/tenant-client.md](./client-sdk/tenant-client.md) |
| k1s0-app-updater | App Registry バージョンチェック・SHA-256 チェックサム検証 | [client-sdk/app-updater.md](./client-sdk/app-updater.md) |
| bb-ai-client | AI Gateway への HTTP クライアント（AiClient trait・mockall 対応） | [client-sdk/bb-ai-client.md](./client-sdk/bb-ai-client.md) |

---

## コード生成（codegen）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| codegen | スキャフォールド生成・命名変換ユーティリティ（Tera テンプレート） | [codegen/codegen.md](./codegen/codegen.md) |
| client-sdk-generator | クライアント SDK 自動生成 | [codegen/client-sdk-generator.md](./codegen/client-sdk-generator.md) |
| event-codegen | Kafka イベントコード生成 | [codegen/event-codegen.md](./codegen/event-codegen.md) |
| multi-language-sdk | 多言語 SDK 生成パイプライン | [codegen/multi-language-sdk.md](./codegen/multi-language-sdk.md) |

---

## テスト（testing）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| k1s0-test-helper | フィクスチャ生成・モックビルダー・DBセットアップ・テストコンテナ管理 | [testing/test-helper.md](./testing/test-helper.md) |
| k1s0-validation | 宣言的ルール定義・多言語エラーメッセージ・カスタムバリデーター | [testing/validation.md](./testing/validation.md) |

---

## Building Blocks（building-blocks）

| ライブラリ | 用途 | 設計書 |
|-----------|------|--------|
| bb-core | Building Blocks コア共通インターフェース | [building-blocks/bb-core.md](./building-blocks/bb-core.md) |

---

## 独立した設計書

| ドキュメント | 内容 |
|------------|------|
| [server-common.md](./server-common.md) | サーバー共通ライブラリ（エラー型・HTTP/gRPC統合基盤）設計書 |
| [outbox.md](./outbox.md) | アウトボックスパターン独立設計書 |

---

## 関連ドキュメント

- [アーキテクチャ設計書](../architecture/README.md) — 全体設計方針
- [サーバー設計書](../servers/README.md) — ライブラリを利用するサーバー一覧
- [コーディング規約](../architecture/conventions/コーディング規約.md) — 言語別コーディングルール
- [可観測性設計](../architecture/observability/可観測性設計.md) — OpenTelemetry 設計
