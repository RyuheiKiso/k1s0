# k1s0-correlation ライブラリ設計

## 概要

分散トレーシング用相関 ID・トレース ID 管理ライブラリ。`CorrelationId`（UUID v4）、`TraceId`（32 文字 hex）、`CorrelationContext`、HTTP ヘッダー定数・ヘッダー変換関数を提供する。サービス間リクエストの追跡に使用し、全サーバー・クライアントで統一的に利用する。

**配置先**:

- Go: `regions/system/library/go/correlation/`
- Rust: `regions/system/library/rust/correlation/`
- TypeScript: `regions/system/library/typescript/correlation/`
- Dart: `regions/system/library/dart/correlation/`

## 公開 API

| 型・定数・関数 | 種別 | 説明 |
|---------|------|------|
| `CorrelationId` | 構造体 / クラス | UUID v4 ベースの相関 ID（新規生成・パース・文字列変換対応） |
| `TraceId` | 構造体 / クラス | 32 文字 hex のトレース ID（OpenTelemetry 互換、パース時バリデーションあり） |
| `CorrelationContext` | 構造体 / インターフェース | 相関 ID + トレース ID をまとめたコンテキスト |
| `CorrelationHeaders` | 構造体（Rust のみ） | HTTP ヘッダー定数とヘッダー変換メソッドを持つゼロサイズ型 |
| ヘッダー定数 | 定数 | `X-Correlation-Id` / `X-Trace-Id`（Rust のみ小文字、下記注記参照） |
| `toHeaders` / `ToHeaders` | 関数 | `CorrelationContext` を HTTP ヘッダーマップに変換 |
| `fromHeaders` / `FromHeaders` | 関数 | HTTP ヘッダーマップから `CorrelationContext` を復元（欠落時は自動生成） |
| `newCorrelationContext` 等 | ファクトリ関数 | 新しい `CorrelationContext` を生成（言語別の命名は下記参照） |

## 言語間の相違点

### ヘッダー定数の大文字小文字

| 言語 | 定数名 | 値 |
|------|--------|-----|
| Go | `HeaderCorrelationId` / `HeaderTraceId` | `"X-Correlation-Id"` / `"X-Trace-Id"` |
| Rust | `CorrelationHeaders::CORRELATION_ID` / `::TRACE_ID` | `"x-correlation-id"` / `"x-trace-id"` |
| TypeScript | `HEADER_CORRELATION_ID` / `HEADER_TRACE_ID` | `'X-Correlation-Id'` / `'X-Trace-Id'` |
| Dart | `headerCorrelationId` / `headerTraceId` | `'X-Correlation-Id'` / `'X-Trace-Id'` |

Rust のみヘッダー値が全小文字。`from_headers()` はキーを小文字に正規化して比較するため、他言語が Title-Case で送信したヘッダーも正しく受信できる。

### TraceId パースのバリデーション

| 言語 | 大文字 hex の扱い | エラー表現 |
|------|-----------------|-----------|
| Go | 拒否（小文字のみ受け付け） | `error` を返す |
| Rust | 受け入れ（`is_ascii_hexdigit` により A-F も有効） | `Option<Self>`（`None` で返す） |
| TypeScript | 拒否（小文字のみ受け付け） | `Error` を throw |
| Dart | 拒否（小文字のみ受け付け） | `ArgumentError` を throw |

### CorrelationContext の生成挙動

| 言語 | ファクトリ | correlation_id | trace_id |
|------|----------|---------------|----------|
| Go | `NewCorrelationContext()` | 自動生成 | 自動生成（常に存在） |
| Rust | `CorrelationContext::new()` | 自動生成 | `None`（`Option<TraceId>`） |
| TypeScript | `newCorrelationContext()` | 自動生成 | 自動生成（常に存在） |
| Dart | `CorrelationContext.generate()` | 自動生成 | 自動生成（常に存在） |

Rust のみ `trace_id` が `Option<TraceId>` であり、`new()` では `None` に設定される。trace_id を付与するには `with_trace_id()` ビルダーメソッドを使用する。Go・TypeScript・Dart では `trace_id` は常に必須であり、ファクトリ関数で自動生成される。

