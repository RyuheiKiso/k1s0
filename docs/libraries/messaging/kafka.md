# k1s0-kafka ライブラリ設計

## 概要

Kafka 接続設定・管理・ヘルスチェックライブラリ。`KafkaConfig`（TLS・SASL 対応）、`KafkaHealthChecker`、`TopicConfig`（命名規則検証）を提供する。k1s0-messaging の具体的な Kafka 実装の基盤となる。

**配置先**: `regions/system/library/rust/kafka/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `KafkaConfig` | 構造体 | ブローカーアドレス・セキュリティプロトコル・コンシューマーグループ・タイムアウト・メッセージサイズ設定 |
| `KafkaConfigBuilder` | 構造体 | `KafkaConfig` のビルダー |
| `KafkaHealthChecker` | 構造体（Rust）/ インターフェース（Go/TS）/ 抽象クラス（Dart） | Kafka クラスター設定妥当性確認・ヘルスチェック |
| `KafkaHealthStatus` | enum / 構造体 | ヘルス状態。Rust は enum（`Healthy` / `Unhealthy(String)`）、Go/TypeScript/Dart は構造体（`healthy: bool`, `message: String`, `brokerCount: int` フィールド） |
| `TopicConfig` | 構造体 | トピック名・パーティション数・レプリケーションファクター・保持期間の設定 |
| `TopicPartitionInfo` | 構造体 | トピックのパーティション情報（リーダー・レプリカ・ISR）。Rust/Go のみ実装 [^1] |
| `KafkaError` | enum | 接続失敗・トピック未検出・パーティション・設定・タイムアウトエラー型 |
| `NoOpKafkaHealthChecker` | 構造体 / クラス | テスト用 no-op ヘルスチェッカー。Go/TS/Dart に存在 [^2] |

[^1]: TS/Dart は現時点で設定・検証のみのスコープであり、`TopicPartitionInfo` は未実装。
[^2]: Rust の `KafkaHealthChecker` は具象 struct であるためテスト時に直接利用可能であり、`NoOpKafkaHealthChecker` は不要。Go/TS/Dart では `KafkaHealthChecker` がインターフェース / 抽象クラスであるため、テスト用の no-op 実装を提供している。

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-kafka"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"

[dev-dependencies]
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-kafka = { path = "../../system/library/rust/kafka" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
kafka/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── config.rs       # KafkaConfig（TLS・SASL 設定を含む）
│   ├── error.rs        # KafkaError
│   ├── health.rs       # KafkaHealthChecker
│   └── topic.rs        # TopicConfig・TopicPartitionInfo・命名規則検証
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_kafka::{KafkaConfig, KafkaHealthChecker, TopicConfig};

// 設定例（SASL_SSL）- ビルダーパターンで構築
let config = KafkaConfig::builder()
    .brokers(vec!["kafka:9092".to_string()])
    .consumer_group("auth-service-group")
    .security_protocol("SASL_SSL")
    .connection_timeout_ms(10000)
    .request_timeout_ms(60000)
    .max_message_bytes(2_000_000)
    .build()?;

// ヘルスチェック（設定の妥当性確認）
let checker = KafkaHealthChecker::new(config.clone());
checker.check().await?;  // async 版: KafkaHealthStatus::Healthy を返す
// 同期チェックも利用可能
checker.check_config()?;

// TLS 使用判定
assert!(config.uses_tls());
// ブローカー文字列取得（rdkafka 用）
let bootstrap = config.bootstrap_servers();

// トピック設定と命名規則検証（k1s0.{tier}.{domain}.{event-type}.{version}）
let topic = TopicConfig {
    name: "k1s0.system.auth.user-created.v1".to_string(),
    partitions: 3,       // デフォルト: 3
    replication_factor: 3, // デフォルト: 3
    retention_ms: 604_800_000, // デフォルト: 7日
};
assert!(topic.validate_name());
```

**SASL フィールドに関する注記**: Go/TypeScript/Dart の `KafkaConfig` は `saslMechanism`・`saslUsername`・`saslPassword` フィールドを保持するが、Rust の `KafkaConfig` は現時点でこれらのフィールドを持たない（`security_protocol` のみ対応）。SASL 認証が必要な場合は rdkafka の設定で別途指定する想定。

## Go 実装

**配置先**: `regions/system/library/go/kafka/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.10.0`（Kafka クライアントライブラリ不要、設定・検証のみ）

**主要型**:

