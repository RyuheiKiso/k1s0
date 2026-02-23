# tracing ライブラリ設計（telemetry ライブラリから分離した分散トレーシング特化ライブラリ）

> 詳細な設計方針は [可観測性-トレーシング設計.md](可観測性-トレーシング設計.md) を参照。

## 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| InitTracing | `(config) -> TracerProvider` | OpenTelemetry TracerProvider 初期化 |
| Shutdown | `() -> void` | TracerProvider のシャットダウン |
| StartSpan | `(ctx, name, attributes?) -> (ctx, Span)` | 新しい Span を開始 |
| EndSpan | `(span, status?) -> void` | Span を終了 |
| AddEvent | `(span, name, attributes?) -> void` | Span にイベントを追加 |
| InjectContext | `(ctx, carrier) -> void` | W3C TraceContext を HTTP ヘッダー等に注入 |
| ExtractContext | `(carrier) -> ctx` | HTTP ヘッダー等から W3C TraceContext を抽出 |
| GetTraceId | `(ctx) -> Option<String>` | 現在の Trace ID を取得 |
| GetSpanId | `(ctx) -> Option<String>` | 現在の Span ID を取得 |
| SetBaggage | `(ctx, key, value) -> ctx` | Baggage にキーバリューを設定 |
| GetBaggage | `(ctx, key) -> Option<String>` | Baggage からキーバリューを取得 |

## Go 実装

**配置先**: `regions/system/library/go/tracing/`

```
tracing/
├── tracing.go         # InitTracing, Shutdown
├── span.go            # StartSpan, EndSpan, AddEvent, AddAttribute
├── propagation.go     # InjectContext, ExtractContext（W3C TraceContext）
├── baggage.go         # SetBaggage, GetBaggage
├── tracing_test.go
├── go.mod
└── go.sum
```

**依存関係**:

```
go.opentelemetry.io/otel
go.opentelemetry.io/otel/sdk
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc
go.opentelemetry.io/otel/propagation
go.opentelemetry.io/otel/baggage
```

**主要コード**:

```go
package tracing

import (
    "context"

    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/attribute"
    "go.opentelemetry.io/otel/baggage"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/propagation"
    "go.opentelemetry.io/otel/sdk/resource"
    sdktrace "go.opentelemetry.io/otel/sdk/trace"
    semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
    "go.opentelemetry.io/otel/trace"
)

type TracingConfig struct {
    ServiceName   string
    Version       string
    Environment   string
    TraceEndpoint string
    SampleRate    float64
}

type TracerProvider struct {
    provider *sdktrace.TracerProvider
    tracer   trace.Tracer
}

func InitTracing(ctx context.Context, cfg TracingConfig) (*TracerProvider, error) {
    exporter, err := otlptracegrpc.New(ctx,
        otlptracegrpc.WithEndpoint(cfg.TraceEndpoint),
        otlptracegrpc.WithInsecure(),
    )
    if err != nil {
        return nil, err
    }
    provider := sdktrace.NewTracerProvider(
        sdktrace.WithBatcher(exporter),
        sdktrace.WithSampler(sdktrace.TraceIDRatioBased(cfg.SampleRate)),
        sdktrace.WithResource(resource.NewWithAttributes(
            semconv.SchemaURL,
            semconv.ServiceNameKey.String(cfg.ServiceName),
            semconv.ServiceVersionKey.String(cfg.Version),
        )),
    )
    otel.SetTracerProvider(provider)
    otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
        propagation.TraceContext{},
        propagation.Baggage{},
    ))
    return &TracerProvider{provider: provider, tracer: provider.Tracer(cfg.ServiceName)}, nil
}

func (tp *TracerProvider) StartSpan(ctx context.Context, name string, attrs ...attribute.KeyValue) (context.Context, trace.Span) {
    return tp.tracer.Start(ctx, name, trace.WithAttributes(attrs...))
}

func (tp *TracerProvider) Shutdown(ctx context.Context) error {
    return tp.provider.Shutdown(ctx)
}

func InjectContext(ctx context.Context, carrier propagation.TextMapCarrier) {
    otel.GetTextMapPropagator().Inject(ctx, carrier)
}

func ExtractContext(ctx context.Context, carrier propagation.TextMapCarrier) context.Context {
    return otel.GetTextMapPropagator().Extract(ctx, carrier)
}

func GetTraceId(ctx context.Context) string {
    span := trace.SpanFromContext(ctx)
    if span.SpanContext().HasTraceID() {
        return span.SpanContext().TraceID().String()
    }
    return ""
}

func GetSpanId(ctx context.Context) string {
    span := trace.SpanFromContext(ctx)
    if span.SpanContext().HasSpanID() {
        return span.SpanContext().SpanID().String()
    }
    return ""
}

func SetBaggage(ctx context.Context, key, value string) context.Context {
    member, _ := baggage.NewMember(key, value)
    bag, _ := baggage.New(member)
    return baggage.ContextWithBaggage(ctx, bag)
}

func GetBaggage(ctx context.Context, key string) string {
    return baggage.FromContext(ctx).Member(key).Value()
}
```