### fromHeaders で不正な trace_id を受けた場合

| 言語 | 挙動 |
|------|------|
| Go | 新しい TraceId を自動生成 |
| Rust | `trace_id` を `None` に設定 |
| TypeScript | 新しい TraceId を自動生成 |
| Dart | 新しい TraceId を自動生成 |

## Rust 実装

**配置先**: `regions/system/library/rust/correlation/`

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
│   ├── context.rs      # CorrelationContext・CorrelationHeaders（HTTP ヘッダー定数・変換）
│   └── id.rs           # CorrelationId（UUID v4）・TraceId（32 文字 hex）
└── Cargo.toml
```

**公開 API**:

```rust
// --- id.rs ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new() -> Self;                              // UUID v4 で自動生成
    pub fn from_string(s: impl Into<String>) -> Self;  // バリデーションなし
    pub fn as_str(&self) -> &str;                      // 借用
}
// impl Default for CorrelationId  — new() に委譲
// impl Display for CorrelationId  — 内部文字列をそのまま出力

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(String);

impl TraceId {
    pub fn new() -> Self;                                          // UUID ベース 32 文字 hex
    pub fn from_string(s: impl Into<String>) -> Option<Self>;     // 32 文字 ASCII hex（大文字も許可）
    pub fn as_str(&self) -> &str;                                  // 借用
}
// impl Default for TraceId  — new() に委譲
// impl Display for TraceId  — 内部文字列をそのまま出力

// --- context.rs ---

#[derive(Debug, Clone)]
pub struct CorrelationContext {
    pub correlation_id: CorrelationId,  // 公開フィールド
    pub trace_id: Option<TraceId>,      // 公開フィールド（Optional）
}

impl CorrelationContext {
    pub fn new() -> Self;                                                  // correlation_id を自動生成、trace_id = None
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self;            // ビルダーパターン
    pub fn from_correlation_id(correlation_id: CorrelationId) -> Self;    // 既存 ID から生成、trace_id = None
}
// impl Default for CorrelationContext  — new() に委譲

pub struct CorrelationHeaders;

impl CorrelationHeaders {
    pub const CORRELATION_ID: &'static str = "x-correlation-id";   // 小文字
    pub const TRACE_ID: &'static str = "x-trace-id";               // 小文字
    pub fn to_headers(ctx: &CorrelationContext) -> Vec<(String, String)>;
    pub fn from_headers(headers: &[(String, String)]) -> CorrelationContext;
}
```

**使用例**:

```rust
use k1s0_correlation::{CorrelationContext, CorrelationHeaders, CorrelationId, TraceId};

// 新規コンテキスト生成（相関 ID のみ、trace_id は None）
let ctx = CorrelationContext::new();

// trace_id 付きでコンテキスト生成（ビルダーパターン）
let ctx = CorrelationContext::new().with_trace_id(TraceId::new());

// 既存の相関 ID からコンテキスト生成
let ctx = CorrelationContext::from_correlation_id(
    CorrelationId::from_string("req-abc-123")
).with_trace_id(TraceId::new());

// HTTP ヘッダーへの変換
let headers = CorrelationHeaders::to_headers(&ctx);
// => [("x-correlation-id", "..."), ("x-trace-id", "...")]

// フィールドへの直接アクセス（公開フィールド）
println!("correlation_id: {}", ctx.correlation_id);
if let Some(ref trace_id) = ctx.trace_id {
    println!("trace_id: {}", trace_id);
}

// HTTP ヘッダーからコンテキスト復元
let incoming_headers = vec![
    ("X-Correlation-Id".to_string(), "corr-123".to_string()),
    ("X-Trace-Id".to_string(), "4bf92f3577b34da6a3ce929d0e0e4736".to_string()),
];
let ctx = CorrelationHeaders::from_headers(&incoming_headers);
```

## Go 実装

**配置先**: `regions/system/library/go/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**ファイル構成**:

```
correlation/
├── correlation.go      # CorrelationId, TraceId, CorrelationContext
├── headers.go          # ヘッダー定数、ToHeaders, FromHeaders
└── correlation_test.go
```

