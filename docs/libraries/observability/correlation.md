# k1s0-correlation ライブラリ設計

## 概要

分散トレーシング用相関 ID・トレース ID 管理ライブラリ。`CorrelationId`（UUID v4）、`TraceId`（32 文字 hex）、`CorrelationContext`、HTTP ヘッダー定数を提供する。サービス間リクエストの追跡に使用し、全サーバー・クライアントで統一的に利用する。

**配置先**: `regions/system/library/rust/correlation/`

## 公開 API

| 型・定数 | 種別 | 説明 |
|---------|------|------|
| `CorrelationId` | 構造体 | UUID v4 ベースの相関 ID（新規生成・文字列変換対応） |
| `TraceId` | 構造体 | 32 文字 hex のトレース ID（OpenTelemetry 互換） |
| `CorrelationContext` | 構造体 | 相関 ID + トレース ID をまとめたコンテキスト |
| `CorrelationHeaders` | 構造体 | HTTP ヘッダー定数（`X-Correlation-Id`・`X-Trace-Id` 等） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-correlation"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

**依存追加**: `k1s0-correlation = { path = "../../system/library/rust/correlation" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
correlation/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── context.rs      # CorrelationContext・CorrelationHeaders（HTTP ヘッダー定数）
│   └── id.rs           # CorrelationId（UUID v4）・TraceId（32 文字 hex）
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_correlation::{CorrelationContext, CorrelationHeaders, CorrelationId, TraceId};

// 新規コンテキスト生成（リクエスト受信時）
let ctx = CorrelationContext::new(CorrelationId::new(), TraceId::new());

// HTTP ヘッダーへの設定
let headers = [
    (CorrelationHeaders::CORRELATION_ID, ctx.correlation_id().to_string()),
    (CorrelationHeaders::TRACE_ID, ctx.trace_id().to_string()),
];

// 下流リクエストへの伝播
let child_ctx = ctx.propagate(); // 相関 ID 継承・新規スパン ID 生成
```

## Go 実装

**配置先**: `regions/system/library/go/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**主要型**:

```go
type CorrelationId string
type TraceId string
type CorrelationContext struct {
    CorrelationId CorrelationId
    TraceId       TraceId
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**パッケージ名**: `@k1s0/correlation`
**依存関係**: なし（`crypto.randomUUID()` を使用）

## Dart 実装

**配置先**: `regions/system/library/dart/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**パッケージ名**: `k1s0_correlation`
**依存関係**: `uuid: ^4.4.0`

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) — k1s0-kafka ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ

---
