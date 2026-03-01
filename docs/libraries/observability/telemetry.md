# telemetry ライブラリ設計

> 詳細な設計方針は [可観測性設計.md](../../architecture/observability/可観測性設計.md) を参照。

## 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| InitTelemetry | `(config) -> Provider` | OpenTelemetry 初期化（トレース + ログ） |
| Shutdown | `() -> void` | プロバイダーのシャットダウン |
| NewLogger / createLogger | Go/Rust/TS: `(config) -> Logger`, Dart: `(name) -> Logger` | 構造化ログのロガー生成 |
| NewMetrics | `(serviceName) -> Metrics` | Prometheus メトリクス（RED メソッド: リクエスト数・エラー率・レイテンシ） |
| MetricsHandler / gather_metrics / getMetrics / toPrometheusText | `() -> Handler / String` | `/metrics` エンドポイント用ハンドラ / Prometheus テキスト出力 |
| LogWithTrace | `(ctx, logger) -> Logger` | トレース ID・スパン ID をロガーに付与（Go 明示呼び出し / TS は mixin で自動注入 / Dart は middleware で x-trace-id ヘッダ） |

## Go 実装

**配置先**: `regions/system/library/go/telemetry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**ファイル構成**:

```
telemetry/
├── telemetry.go    # TelemetryConfig, Provider, InitTelemetry, Shutdown
├── logger.go       # NewLogger, LogWithTrace
├── middleware.go    # HTTPMiddleware, GRPCUnaryInterceptor
├── metrics.go      # Metrics, NewMetrics, MetricsHandler
└── go.mod
```

**依存関係**:

```
go.opentelemetry.io/otel
go.opentelemetry.io/otel/sdk
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc
github.com/prometheus/client_golang
```

**主要コード**:

```go
package telemetry

import (
    "context"
    "log/slog"
    "os"

    "go.opentelemetry.io/otel"
    "go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
    "go.opentelemetry.io/otel/sdk/resource"
    sdktrace "go.opentelemetry.io/otel/sdk/trace"
    semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
    "go.opentelemetry.io/otel/trace"
)

type TelemetryConfig struct {
    ServiceName   string
    Version       string
    Tier          string
    Environment   string
    TraceEndpoint string
    SampleRate    float64
    LogLevel      string
    LogFormat     string
}

type Provider struct {
    tracerProvider *sdktrace.TracerProvider
    logger         *slog.Logger
}

func InitTelemetry(ctx context.Context, cfg TelemetryConfig) (*Provider, error) {
    var tp *sdktrace.TracerProvider

    if cfg.TraceEndpoint != "" {
        exporter, err := otlptracegrpc.New(ctx,
            otlptracegrpc.WithEndpoint(cfg.TraceEndpoint),
            otlptracegrpc.WithInsecure(),
        )
        if err != nil {
            return nil, err
        }
        tp = sdktrace.NewTracerProvider(
            sdktrace.WithBatcher(exporter),
            sdktrace.WithSampler(sdktrace.TraceIDRatioBased(cfg.SampleRate)),
            sdktrace.WithResource(resource.NewWithAttributes(
                semconv.SchemaURL,
                semconv.ServiceNameKey.String(cfg.ServiceName),
                semconv.ServiceVersionKey.String(cfg.Version),
            )),
        )
        otel.SetTracerProvider(tp)
    }

    logger := NewLogger(cfg)
    return &Provider{tracerProvider: tp, logger: logger}, nil
}

func (p *Provider) Shutdown(ctx context.Context) error {
    if p.tracerProvider != nil {
        return p.tracerProvider.Shutdown(ctx)
    }
    return nil
}

func (p *Provider) Logger() *slog.Logger {
    return p.logger
}

