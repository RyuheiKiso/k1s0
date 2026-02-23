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

**Cargo.toml への追加行**:

```toml
k1s0-correlation = { path = "../../system/library/rust/correlation" }
```

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

**配置先**: `regions/system/library/go/correlation/`

```
correlation/
├── correlation.go
├── headers.go
├── correlation_test.go
├── go.mod
└── go.sum
```

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

**配置先**: `regions/system/library/typescript/correlation/`

```
correlation/
├── src/
│   ├── types.ts
│   ├── headers.ts
│   └── index.ts
├── __tests__/
│   └── correlation.test.ts
├── package.json
└── tsconfig.json
```

**パッケージ名**: `@k1s0/correlation`
**依存関係**: なし（`crypto.randomUUID()` を使用）

## Dart 実装

**配置先**: `regions/system/library/dart/correlation/`

```
correlation/
├── lib/
│   ├── src/
│   │   ├── types.dart
│   │   └── headers.dart
│   └── correlation.dart
├── test/
│   └── correlation_test.dart
├── pubspec.yaml
└── analysis_options.yaml
```

**パッケージ名**: `k1s0_correlation`
**依存関係**: `uuid: ^4.4.0`

## C# 実装

**配置先**: `regions/system/library/csharp/correlation/`

```
correlation/
├── src/
│   ├── Correlation.csproj
│   ├── CorrelationContext.cs      # 相関 ID + トレース ID コンテキスト
│   ├── CorrelationId.cs           # UUID v4 ベースの相関 ID
│   ├── TraceId.cs                 # 32 文字 hex のトレース ID
│   ├── CorrelationIdGenerator.cs  # 相関 ID 生成ユーティリティ
│   └── CorrelationHeaders.cs      # HTTP ヘッダー定数（X-Correlation-Id 等）
├── tests/
│   ├── Correlation.Tests.csproj
│   └── Unit/
│       ├── CorrelationContextTests.cs
│       └── CorrelationIdTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**: なし（`System.Guid` を使用）

**名前空間**: `K1s0.System.Correlation`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `CorrelationContext` | record | 相関 ID + トレース ID をまとめたコンテキスト |
| `CorrelationId` | readonly struct | UUID v4 ベースの相関 ID |
| `TraceId` | readonly struct | 32 文字 hex のトレース ID（OpenTelemetry 互換） |
| `CorrelationIdGenerator` | static class | 相関 ID・トレース ID 生成 |
| `CorrelationHeaders` | static class | HTTP ヘッダー定数 |

**主要 API**:

```csharp
namespace K1s0.System.Correlation;

public readonly record struct CorrelationId(Guid Value)
{
    public static CorrelationId New() => new(Guid.NewGuid());
    public override string ToString() => Value.ToString("D");
}

public readonly record struct TraceId(string Value)
{
    public static TraceId New();
    public override string ToString() => Value;
}

public record CorrelationContext(CorrelationId CorrelationId, TraceId TraceId)
{
    public static CorrelationContext New() =>
        new(CorrelationId.New(), TraceId.New());
    public CorrelationContext Propagate();
}

public static class CorrelationHeaders
{
    public const string CorrelationIdHeader = "X-Correlation-Id";
    public const string TraceIdHeader = "X-Trace-Id";
}
```

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Correlation`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// 相関 ID（UUID v4 ラッパー）
public struct CorrelationId: Hashable, Sendable, CustomStringConvertible {
    public let value: UUID
    public init()
    public init?(string: String)
    public var description: String { value.uuidString }
}

// トレース ID（32 文字 hex）
public struct TraceId: Hashable, Sendable, CustomStringConvertible {
    public let value: String
    public init()
    public init?(hex: String)
    public var description: String { value }
}

// 相関コンテキスト
public struct CorrelationContext: Sendable {
    public let correlationId: CorrelationId
    public let traceId: TraceId
    public let parentId: String?
    public init(correlationId: CorrelationId = .init(), traceId: TraceId = .init(), parentId: String? = nil)
}

// HTTP ヘッダー定数
public enum CorrelationHeaders {
    public static let correlationId = "X-Correlation-Id"
    public static let traceId = "X-Trace-Id"
    public static let parentId = "X-Parent-Id"

    public static func extract(from headers: [String: String]) -> CorrelationContext?
    public static func inject(_ context: CorrelationContext, into headers: inout [String: String])
}
```

### エラー型
なし（不正な値は `Optional` で対応）

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