**公開 API**:

```go
// --- correlation.go ---

type CorrelationId string

func NewCorrelationId() CorrelationId                    // UUID v4 で自動生成
func ParseCorrelationId(s string) CorrelationId          // バリデーションなし
func (c CorrelationId) String() string
func (c CorrelationId) IsEmpty() bool

type TraceId string

func NewTraceId() TraceId                                // UUID v4 のハイフン除去 32 文字 hex
func ParseTraceId(s string) (TraceId, error)             // 32 文字小文字 hex のみ許可（大文字拒否）
func (t TraceId) String() string
func (t TraceId) IsEmpty() bool

type CorrelationContext struct {
    CorrelationId CorrelationId
    TraceId       TraceId
}

func NewCorrelationContext() CorrelationContext           // 両 ID を自動生成

// --- headers.go ---

const (
    HeaderCorrelationId = "X-Correlation-Id"
    HeaderTraceId       = "X-Trace-Id"
)

func ToHeaders(ctx CorrelationContext) map[string]string
func FromHeaders(headers map[string]string) CorrelationContext  // 欠落・不正時は自動生成
```

**使用例**:

```go
package main

import (
    "fmt"
    "regions/system/library/go/correlation"
)

func main() {
    // 新規コンテキスト生成（両 ID を自動生成）
    ctx := correlation.NewCorrelationContext()

    // HTTP ヘッダーへの変換
    headers := correlation.ToHeaders(ctx)
    // => map["X-Correlation-Id":"..." "X-Trace-Id":"..."]

    // フィールドアクセス
    fmt.Println("correlation_id:", ctx.CorrelationId)
    fmt.Println("trace_id:", ctx.TraceId)

    // HTTP ヘッダーからコンテキスト復元
    incoming := map[string]string{
        correlation.HeaderCorrelationId: "corr-123",
        correlation.HeaderTraceId:       "4bf92f3577b34da6a3ce929d0e0e4736",
    }
    ctx = correlation.FromHeaders(incoming)

    // 個別 ID の生成・パース
    cid := correlation.NewCorrelationId()
    tid, err := correlation.ParseTraceId("4bf92f3577b34da6a3ce929d0e0e4736")
    if err != nil {
        fmt.Println("invalid trace id:", err)
    }
    fmt.Println(cid, tid)
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**パッケージ名**: `@k1s0/correlation`
**依存関係**: なし（`crypto.randomUUID()` を使用）

**ファイル構成**:

```
correlation/
└── src/
    ├── types.ts     # CorrelationId, TraceId, CorrelationContext, newCorrelationContext
    ├── headers.ts   # ヘッダー定数, toHeaders, fromHeaders
    └── index.ts     # 再エクスポート
```

**公開 API**:

```typescript
// --- types.ts ---

export class CorrelationId {
    // private constructor — 直接 new 不可
    static generate(): CorrelationId;       // crypto.randomUUID() で生成
    static parse(s: string): CorrelationId; // バリデーションなし
    toString(): string;
    isEmpty(): boolean;
}

export class TraceId {
    // private constructor — 直接 new 不可
    static generate(): TraceId;             // UUID ハイフン除去 32 文字 hex
    static parse(s: string): TraceId;       // 32 文字小文字 hex のみ（大文字は Error を throw）
    toString(): string;
    isEmpty(): boolean;
}

export interface CorrelationContext {
    readonly correlationId: CorrelationId;
    readonly traceId: TraceId;
}

export function newCorrelationContext(): CorrelationContext;  // 両 ID を自動生成

// --- headers.ts ---

export const HEADER_CORRELATION_ID = 'X-Correlation-Id';
export const HEADER_TRACE_ID = 'X-Trace-Id';

export function toHeaders(ctx: CorrelationContext): Record<string, string>;
export function fromHeaders(headers: Record<string, string>): CorrelationContext;  // 欠落・不正時は自動生成
```

**使用例**:

```typescript
import {
  CorrelationId,
  TraceId,
  newCorrelationContext,
  HEADER_CORRELATION_ID,
  HEADER_TRACE_ID,
  toHeaders,
  fromHeaders,
} from '@k1s0/correlation';