func NewLogger(cfg TelemetryConfig) *slog.Logger {
    level := slog.LevelWarn
    switch cfg.LogLevel {
    case "debug":
        level = slog.LevelDebug
    case "info":
        level = slog.LevelInfo
    case "warn":
        level = slog.LevelWarn
    case "error":
        level = slog.LevelError
    }

    var handler slog.Handler
    if cfg.LogFormat == "text" {
        handler = slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: level})
    } else {
        handler = slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
    }
    return slog.New(handler).With(
        slog.String("service", cfg.ServiceName),
        slog.String("version", cfg.Version),
        slog.String("tier", cfg.Tier),
        slog.String("environment", cfg.Environment),
    )
}

func LogWithTrace(ctx context.Context, logger *slog.Logger) *slog.Logger {
    spanCtx := trace.SpanContextFromContext(ctx)
    if spanCtx.HasTraceID() {
        return logger.With(
            slog.String("trace_id", spanCtx.TraceID().String()),
            slog.String("span_id", spanCtx.SpanID().String()),
        )
    }
    return logger
}
```

**Metrics の使用パターン**: Go の `Metrics` 構造体は 4 つのフィールドを直接公開する。他言語のように `record*` ヘルパーメソッドは無く、呼び出し元が直接 Prometheus API を使用する。

```go
m := NewMetrics("order-server")
m.HTTPRequestsTotal.With(prometheus.Labels{"method": "GET", "path": "/api/v1/orders", "status": "200"}).Inc()
m.HTTPRequestDuration.With(prometheus.Labels{"method": "GET", "path": "/api/v1/orders"}).Observe(0.123)
```

## Rust 実装

**配置先**: `regions/system/library/rust/telemetry/`

**ファイル構成**:

```
telemetry/
├── src/
│   ├── lib.rs              # TelemetryConfig, init_telemetry, shutdown
│   ├── logger.rs           # init_logger, parse_log_level
│   ├── metrics.rs          # Metrics (9 メトリクス + recording メソッド)
│   ├── middleware/
│   │   ├── mod.rs          # TelemetryMiddleware, GrpcInterceptor, trace_request!, trace_grpc_call!
│   │   ├── http_layer.rs   # MetricsLayer (feature: axum-layer)
│   │   └── grpc_layer.rs   # GrpcMetricsLayer (feature: grpc-layer)
│   └── tests.rs
└── Cargo.toml
```

### Feature Flags

Rust 実装では Tower Layer によるミドルウェア自動計測を feature flag で提供する。

| Feature | 依存 crate | 有効になる API |
|---------|-----------|---------------|
| `axum-layer` | axum, http, tower, pin-project-lite | `MetricsLayer` (HTTP 用 Tower Layer) |
| `grpc-layer` | tonic, http, tower, pin-project-lite | `GrpcMetricsLayer` (gRPC 用 Tower Layer) |
| `full` | axum-layer + grpc-layer | 上記すべて |

```toml
# Cargo.toml で feature を有効化する例
[dependencies]
k1s0-telemetry = { path = "...", features = ["axum-layer"] }
# または全機能:
k1s0-telemetry = { path = "...", features = ["full"] }
```

### Cargo.toml

```toml
[package]
name = "k1s0-telemetry"
version = "0.1.0"
edition = "2021"

[dependencies]
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"
prometheus = "0.13"

# Tower Layer support (optional)
axum = { version = "0.7", optional = true }
http = { version = "1", optional = true }
tonic = { version = "0.12", optional = true }
tower = { version = "0.5", features = ["util"], optional = true }
pin-project-lite = { version = "0.2", optional = true }

