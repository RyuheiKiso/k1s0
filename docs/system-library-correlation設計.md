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

## Python 実装

### パッケージ構造

```
correlation/
├── pyproject.toml
├── src/
│   └── k1s0_correlation/
│       ├── __init__.py        # 公開 API エクスポート
│       ├── context.py         # CorrelationContext データクラス（correlation_id・trace_id・request_id）
│       ├── generator.py       # generate_correlation_id()（UUID v4）・generate_trace_id()（32文字 hex）
│       ├── propagation.py     # contextvars ベースのコンテキスト伝播・HTTP ヘッダー抽出/注入
│       ├── headers.py         # HTTP ヘッダー定数（x-correlation-id・x-trace-id・x-request-id）
│       ├── exceptions.py      # CorrelationError・CorrelationErrorCodes
│       └── py.typed           # PEP 561 型スタブマーカー
└── tests/
    ├── test_context.py
    ├── test_generator.py
    └── test_propagation.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `CorrelationContext` | dataclass | 相関 ID + トレース ID + リクエスト ID を保持するコンテキスト |
| `generate_correlation_id()` | function | UUID v4 形式の相関 ID を生成 |
| `generate_trace_id()` | function | 32 文字 hex のトレース ID を生成 |
| `set_correlation_context()` | function | `contextvars` で現在のコンテキストに相関コンテキストをセット |
| `get_correlation_context()` | function | 現在のコンテキストから相関コンテキストを取得 |
| `extract_from_headers()` | function | HTTP ヘッダー辞書から `CorrelationContext` を抽出（未存在時は新規生成） |
| `inject_into_headers()` | function | `CorrelationContext` を HTTP ヘッダー辞書に注入 |
| `X_CORRELATION_ID` / `X_TRACE_ID` / `X_REQUEST_ID` | str 定数 | HTTP ヘッダー名定数 |

### 使用例

```python
from k1s0_correlation import (
    CorrelationContext,
    extract_from_headers,
    inject_into_headers,
    set_correlation_context,
    get_correlation_context,
)

# HTTP ヘッダーからコンテキスト抽出
ctx = extract_from_headers(request.headers)
set_correlation_context(ctx)

# 下流リクエストへのヘッダー注入
headers: dict[str, str] = {}
inject_into_headers(ctx, headers)

# 現在のコンテキスト取得（asyncio / contextvars 対応）
current = get_correlation_context()
```

### 依存ライブラリ

外部依存なし（標準ライブラリの `uuid`・`contextvars` のみ使用）。

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90% 以上（`pyproject.toml` の `fail_under = 90`）
- 実行: `pytest` / `ruff check .`