## Rust 実装

**配置先**: `regions/system/library/rust/tracing/`（注意: ディレクトリ名はライブラリ名との競合を避けるため `k1s0-tracing/` も可）

```
tracing/
├── src/
│   ├── lib.rs              # 公開 API（init_tracing, shutdown, start_span 等）
│   ├── span.rs             # Span ユーティリティ（start_span, end_span, add_event）
│   ├── propagation.rs      # W3C TraceContext 注入・抽出
│   └── baggage.rs          # Baggage 管理
├── tests/
│   └── integration/
│       └── tracing_test.rs
└── Cargo.toml
```

**Cargo.toml**:

```toml
[package]
name = "k1s0-tracing"
version = "0.1.0"
edition = "2021"

[dependencies]
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
opentelemetry-http = "0.27"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.28"
```

**主要コード**:

```rust
use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};

pub struct TracingConfig {
    pub service_name: String,
    pub version: String,
    pub environment: String,
    pub trace_endpoint: String,
    pub sample_rate: f64,
}

pub fn init_tracing(cfg: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&cfg.trace_endpoint)
        .build()?;
    let provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sdktrace::Sampler::TraceIdRatioBased(cfg.sample_rate))
        .with_resource(Resource::builder()
            .with_service_name(&cfg.service_name)
            .build())
        .build();
    global::set_tracer_provider(provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
    Ok(())
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}

pub fn inject_context<I: Injector>(ctx: &opentelemetry::Context, injector: &mut I) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(ctx, injector);
    });
}

pub fn extract_context<E: Extractor>(extractor: &E) -> opentelemetry::Context {
    global::get_text_map_propagator(|propagator| {
        propagator.extract(extractor)
    })
}

pub fn get_trace_id(ctx: &opentelemetry::Context) -> Option<String> {
    let span = ctx.span();
    let span_ctx = span.span_context();
    if span_ctx.is_valid() {
        Some(span_ctx.trace_id().to_string())
    } else {
        None
    }
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/tracing/`

```
tracing/
├── src/
│   ├── index.ts         # 公開 API エクスポート
│   ├── tracing.ts       # initTracing, shutdown
│   ├── span.ts          # startSpan, addEvent, addAttribute
│   └── propagation.ts   # injectContext, extractContext, getBaggageValue, setBaggageValue
├── tests/
│   └── unit/
│       └── tracing.test.ts
├── package.json
└── tsconfig.json
```

**package.json**:

```json
{
  "name": "@k1s0/tracing",
  "version": "0.1.0",
  "dependencies": {
    "@opentelemetry/api": "^1.9.0",
    "@opentelemetry/sdk-node": "^0.56.0",
    "@opentelemetry/exporter-trace-otlp-grpc": "^0.56.0",
    "@opentelemetry/propagator-b3": "^1.29.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}
```

**主要コード**:

```typescript
import { context, trace, propagation, Span, SpanStatusCode } from '@opentelemetry/api';
import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';

export interface TracingConfig {
  serviceName: string;
  version: string;
  environment: string;
  traceEndpoint: string;
  sampleRate?: number;
}

let sdk: NodeSDK | undefined;

export function initTracing(cfg: TracingConfig): void {
  const exporter = new OTLPTraceExporter({ url: cfg.traceEndpoint });
  sdk = new NodeSDK({
    traceExporter: exporter,
    serviceName: cfg.serviceName,
  });
  sdk.start();
}

export function shutdown(): Promise<void> {
  return sdk?.shutdown() ?? Promise.resolve();
}

export function startSpan(name: string, attributes?: Record<string, string | number | boolean>): Span {
  const tracer = trace.getActiveSpan()
    ? trace.getTracer('k1s0')
    : trace.getTracer('k1s0');
  return tracer.startSpan(name, { attributes });
}

export function injectContext(carrier: Record<string, string>): void {
  propagation.inject(context.active(), carrier);
}

export function extractContext(carrier: Record<string, string>): void {
  propagation.extract(context.active(), carrier);
}

export function getTraceId(): string | undefined {
  return trace.getActiveSpan()?.spanContext().traceId;
}

export function getSpanId(): string | undefined {
  return trace.getActiveSpan()?.spanContext().spanId;
}

export function setBaggageValue(key: string, value: string): void {
  const bag = propagation.getBaggage(context.active()) ?? propagation.createBaggage();
  const newBag = bag.setEntry(key, { value });
  propagation.setBaggage(context.active(), newBag);
}

export function getBaggageValue(key: string): string | undefined {
  return propagation.getBaggage(context.active())?.getEntry(key)?.value;
}
```