[features]
default = []
axum-layer = ["axum", "http", "tower", "pin-project-lite"]
grpc-layer = ["tonic", "http", "tower", "pin-project-lite"]
full = ["axum-layer", "grpc-layer"]

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
http = "1"
tonic = "0.12"
tower = { version = "0.5", features = ["util"] }
pin-project-lite = "0.2"
```

### 主要コード (lib.rs)

```rust
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{
    fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub struct TelemetryConfig {
    pub service_name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
    pub trace_endpoint: Option<String>,
    pub sample_rate: f64,
    pub log_level: String,
    pub log_format: String,
}

pub fn init_telemetry(cfg: &TelemetryConfig) -> Result<(), Box<dyn std::error::Error>> {
    let tracer = if let Some(ref endpoint) = cfg.trace_endpoint {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        let provider = sdktrace::TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_sampler(sdktrace::Sampler::TraceIdRatioBased(cfg.sample_rate))
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", cfg.service_name.clone()),
                KeyValue::new("service.version", cfg.version.clone()),
                KeyValue::new("tier", cfg.tier.clone()),
                KeyValue::new("environment", cfg.environment.clone()),
            ]))
            .build();
        let tracer = provider.tracer("k1s0");
        global::set_tracer_provider(provider);
        Some(tracer)
    } else {
        None
    };

    let filter = EnvFilter::new(&cfg.log_level);
    let registry = tracing_subscriber::registry().with(filter);

    if cfg.log_format == "text" {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE);
        let subscriber = registry.with(fmt_layer);
        if let Some(t) = tracer {
            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(t);
            subscriber.with(telemetry_layer).init();
        } else {
            subscriber.init();
        }
    } else {
        let fmt_layer = fmt::layer()
            .json()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE);
        let subscriber = registry.with(fmt_layer);
        if let Some(t) = tracer {
            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(t);
            subscriber.with(telemetry_layer).init();
        } else {
            subscriber.init();
        }
    }

    Ok(())
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
```

### init_logger / parse_log_level (logger.rs)

`init_telemetry` とは独立したロガー初期化関数。トレース連携が不要な場合に使用する。

```rust
/// init_logger は環境名に基づいてログレベルを設定し、tracing-subscriber を初期化する。
/// - "dev" -> debug, "staging" -> info, その他 -> warn
/// format が "text" の場合はプレーンテキスト出力、それ以外は JSON 出力。
pub fn init_logger(env: &str, format: &str)

/// parse_log_level はログレベル文字列を tracing::Level に変換する。
/// "debug"/"info"/"warn"/"error" に対応（デフォルト: INFO）。
pub fn parse_log_level(level: &str) -> tracing::Level
```

### 拡張メトリクス（Rust 専用）

Rust の `Metrics` 構造体は共通 4 メトリクス（HTTP/gRPC）に加え、DB・Kafka・キャッシュの 5 メトリクスを提供する。これらは Rust のみの組み込み機能であり、他言語では利用できない。

> アーキテクチャ上の位置づけは [可観測性設計.md](../../architecture/observability/可観測性設計.md) の「カスタムメトリクス」セクションを参照。

| メトリクス名 | 型 | ラベル | 記録メソッド |
|-------------|-----|--------|-------------|
| `db_query_duration_seconds` | HistogramVec | query_name, table | `record_db_query_duration(query_name, table, duration_secs)` |
| `kafka_messages_produced_total` | IntCounterVec | topic | `record_kafka_message_produced(topic)` |
| `kafka_messages_consumed_total` | IntCounterVec | topic, consumer_group | `record_kafka_message_consumed(topic, consumer_group)` |
| `cache_hits_total` | IntCounterVec | cache_name | `record_cache_hit(cache_name)` |
| `cache_misses_total` | IntCounterVec | cache_name | `record_cache_miss(cache_name)` |

使用例:

```rust
let metrics = Metrics::new("order-server");
metrics.record_db_query_duration("find_order", "orders", 0.042);
metrics.record_kafka_message_produced("order.created");
metrics.record_kafka_message_consumed("order.created", "payment-consumer");
metrics.record_cache_hit("order-cache");
metrics.record_cache_miss("user-cache");
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/telemetry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**ファイル構成**:

```
telemetry/src/
├── index.ts            # バレルエクスポート
├── telemetry.ts        # TelemetryConfig, initTelemetry, shutdown
├── logger.ts           # createLogger (pino + OTel mixin)
├── middleware.ts        # httpMiddleware (Express/Fastify)
├── metrics.ts          # Metrics クラス
└── grpcInterceptor.ts  # createGrpcInterceptor
```

**package.json**:

```json
{
  "name": "@k1s0/telemetry",
  "version": "0.1.0",
  "dependencies": {
    "@opentelemetry/api": "^1.9.0",
    "@opentelemetry/sdk-node": "^0.56.0",
    "@opentelemetry/exporter-trace-otlp-grpc": "^0.56.0",
    "pino": "^9.5.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}
```

**主要コード**:

```typescript
// telemetry.ts
import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';

export interface TelemetryConfig {
  serviceName: string;
  version: string;
  tier: string;
  environment: string;
  traceEndpoint?: string;
  sampleRate?: number;
  logLevel: string;
  logFormat?: string;
}

let sdk: NodeSDK | undefined;

export function initTelemetry(cfg: TelemetryConfig): void {
  if (cfg.traceEndpoint) {
    const exporter = new OTLPTraceExporter({ url: cfg.traceEndpoint });
    sdk = new NodeSDK({
      traceExporter: exporter,
      serviceName: cfg.serviceName,
    });
    sdk.start();
  }
}

export function shutdown(): Promise<void> {
  return sdk?.shutdown() ?? Promise.resolve();
}
```

```typescript
// logger.ts
import pino from 'pino';
import { trace } from '@opentelemetry/api';
import type { TelemetryConfig } from './telemetry';

export function createLogger(cfg: TelemetryConfig): pino.Logger {
  const options: pino.LoggerOptions = {
    level: cfg.logLevel,
    base: {
      service: cfg.serviceName,
      version: cfg.version,
      tier: cfg.tier,
      environment: cfg.environment,
    },
    mixin() {
      const span = trace.getActiveSpan();
      if (span) {
        const spanContext = span.spanContext();
        return {
          trace_id: spanContext.traceId,
          span_id: spanContext.spanId,
        };
      }
      return {};
    },
  };

  if (cfg.logFormat === 'text') {
    options.transport = {
      target: 'pino-pretty',
      options: { colorize: true, translateTime: 'SYS:standard' },
    };
  }

  return pino(options);
}
```

**トレース相関の注意**: TypeScript の `createLogger` は pino の `mixin()` オプションを使用して、アクティブな OpenTelemetry スパンから `trace_id` / `span_id` を自動注入する。Go のように明示的に `LogWithTrace` を呼ぶ必要はない。

## Dart 実装

**配置先**: `regions/system/library/dart/telemetry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**ファイル構成**:

```
telemetry/lib/
├── telemetry.dart      # バレルエクスポート
└── src/
    ├── init.dart       # TelemetryConfig, initTelemetry, shutdown
    ├── logger.dart     # createLogger
    ├── middleware.dart  # TelemetryMiddleware (shelf)
    └── metrics.dart    # Metrics, HistogramData 等
```

**pubspec.yaml**:

```yaml
name: k1s0_telemetry
version: 0.1.0
description: k1s0 platform telemetry library for structured logging and observability.

environment:
  sdk: ">=3.4.0 <4.0.0"

dependencies:
  logging: ^1.2.0
  http: ^1.2.0
  shelf: ^1.4.0

dev_dependencies:
  test: ^1.25.0
  lints: ^5.0.0
```

**主要コード**:

```dart
import 'dart:convert';
import 'package:logging/logging.dart';

class TelemetryConfig {
  final String serviceName;
  final String version;
  final String tier;
  final String environment;
  final String? traceEndpoint;
  final double sampleRate;
  final String logLevel;
  final String logFormat;

  TelemetryConfig({
    required this.serviceName,
    required this.version,
    required this.tier,
    required this.environment,
    this.traceEndpoint,
    this.sampleRate = 1.0,
    this.logLevel = 'info',
    this.logFormat = 'json',
  });
}