// 新規コンテキスト生成（両 ID を自動生成）
const ctx = newCorrelationContext();

// HTTP ヘッダーへの変換
const headers = toHeaders(ctx);
// => { 'X-Correlation-Id': '...', 'X-Trace-Id': '...' }

// フィールドアクセス
console.log('correlationId:', ctx.correlationId.toString());
console.log('traceId:', ctx.traceId.toString());

// HTTP ヘッダーからコンテキスト復元
const incoming = {
  [HEADER_CORRELATION_ID]: 'corr-123',
  [HEADER_TRACE_ID]: '4bf92f3577b34da6a3ce929d0e0e4736',
};
const restored = fromHeaders(incoming);

// 個別 ID の生成・パース
const cid = CorrelationId.generate();
const tid = TraceId.parse('4bf92f3577b34da6a3ce929d0e0e4736');
```

## Dart 実装

**配置先**: `regions/system/library/dart/correlation/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**パッケージ名**: `k1s0_correlation`
**依存関係**: `uuid: ^4.4.0`

**ファイル構成**:

```
correlation/
└── lib/
    ├── correlation.dart      # 再エクスポート
    └── src/
        ├── types.dart        # CorrelationId, TraceId, CorrelationContext
        └── headers.dart      # ヘッダー定数, toHeaders, fromHeaders
```

**公開 API**:

```dart
// --- types.dart ---

class CorrelationId {
    final String value;
    // const CorrelationId._(this.value) — private constructor
    factory CorrelationId.generate();                  // UUID v4 で生成
    factory CorrelationId.parse(String s);              // バリデーションなし
    bool get isEmpty;
    @override String toString();
    @override bool operator ==(Object other);          // value ベースの等値比較
    @override int get hashCode;                        // value ベースのハッシュ
}

class TraceId {
    final String value;
    // const TraceId._(this.value) — private constructor
    factory TraceId.generate();                         // UUID ハイフン除去 32 文字 hex
    factory TraceId.parse(String s);                    // 32 文字小文字 hex のみ（大文字は ArgumentError を throw）
    bool get isEmpty;
    @override String toString();
    @override bool operator ==(Object other);          // value ベースの等値比較
    @override int get hashCode;                        // value ベースのハッシュ
}

class CorrelationContext {
    final CorrelationId correlationId;
    final TraceId traceId;
    const CorrelationContext({required this.correlationId, required this.traceId});
    factory CorrelationContext.generate();              // 両 ID を自動生成
}

// --- headers.dart ---

const headerCorrelationId = 'X-Correlation-Id';
const headerTraceId = 'X-Trace-Id';

Map<String, String> toHeaders(CorrelationContext ctx);
CorrelationContext fromHeaders(Map<String, String> headers);  // 欠落・不正時は自動生成
```

**使用例**:

```dart
import 'package:k1s0_correlation/correlation.dart';

void main() {
  // 新規コンテキスト生成（両 ID を自動生成）
  final ctx = CorrelationContext.generate();

  // HTTP ヘッダーへの変換
  final headers = toHeaders(ctx);
  // => {'X-Correlation-Id': '...', 'X-Trace-Id': '...'}

  // フィールドアクセス
  print('correlationId: ${ctx.correlationId}');
  print('traceId: ${ctx.traceId}');

  // HTTP ヘッダーからコンテキスト復元
  final incoming = {
    headerCorrelationId: 'corr-123',
    headerTraceId: '4bf92f3577b34da6a3ce929d0e0e4736',
  };
  final restored = fromHeaders(incoming);

  // 個別 ID の生成・パース
  final cid = CorrelationId.generate();
  final tid = TraceId.parse('4bf92f3577b34da6a3ce929d0e0e4736');
  print('$cid $tid');
}
```

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) -- ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) -- config ライブラリ
- [system-library-telemetry設計](telemetry.md) -- telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) -- authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) -- k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) -- k1s0-kafka ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) -- k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) -- k1s0-schemaregistry ライブラリ

---
