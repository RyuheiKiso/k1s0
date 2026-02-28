# tracing ライブラリ設計（軽量 W3C TraceContext 伝播ライブラリ）

## 概要

外部依存なしの軽量 W3C TraceContext 伝播ライブラリ。`traceparent` / `baggage` ヘッダーのパース・生成・注入・抽出を提供する。OpenTelemetry SDK には依存せず、純粋なヘッダー操作のみを行う。Rust 実装には簡易的な SpanHandle（名前・イベント管理）も含む。

**設計方針**: OTel 統合ではなく、サービス間でのトレースコンテキスト伝播に特化した軽量実装。

## 公開 API（全言語共通契約）

| 型・関数 | 説明 |
|---------|------|
| `TraceContext` | W3C TraceContext（traceId・parentId・flags） |
| `TraceContext.toTraceparent()` | `traceparent` ヘッダー文字列を生成 |
| `TraceContext.fromTraceparent(s)` | `traceparent` ヘッダー文字列をパース |
| `Baggage` | W3C Baggage（key-value エントリの集合） |
| `Baggage.set(key, value)` | エントリを設定 |
| `Baggage.get(key)` | エントリを取得 |
| `Baggage.toHeader()` | `baggage` ヘッダー文字列を生成 |
| `Baggage.fromHeader(s)` | `baggage` ヘッダー文字列をパース |
| `injectContext(headers, ctx, baggage?)` | headers に `traceparent` と `baggage` を注入 |
| `extractContext(headers)` | headers から `TraceContext` と `Baggage` を抽出 |

## Go 実装

**配置先**: `regions/system/library/go/tracing/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要コード**:

```go
package tracing

type TraceContext struct {
    TraceID  string // 32 hex chars
    ParentID string // 16 hex chars
    Flags    byte
}

func (t TraceContext) ToTraceparent() string
func FromTraceparent(s string) (*TraceContext, error)

type Baggage struct { /* sync.RWMutex + map[string]string */ }
func NewBaggage() *Baggage
func (b *Baggage) Set(key, value string)
func (b *Baggage) Get(key string) (string, bool)
func (b *Baggage) ToHeader() string
func BaggageFromHeader(s string) *Baggage

func InjectContext(_ context.Context, headers map[string]string, tc *TraceContext, bag *Baggage)
func ExtractContext(headers map[string]string) (*TraceContext, *Baggage)
```

## Rust 実装

**配置先**: `regions/system/library/rust/tracing/`

```
tracing/
├── src/
│   ├── lib.rs              # inject_context, extract_context（再エクスポート）
│   ├── propagation.rs      # TraceContext（parse/format/inject/extract）
│   ├── baggage.rs          # Baggage（key-value管理、ヘッダーparse/format）
│   └── span.rs             # SpanHandle, start_span, end_span, add_event
└── Cargo.toml
```

**依存関係**: なし（標準ライブラリのみ）

**主要コード**:

```rust
// propagation.rs
#[derive(Debug, Clone, PartialEq)]
pub struct TraceContext {
    pub trace_id: String,
    pub parent_id: String,
    pub flags: u8,
}

impl TraceContext {
    pub fn new(trace_id: &str, parent_id: &str, flags: u8) -> Self;
    pub fn to_traceparent(&self) -> String;
    pub fn from_traceparent(s: &str) -> Option<TraceContext>;
}

// baggage.rs
pub struct Baggage(HashMap<String, String>);

impl Baggage {
    pub fn new() -> Self;
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>);
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn to_header(&self) -> String;
    pub fn from_header(s: &str) -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
}

// lib.rs
pub fn inject_context(headers: &mut HashMap<String, String>, ctx: &TraceContext, baggage: Option<&Baggage>);
pub fn extract_context(headers: &HashMap<String, String>) -> (Option<TraceContext>, Baggage);

// span.rs（簡易スパン管理）
pub struct SpanHandle { pub name: String, pub trace_id: String, pub span_id: String, pub attributes: HashMap<String, String>, pub events: Vec<SpanEvent> }
pub fn start_span(name: &str) -> SpanHandle;
pub fn end_span(handle: SpanHandle);
pub fn add_event(handle: &mut SpanHandle, name: &str, attributes: HashMap<String, String>);
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/tracing/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要 API**:

```typescript
export interface TraceContext {
  traceId: string;
  parentId: string;
  flags: number;
}

export function toTraceparent(ctx: TraceContext): string;
export function fromTraceparent(s: string): TraceContext | null;

export class Baggage {
  set(key: string, value: string): void;
  get(key: string): string | undefined;
  toHeader(): string;
  static fromHeader(s: string): Baggage;
}

export function injectContext(headers: Record<string, string>, ctx: TraceContext, baggage?: Baggage): void;
export function extractContext(headers: Record<string, string>): { context: TraceContext | null; baggage: Baggage };
```

## Dart 実装

**配置先**: `regions/system/library/dart/tracing/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要 API**:

```dart
class TraceContext {
  final String traceId;   // 32 hex chars
  final String parentId;  // 16 hex chars
  final int flags;

  String toTraceparent();
  static TraceContext? fromTraceparent(String s);
}

class Baggage {
  void set(String key, String value);
  String? get(String key);
  String toHeader();
  static Baggage fromHeader(String s);
}

void injectContext(Map<String, String> headers, TraceContext ctx, [Baggage? baggage]);
({TraceContext? context, Baggage baggage}) extractContext(Map<String, String> headers);
```

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-correlation設計](correlation.md) — k1s0-correlation（相関ID・トレースID管理）
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) — 可観測性設計ガイドライン