void initTelemetry(TelemetryConfig cfg) {
  Logger.root.level = _parseLevel(cfg.logLevel);
  Logger.root.onRecord.listen((record) {
    if (cfg.logFormat == 'text') {
      print(
          '${record.time} [${record.level.name}] ${record.loggerName}: ${record.message}');
      if (record.error != null) {
        print('  Error: ${record.error}');
      }
    } else {
      final entry = {
        'timestamp': record.time.toUtc().toIso8601String(),
        'level': record.level.name.toLowerCase(),
        'message': record.message,
        'service': cfg.serviceName,
        'version': cfg.version,
        'tier': cfg.tier,
        'environment': cfg.environment,
        'logger': record.loggerName,
      };
      if (record.error != null) entry['error'] = record.error.toString();
      print(jsonEncode(entry));
    }
  });
}

void shutdown() {
  Logger.root.clearListeners();
}

/// createLogger は名前を指定して Logger を生成する。
/// 注意: Go/Rust/TypeScript と異なり、TelemetryConfig ではなく name のみを受け取る。
Logger createLogger(String name) => Logger(name);
```

**Dart 固有の注意点**:
- `sampleRate` のデフォルト値は `1.0`（他言語はゼロ値）
- `logLevel` のデフォルト値は `'info'`（他言語はゼロ値 / required）
- `logFormat` のデフォルト値は `'json'`（他言語はゼロ値 / optional）
- `createLogger` は `TelemetryConfig` ではなく `String name` のみを引数に取る
- OpenTelemetry トレース連携は未実装（ミドルウェアで独自の `x-trace-id` ヘッダを使用）

## ミドルウェア

各言語は HTTP / gRPC のリクエスト計測用ミドルウェアを提供する。

### Go

```go
// HTTPMiddleware は HTTP リクエストの分散トレーシングと構造化ログを提供する。
// リクエストごとに OTel スパン（tracer: "k1s0-http"）を生成し、
// method/path/status/duration を LogWithTrace 経由でログ出力する。
func HTTPMiddleware(logger *slog.Logger) func(http.Handler) http.Handler

// GRPCUnaryInterceptor は gRPC Unary RPC のトレーシングとログを提供する。
// リクエストごとに OTel スパン（tracer: "k1s0-grpc"）を生成し、
// method/duration をログ出力する。エラー時は error フィールドも記録する。
func GRPCUnaryInterceptor(logger *slog.Logger) func(
    ctx context.Context,
    method string,
    req, reply interface{},
    invoker func(ctx context.Context, method string, req, reply interface{}) error,
) error
```

使用例:

```go
logger := provider.Logger()
mux := http.NewServeMux()
handler := HTTPMiddleware(logger)(mux)
```

### Rust

Rust は用途に応じて 2 段階のミドルウェアを提供する。

**手動計測用の構造体** (`middleware/mod.rs`):

```rust
/// TelemetryMiddleware は HTTP リクエストの分散トレーシングとメトリクス記録を提供する。
pub struct TelemetryMiddleware {
    pub metrics: Arc<Metrics>,
}

impl TelemetryMiddleware {
    pub fn new(metrics: Arc<Metrics>) -> Self
    pub fn on_request(&self, method: &str, path: &str)
    pub fn on_response(&self, method: &str, path: &str, status: u16, duration_secs: f64)
}

/// GrpcInterceptor は gRPC Unary RPC のトレーシングとメトリクス記録を提供する。
pub struct GrpcInterceptor {
    pub metrics: Arc<Metrics>,
}

impl GrpcInterceptor {
    pub fn new(metrics: Arc<Metrics>) -> Self
    pub fn on_request(&self, service: &str, method: &str)
    pub fn on_response(&self, service: &str, method: &str, code: &str, duration_secs: f64)
}
```

**Tower Layer による自動計測** (feature flag 必要):

```rust
// feature: axum-layer -- axum Router に適用する Tower Layer
// HTTP method/path/status/duration を自動記録する。
pub struct MetricsLayer { /* ... */ }
impl MetricsLayer {
    pub fn new(metrics: Arc<Metrics>) -> Self
}

// 使用例:
let app = Router::new()
    .route("/healthz", get(healthz))
    .layer(MetricsLayer::new(metrics.clone()));