```go
// --- デフォルト値定数 ---
const (
    DefaultConnectionTimeoutMs = 5000    // 接続タイムアウト（ミリ秒）
    DefaultRequestTimeoutMs    = 30000   // リクエストタイムアウト（ミリ秒）
    DefaultMaxMessageBytes     = 1000000 // 最大メッセージサイズ（バイト）
)

// --- KafkaConfig ---
type KafkaConfig struct {
    BootstrapServers    []string  // ブローカーアドレスリスト
    SecurityProtocol    string    // PLAINTEXT, SSL, SASL_PLAINTEXT, SASL_SSL
    SASLMechanism       string    // PLAIN, SCRAM-SHA-256, SCRAM-SHA-512
    SASLUsername         string
    SASLPassword         string
    ConsumerGroup       string    // コンシューマーグループ ID
    ConnectionTimeoutMs int       // 0 の場合はデフォルト値を使用
    RequestTimeoutMs    int       // 0 の場合はデフォルト値を使用
    MaxMessageBytes     int       // 0 の場合はデフォルト値を使用
}

func (c *KafkaConfig) BootstrapServersString() string
func (c *KafkaConfig) UsesTLS() bool
func (c *KafkaConfig) Validate() error
func (c *KafkaConfig) EffectiveConnectionTimeoutMs() int  // 0 の場合はデフォルト値を返す
func (c *KafkaConfig) EffectiveRequestTimeoutMs() int     // 0 の場合はデフォルト値を返す
func (c *KafkaConfig) EffectiveMaxMessageBytes() int      // 0 の場合はデフォルト値を返す

// --- TopicConfig ---
type TopicConfig struct {
    Name              string
    Partitions        int
    ReplicationFactor int
    RetentionMs       int64
}

func (t *TopicConfig) ValidateName() error  // エラー返却（Rust は bool 返却）
func (t *TopicConfig) Tier() string

// --- TopicPartitionInfo ---
type TopicPartitionInfo struct {
    Topic     string
    Partition int32
    Leader    int32
    Replicas  []int32
    ISR       []int32
}

// --- Health Check ---
type KafkaHealthChecker interface {
    HealthCheck(ctx context.Context) (*KafkaHealthStatus, error)
}

type NoOpKafkaHealthChecker struct {
    Status *KafkaHealthStatus
    Err    error
}

func (n *NoOpKafkaHealthChecker) HealthCheck(ctx context.Context) (*KafkaHealthStatus, error)

// --- KafkaError ---
type KafkaError struct {
    Op      string  // エラーが発生した操作名
    Message string
    Err     error   // 原因エラー
}

func (e *KafkaError) Error() string
func (e *KafkaError) Unwrap() error
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/kafka/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface KafkaConfig {
  bootstrapServers: string[];
  securityProtocol?: 'PLAINTEXT' | 'SSL' | 'SASL_PLAINTEXT' | 'SASL_SSL';
  saslMechanism?: string;
  saslUsername?: string;
  saslPassword?: string;
}

export function validateKafkaConfig(config: KafkaConfig): void;
export function bootstrapServersString(config: KafkaConfig): string;
export function usesTLS(config: KafkaConfig): boolean;

export interface TopicConfig {
  name: string;
  partitions?: number;
  replicationFactor?: number;
  retentionMs?: number;
}

export function validateTopicName(topic: TopicConfig): void;
export function topicTier(topic: TopicConfig): 'system' | 'business' | 'service' | '';

export interface KafkaHealthStatus {
  healthy: boolean;
  message: string;
  brokerCount: number;
}

export interface KafkaHealthChecker {
  healthCheck(): Promise<KafkaHealthStatus>;
}

export class NoOpKafkaHealthChecker implements KafkaHealthChecker {
  constructor(status: KafkaHealthStatus, error?: Error);
  healthCheck(): Promise<KafkaHealthStatus>;
}

export class KafkaError extends Error {
  constructor(message: string);
}
```

**カバレッジ目標**: 80%以上

## Dart 実装

**配置先**: `regions/system/library/dart/kafka/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `lints: ^4.0.0` (dev)（Kafka クライアントライブラリ不要、設定・検証のみ）

**主要 API**:

```dart
class KafkaConfig {
  final List<String> bootstrapServers;
  final String? securityProtocol;
  final String? saslMechanism;
  final String? saslUsername;
  final String? saslPassword;

  String bootstrapServersString();
  bool usesTLS();
  void validate();
}

class TopicConfig {
  final String name;
  final int? partitions;
  final int? replicationFactor;
  final int? retentionMs;

  void validateName();
  String tier();
}

class KafkaHealthStatus {
  final bool healthy;
  final String message;
  final int brokerCount;
}

abstract class KafkaHealthChecker {
  Future<KafkaHealthStatus> healthCheck();
}

class NoOpKafkaHealthChecker implements KafkaHealthChecker {
  final KafkaHealthStatus status;
  final Exception? error;

  Future<KafkaHealthStatus> healthCheck();
}

class KafkaError implements Exception {
  final String message;
  final Object? cause;
}
```

**カバレッジ目標**: 80%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-messaging設計](messaging.md) — k1s0-messaging ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](outbox.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ

---