## Dart 実装

**配置先**: `regions/system/library/dart/tracing/`

```
tracing/
├── lib/
│   ├── tracing.dart       # エントリーポイント
│   └── src/
│       ├── tracing.dart   # initTracing, shutdown
│       ├── span.dart      # Span ユーティリティ
│       └── propagation.dart # W3C TraceContext 注入・抽出
├── test/
│   └── unit/
│       └── tracing_test.dart
├── pubspec.yaml
└── analysis_options.yaml
```

**pubspec.yaml**:

```yaml
name: k1s0_tracing
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  opentelemetry: ^0.19.0
dev_dependencies:
  test: ^1.25.0
```

**主要コード**:

```dart
import 'package:opentelemetry/api.dart' as otel;
import 'package:opentelemetry/sdk.dart' as otel_sdk;

class TracingConfig {
  final String serviceName;
  final String version;
  final String environment;
  final String traceEndpoint;
  final double sampleRate;

  TracingConfig({
    required this.serviceName,
    required this.version,
    required this.environment,
    required this.traceEndpoint,
    this.sampleRate = 1.0,
  });
}

late otel.Tracer _tracer;

Future<void> initTracing(TracingConfig cfg) async {
  final sdk = otel_sdk.TracerProviderBase(
    processors: [
      otel_sdk.BatchSpanProcessor(
        otel_sdk.CollectorExporter(Uri.parse(cfg.traceEndpoint)),
      ),
    ],
    resource: otel_sdk.Resource([
      otel.Attribute.fromString('service.name', cfg.serviceName),
      otel.Attribute.fromString('service.version', cfg.version),
    ]),
  );
  otel.registerGlobalTracerProvider(sdk);
  _tracer = otel.globalTracerProvider.getTracer(cfg.serviceName);
}

otel.Span startSpan(String name, {Map<String, String> attributes = const {}}) {
  final span = _tracer.startSpan(name);
  for (final entry in attributes.entries) {
    span.setStringAttribute(entry.key, entry.value);
  }
  return span;
}

String? getTraceId(otel.Context ctx) {
  final span = otel.spanFromContext(ctx);
  final traceId = span.spanContext.traceId;
  return traceId.toString().replaceAll('00000000000000000000000000000000', '').isEmpty
      ? null
      : traceId.toString();
}
```

## C# 実装

**配置先**: `regions/system/library/csharp/tracing/`

```
tracing/
├── src/
│   ├── Tracing.csproj
│   ├── TracingInitializer.cs      # InitTracing（TracerProvider 初期化）
│   ├── SpanExtensions.cs          # StartSpan, AddEvent, AddAttribute 拡張メソッド
│   ├── PropagationHelper.cs       # InjectContext, ExtractContext（W3C TraceContext）
│   ├── BaggageHelper.cs           # SetBaggage, GetBaggage
│   └── TracingException.cs        # 公開例外型
├── tests/
│   ├── Tracing.Tests.csproj
│   ├── Unit/
│   │   ├── SpanExtensionsTests.cs
│   │   └── PropagationHelperTests.cs
│   └── Integration/
│       └── TracingInitializerTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| OpenTelemetry | テレメトリ API |
| OpenTelemetry.Extensions.Hosting | ホスティング統合 |
| OpenTelemetry.Exporter.OpenTelemetryProtocol | OTLP エクスポーター |
| OpenTelemetry.Instrumentation.AspNetCore | ASP.NET Core 自動計測 |

**名前空間**: `K1s0.System.Tracing`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `TracingInitializer` | static class | TracerProvider 初期化・シャットダウン |
| `SpanExtensions` | static class | Span 操作拡張メソッド（StartSpan, AddEvent, AddAttribute） |
| `PropagationHelper` | static class | W3C TraceContext 注入・抽出 |
| `BaggageHelper` | static class | Baggage 設定・取得 |

**主要 API**:

```csharp
namespace K1s0.System.Tracing;

public static class TracingInitializer
{
    public static IServiceCollection AddK1s0Tracing(
        this IServiceCollection services,
        TracingConfig config,
        CancellationToken cancellationToken = default);
}

public static class SpanExtensions
{
    public static Activity? StartSpan(this ActivitySource source, string name, IReadOnlyDictionary<string, string>? attributes = null);
    public static void AddEvent(this Activity? activity, string name, IReadOnlyDictionary<string, string>? attributes = null);
    public static void SetAttribute(this Activity? activity, string key, string value);
}

