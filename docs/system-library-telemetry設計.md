# telemetry ライブラリ設計

> 詳細な設計方針は [可観測性設計.md](可観測性設計.md) を参照。

## 公開 API（全言語共通契約）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| InitTelemetry | `(config) -> Provider` | OpenTelemetry 初期化（トレース + メトリクス） |
| Shutdown | `() -> void` | プロバイダーのシャットダウン |
| NewLogger | `(config) -> Logger` | 構造化ログのロガー生成 |

## Go 実装

**配置先**: `regions/system/library/go/telemetry/`

```
telemetry/
├── telemetry.go       # InitTelemetry, Shutdown
├── logger.go          # NewLogger, LogWithTrace
├── metrics.go         # Prometheus メトリクス（RED メソッド: request_total, request_duration, request_errors, request_in_flight）
├── middleware.go      # gin HTTP middleware + gRPC interceptor（リクエストログ・duration計測・メトリクス記録）
├── telemetry_test.go
├── go.mod
└── go.sum
```

**依存関係**:

```
go.opentelemetry.io/otel
go.opentelemetry.io/otel/sdk
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc
go.opentelemetry.io/otel/exporters/prometheus
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
    ServiceName string
    Version     string
    Tier        string
    Environment string
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

    handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
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

## Rust 実装

**配置先**: `regions/system/library/rust/telemetry/`

```
telemetry/
├── src/
│   ├── lib.rs           # 公開 API（init_telemetry, shutdown）
│   ├── metrics.rs       # Prometheus メトリクス（prometheus クレート使用、Go の RED メソッドと同等の4メトリクス）
│   └── middleware.rs    # axum HTTP middleware + tonic gRPC interceptor
├── tests/
│   └── integration/
│       └── telemetry_test.rs
└── Cargo.toml
```

**Cargo.toml**:

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
```

**主要コード**:

```rust
use opentelemetry::global;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct TelemetryConfig {
    pub service_name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
    pub trace_endpoint: Option<String>,
    pub sample_rate: f64,
    pub log_level: String,
}

pub fn init_telemetry(cfg: &TelemetryConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref endpoint) = cfg.trace_endpoint {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;
        let provider = sdktrace::TracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_sampler(sdktrace::Sampler::TraceIdRatioBased(cfg.sample_rate))
            .with_resource(Resource::builder()
                .with_service_name(&cfg.service_name)
                .build())
            .build();
        global::set_tracer_provider(provider);
    }

    let filter = EnvFilter::new(&cfg.log_level);
    let fmt_layer = fmt::layer().json().with_target(true);
    let telemetry_layer = cfg.trace_endpoint.as_ref().map(|_| {
        tracing_opentelemetry::layer().with_tracer(global::tracer("k1s0"))
    });

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);

    if let Some(tl) = telemetry_layer {
        subscriber.with(tl).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

pub fn shutdown() {
    global::shutdown_tracer_provider();
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/telemetry/`

```
telemetry/
├── src/
│   ├── index.ts             # 公開 API エクスポート
│   ├── telemetry.ts         # initTelemetry, shutdown, createLogger
│   ├── metrics.ts           # prom-client ベースのメトリクス収集
│   └── grpcInterceptor.ts   # gRPC interceptor（リクエストログ・duration計測）
├── tests/
│   └── unit/
│       └── telemetry.test.ts
├── package.json
└── tsconfig.json
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
import { NodeSDK } from '@opentelemetry/sdk-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import pino from 'pino';

export interface TelemetryConfig {
  serviceName: string;
  version: string;
  tier: string;
  environment: string;
  traceEndpoint?: string;
  sampleRate?: number;
  logLevel: string;
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

export function createLogger(cfg: TelemetryConfig): pino.Logger {
  return pino({
    level: cfg.logLevel,
    base: {
      service: cfg.serviceName,
      version: cfg.version,
      tier: cfg.tier,
      environment: cfg.environment,
    },
  });
}
```

## Dart 実装

**配置先**: `regions/system/library/dart/telemetry/`

```
telemetry/
├── lib/
│   ├── telemetry.dart       # エントリーポイント
│   └── src/
│       ├── telemetry.dart   # initTelemetry, createLogger
│       ├── metrics.dart     # メトリクス収集
│       └── middleware.dart  # HTTP middleware（リクエストログ・duration計測）
├── test/
│   └── unit/
│       └── telemetry_test.dart
├── pubspec.yaml
└── analysis_options.yaml
```

**pubspec.yaml**:

```yaml
name: k1s0_telemetry
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  logging: ^1.2.0
  http: ^1.2.0
dev_dependencies:
  test: ^1.25.0
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

  TelemetryConfig({
    required this.serviceName,
    required this.version,
    required this.tier,
    required this.environment,
    this.traceEndpoint,
    this.sampleRate = 1.0,
    this.logLevel = 'info',
  });
}

void initTelemetry(TelemetryConfig cfg) {
  Logger.root.level = _parseLevel(cfg.logLevel);
  Logger.root.onRecord.listen((record) {
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
  });
}

Logger createLogger(String name) => Logger(name);

Level _parseLevel(String level) {
  switch (level) {
    case 'debug': return Level.FINE;
    case 'info': return Level.INFO;
    case 'warn': return Level.WARNING;
    case 'error': return Level.SEVERE;
    default: return Level.INFO;
  }
}
```

## C# 実装

**配置先**: `regions/system/library/csharp/telemetry/`

```
telemetry/
├── src/
│   ├── Telemetry.csproj
│   ├── TelemetryInitializer.cs     # OTel トレース + メトリクス初期化
│   ├── Metrics.cs                  # RED メソッドメトリクス定義
│   ├── Middleware/
│   │   ├── HttpTelemetryMiddleware.cs  # ASP.NET Core HTTP ミドルウェア
│   │   └── GrpcTelemetryInterceptor.cs # gRPC インターセプター
│   └── TelemetryException.cs       # 公開例外型
├── tests/
│   ├── Telemetry.Tests.csproj
│   ├── Unit/
│   │   └── MetricsTests.cs
│   └── Integration/
│       └── TelemetryInitializerTests.cs
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
| OpenTelemetry.Instrumentation.GrpcNetClient | gRPC 自動計測 |
| Serilog | 構造化ログ |
| Serilog.Extensions.Hosting | ホスティング統合 |
| Serilog.Sinks.Console | コンソール出力 |

**名前空間**: `K1s0.System.Telemetry`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `TelemetryInitializer` | static class | OTel トレース + メトリクス + ログの初期化 |
| `Metrics` | static class | RED メソッドメトリクス（request_total Counter, request_duration Histogram, request_errors Counter, request_in_flight UpDownCounter） |
| `HttpTelemetryMiddleware` | class | ASP.NET Core HTTP リクエスト計測ミドルウェア |
| `GrpcTelemetryInterceptor` | class | gRPC サーバーインターセプター |

**主要 API**:

```csharp
namespace K1s0.System.Telemetry;

public static class TelemetryInitializer
{
    public static IServiceCollection AddK1s0Telemetry(
        this IServiceCollection services,
        TelemetryConfig config,
        CancellationToken cancellationToken = default);

    public static IApplicationBuilder UseK1s0Telemetry(
        this IApplicationBuilder app);
}

public static class Metrics
{
    public static readonly Counter<long> RequestTotal;
    public static readonly Histogram<double> RequestDuration;
    public static readonly Counter<long> RequestErrors;
    public static readonly UpDownCounter<long> RequestInFlight;
}
```

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
- [可観測性設計](可観測性設計.md) — OpenTelemetry・構造化ログ・メトリクス