```

```rust
// feature: grpc-layer -- tonic Server に適用する Tower Layer
// URI パスから gRPC service/method を抽出し、grpc-status ヘッダからステータスコードを取得する。
pub struct GrpcMetricsLayer { /* ... */ }
impl GrpcMetricsLayer {
    pub fn new(metrics: Arc<Metrics>) -> Self
}

// 使用例:
tonic::transport::Server::builder()
    .layer(GrpcMetricsLayer::new(metrics.clone()))
    .add_service(my_service)
    .serve(addr)
    .await?;
```

**マクロ** (`middleware/mod.rs`):

```rust
/// trace_request! はハンドラ本体にトレーシングスパンとタイミング計測を付与する。
trace_request!("GET", "/health", { /* handler body */ });

/// trace_grpc_call! は gRPC メソッド呼び出しにトレーシングスパンとタイミング計測を付与する。
trace_grpc_call!("OrderService.CreateOrder", { /* call body */ });
```

### TypeScript

```typescript
// httpMiddleware は Express/Fastify 互換の HTTP ミドルウェアを返す。
// res.finish イベントで method/path/status/duration_ms をログ出力する。
// 注意: OTel スパンは生成しない（ログ出力のみ）。
export function httpMiddleware(logger: pino.Logger):
  (req: IncomingMessage, res: ServerResponse, next: () => void) => void

// createGrpcInterceptor は gRPC unary interceptor を返す。
// OTel スパン（tracer: "k1s0-grpc"）を生成し、Metrics にリクエスト数と duration を記録する。
export function createGrpcInterceptor(metrics: Metrics): GrpcInterceptorFn

export type GrpcInvoker<TReq, TRes> = (req: TReq) => Promise<TRes>;
export type GrpcInterceptorFn = <TReq, TRes>(
  method: string,
  request: TReq,
  invoker: GrpcInvoker<TReq, TRes>,
) => Promise<TRes>;
```

使用例:

```typescript
const logger = createLogger(cfg);
app.use(httpMiddleware(logger));

const metrics = new Metrics('order-server');
const interceptor = createGrpcInterceptor(metrics);
```

### Dart

Dart は `shelf` パッケージを使用した HTTP ミドルウェアを提供する。gRPC インターセプタは未実装。

```dart
/// shelf の Middleware として HTTP リクエストの構造化ログを提供する。
/// - x-trace-id ヘッダの抽出/生成（OTel ではなく独自トレース相関）
/// - method/path/statusCode/duration のログ出力
/// - 5xx でログレベルを warning に変更
/// - 例外キャッチ時は 500 レスポンスを返却
class TelemetryMiddleware {
  TelemetryMiddleware({required Logger logger})
  Middleware get middleware
}
```

使用例:

```dart
final logger = createLogger('MyServer');
final telemetry = TelemetryMiddleware(logger: logger);

final handler = const Pipeline()
    .addMiddleware(telemetry.middleware)
    .addHandler(router);
```

## 共通メトリクス

全言語で以下の 4 メトリクスを提供する（RED メソッド）。

| メトリクス名 | 型 | ラベル |
|-------------|-----|--------|
| `http_requests_total` | Counter | method, path, status |
| `http_request_duration_seconds` | Histogram | method, path |
| `grpc_server_handled_total` | Counter | grpc_service, grpc_method, grpc_code |
| `grpc_server_handling_seconds` | Histogram | grpc_service, grpc_method |

全メトリクスに `service` ラベル（定数）が付与される。ヒストグラムのバケット境界は全言語共通: `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]`。

メトリクス出力メソッドの命名は言語ごとに異なる:

| Go | Rust | TypeScript | Dart |
|:--:|:----:|:----------:|:----:|
| `MetricsHandler()` -> `http.Handler` | `gather_metrics()` -> `String` | `getMetrics()` -> `string` | `toPrometheusText()` -> `String` |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) -- ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) -- config ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) -- authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) -- k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) -- k1s0-kafka ライブラリ
- [system-library-correlation設計](correlation.md) -- k1s0-correlation ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) -- k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) -- k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](../auth-security/serviceauth.md) -- k1s0-serviceauth ライブラリ

---