public static class PropagationHelper
{
    public static void InjectContext(IDictionary<string, string> carrier);
    public static ActivityContext ExtractContext(IReadOnlyDictionary<string, string> carrier);
}

public static class BaggageHelper
{
    public static void SetBaggage(string key, string value);
    public static string? GetBaggage(string key);
}
```

## Swift 実装

### パッケージ構成
- ターゲット: `K1s0Tracing`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public struct TracingConfig: Sendable {
    public let serviceName: String
    public let serviceVersion: String
    public let otlpEndpoint: URL
    public let sampleRate: Double
    public init(serviceName: String, serviceVersion: String = "0.0.0", otlpEndpoint: URL, sampleRate: Double = 1.0)
}

public actor TracingSetup {
    public init(config: TracingConfig)
    public func initialize() async throws
    public func shutdown() async
}

public struct Span: Sendable {
    public let traceId: String
    public let spanId: String

    public func addEvent(_ name: String, attributes: [String: String] = [:])
    public func setAttribute(_ key: String, value: String)
    public func setStatus(_ status: SpanStatus)
    public func end()
}

public enum SpanStatus: Sendable {
    case ok, error(description: String)
}

public struct TracingContext: Sendable {
    public let traceId: String?
    public let spanId: String?
}

// W3C TraceContext 伝播
public func injectContext(_ span: Span, into headers: inout [String: String])
public func extractContext(from headers: [String: String]) -> TracingContext
```

### エラー型
なし（初期化失敗時はログ出力のみ）

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

## Python 実装

**配置先**: `regions/system/library/python/tracing/`

### パッケージ構造

```
tracing/
├── pyproject.toml
├── src/
│   └── k1s0_tracing/
│       ├── __init__.py          # 公開 API（再エクスポート）
│       ├── initializer.py       # init_tracing・shutdown
│       ├── span.py              # start_span・add_event・add_attribute
│       ├── propagation.py       # inject_context・extract_context（W3C TraceContext）
│       ├── baggage.py           # set_baggage・get_baggage
│       ├── models.py            # TracingConfig
│       ├── exceptions.py        # TracingError
│       └── py.typed
└── tests/
    ├── test_tracing.py
    ├── test_propagation.py
    └── test_baggage.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `init_tracing` | function | OpenTelemetry TracerProvider 初期化 |
| `shutdown` | function | TracerProvider のシャットダウン |
| `start_span` | function | 新しい Span をコンテキストマネージャーとして開始 |
| `inject_context` | function | W3C TraceContext を辞書（HTTPヘッダー等）に注入 |
| `extract_context` | function | 辞書から W3C TraceContext を抽出してContextを返す |
| `get_trace_id` | function | 現在の Trace ID を取得 |
| `get_span_id` | function | 現在の Span ID を取得 |
| `set_baggage` | function | Baggage にキーバリューを設定 |
| `get_baggage` | function | Baggage からキーバリューを取得 |
| `TracingConfig` | dataclass | サービス名・バージョン・エンドポイント・サンプリングレート |

### 使用例

```python
from k1s0_tracing import (
    TracingConfig, init_tracing, start_span,
    inject_context, extract_context,
    get_trace_id, set_baggage, get_baggage,
)

# 初期化
config = TracingConfig(
    service_name="order-service",
    service_version="0.1.0",
    endpoint="http://otel-collector:4317",
    sample_rate=0.5,
)
init_tracing(config)

# Span 作成（コンテキストマネージャー）
with start_span("process_order", attributes={"order.id": "ord-001"}) as span:
    span.add_event("order.validated", {"validation.result": "ok"})
    trace_id = get_trace_id()
    print(f"Trace ID: {trace_id}")

# W3C TraceContext 伝播（HTTPリクエストへの注入）
headers: dict[str, str] = {}
inject_context(headers)
# headers に traceparent ヘッダーが設定される

# Baggage 管理
set_baggage("tenant.id", "tenant-001")
tenant_id = get_baggage("tenant.id")
print(f"テナントID: {tenant_id}")
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| opentelemetry-api | >=1.27 | OpenTelemetry API |
| opentelemetry-sdk | >=1.27 | OpenTelemetry SDK |
| opentelemetry-exporter-otlp-proto-grpc | >=1.27 | OTLP gRPC エクスポーター |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 80%以上
- 実行: `pytest` / `ruff check .`

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ（トレース+メトリクス+ログの統合版）
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation（相関ID・トレースID管理）
- [可観測性設計.md](可観測性設計.md) — 可観測性設計ガイドライン
- [可観測性-トレーシング設計.md](可観測性-トレーシング設計.md) — 分散トレーシング詳細設計
