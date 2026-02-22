# k1s0-kafka ライブラリ設計

## 概要

Kafka 接続設定・管理・ヘルスチェックライブラリ。`KafkaConfig`（TLS・SASL 対応）、`KafkaHealthChecker`、`TopicConfig`（命名規則検証）を提供する。k1s0-messaging の具体的な Kafka 実装の基盤となる。

**配置先**: `regions/system/library/rust/kafka/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `KafkaConfig` | 構造体 | ブローカーアドレス・セキュリティプロトコル・コンシューマーグループ・タイムアウト・メッセージサイズ設定 |
| `KafkaConfigBuilder` | 構造体 | `KafkaConfig` のビルダー |
| `KafkaHealthChecker` | 構造体 | Kafka クラスター設定妥当性確認・ヘルスチェック |
| `KafkaHealthStatus` | enum | ヘルス状態（`Healthy` / `Unhealthy(String)`） |
| `TopicConfig` | 構造体 | トピック名・パーティション数・レプリケーションファクター・保持期間の設定 |
| `TopicPartitionInfo` | 構造体 | トピックのパーティション情報（リーダー・レプリカ・ISR） |
| `KafkaError` | enum | 接続失敗・トピック未検出・パーティション・設定・タイムアウトエラー型 |

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

**Cargo.toml への追加行**:

```toml
k1s0-kafka = { path = "../../system/library/rust/kafka" }
```

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

## Go 実装

**配置先**: `regions/system/library/go/kafka/`

```
kafka/
├── config.go
├── topic.go
├── health.go
├── kafka_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/stretchr/testify v1.10.0`（Kafka クライアントライブラリ不要、設定・検証のみ）

**主要型**:

```go
type KafkaConfig struct { ... }
type TopicConfig struct { ... }
type KafkaHealthChecker interface { ... }
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/kafka/`

```
kafka/
├── package.json        # "@k1s0/kafka", "type":"module"
├── tsconfig.json       # ES2022, Node16, strict
├── vitest.config.ts
├── src/
│   └── index.ts        # KafkaConfig, TopicConfig, KafkaHealthStatus, KafkaHealthChecker, NoOpKafkaHealthChecker, KafkaError
└── __tests__/
    └── kafka.test.ts
```

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
  constructor(message: string, cause?: Error);
}
```

**カバレッジ目標**: 80%以上

## Dart 実装

**配置先**: `regions/system/library/dart/kafka/`

```
kafka/
├── pubspec.yaml        # k1s0_kafka, sdk >=3.4.0 <4.0.0
├── analysis_options.yaml
├── lib/
│   ├── kafka.dart
│   └── src/
│       ├── config.dart     # KafkaConfig（バリデーション付き）
│       ├── topic.dart      # TopicConfig（命名規則検証）
│       ├── health.dart     # KafkaHealthStatus, KafkaHealthChecker, NoOpKafkaHealthChecker
│       └── error.dart      # KafkaError
└── test/
    └── kafka_test.dart
```

**カバレッジ目標**: 80%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
