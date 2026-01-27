# @k1s0/observability

React アプリケーション向けの観測性ライブラリ。

## 概要

OpenTelemetry 統合、構造化ログ、エラートラッキング、パフォーマンス計測を提供します。バックエンドの `k1s0-observability` crate と連携して動作することを想定しています。

### 主な機能

- **OpenTelemetry 統合**: W3C Trace Context、スパン管理
- **構造化ログ**: 必須フィールドの自動付与（timestamp, level, service_name, env, trace_id, span_id）
- **エラートラッキング**: グローバルエラーハンドリング、React Error Boundary 統合
- **パフォーマンス計測**: Web Vitals 自動収集、カスタムメトリクス
- **trace_id/span_id 相関**: ログ、エラー、メトリクスの相関付け

## インストール

```bash
pnpm add @k1s0/observability
```

### Peer Dependencies

```bash
pnpm add react react-dom @opentelemetry/api
```

### Optional Dependencies（OpenTelemetry 統合）

```bash
pnpm add @opentelemetry/sdk-trace-web @opentelemetry/exporter-trace-otlp-http
```

## 使用方法

### 基本セットアップ

```tsx
import { ObservabilityProvider } from '@k1s0/observability';

function App() {
  return (
    <ObservabilityProvider
      config={{
        serviceName: 'my-frontend',
        env: 'dev',
        version: '1.0.0',
        logLevel: 'INFO',
        samplingRate: 1.0,
      }}
      enableGlobalErrorHandling={true}
      enableWebVitals={true}
      onError={(event) => {
        // エラー発生時の処理（外部サービスへの送信等）
        console.error('Error captured:', event.error);
      }}
    >
      <YourApp />
    </ObservabilityProvider>
  );
}
```

### ロギング

```tsx
import { useLogger } from '@k1s0/observability';

function MyComponent() {
  const logger = useLogger();

  const handleClick = () => {
    logger.info('Button clicked', { buttonId: 'submit' });
  };

  const handleError = () => {
    try {
      // 処理
    } catch (error) {
      logger.error('Operation failed', error, { operation: 'submit' });
    }
  };

  return (
    <button onClick={handleClick}>Submit</button>
  );
}
```

### 出力されるログ形式

```json
{
  "timestamp": "2026-01-27T10:00:00.000Z",
  "level": "INFO",
  "message": "Button clicked",
  "service_name": "my-frontend",
  "env": "dev",
  "trace_id": "abc123...",
  "span_id": "def456...",
  "buttonId": "submit"
}
```

### トレーシング

```tsx
import { useTracing, useSpan, useTraceContext } from '@k1s0/observability';

function MyComponent() {
  const tracing = useTracing();
  const { traceId, traceparent } = useTraceContext();

  const handleSubmit = async () => {
    // スパンを使用した計測
    await tracing.withSpan('submit_form', async (span) => {
      span.setAttribute('form.type', 'registration');

      // API 呼び出し時に traceparent を付与
      const response = await fetch('/api/submit', {
        headers: {
          'traceparent': traceparent ?? '',
        },
      });

      span.setAttribute('response.status', response.status);
      return response.json();
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      {/* フォームコンテンツ */}
    </form>
  );
}
```

### エラートラッキング

```tsx
import { useErrorTracker } from '@k1s0/observability';

function MyComponent() {
  const errorTracker = useErrorTracker();

  const handleClick = () => {
    try {
      // 処理
    } catch (error) {
      // エラーをキャプチャ
      errorTracker.captureException(error, {
        component: 'MyComponent',
        action: 'handleClick',
      });
    }
  };

  return <button onClick={handleClick}>Click</button>;
}
```

### React Error Boundary 統合

```tsx
import React from 'react';
import { useErrorTracker } from '@k1s0/observability';

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  static contextType = React.createContext<ReturnType<typeof useErrorTracker> | null>(null);

  state = { hasError: false };

  static getDerivedStateFromError() {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    // ObservabilityProvider から ErrorTracker を取得して使用
    const errorTracker = this.context;
    if (errorTracker) {
      errorTracker.captureError(error, {
        componentStack: errorInfo.componentStack,
        type: 'react_error_boundary',
      });
    }
  }

  render() {
    if (this.state.hasError) {
      return <div>Something went wrong.</div>;
    }
    return this.props.children;
  }
}
```

### メトリクス

```tsx
import { useMetrics } from '@k1s0/observability';

function MyComponent() {
  const metrics = useMetrics();

  const handleClick = async () => {
    // 処理時間を計測
    const result = await metrics.measureTimeAsync('api_call', async () => {
      return fetch('/api/data').then(r => r.json());
    }, { endpoint: '/api/data' });

    // カウンターをインクリメント
    metrics.counter('button_clicks', 1, { button: 'submit' });
  };

  return <button onClick={handleClick}>Fetch Data</button>;
}
```

### Web Vitals

```tsx
import { useMetrics, type WebVitals } from '@k1s0/observability';

function WebVitalsDisplay() {
  const metrics = useMetrics();
  const [vitals, setVitals] = React.useState<Partial<WebVitals>>({});

  React.useEffect(() => {
    // 定期的に Web Vitals を更新
    const interval = setInterval(() => {
      setVitals(metrics.getWebVitals());
    }, 1000);

    return () => clearInterval(interval);
  }, [metrics]);

  return (
    <div>
      <p>LCP: {vitals.LCP?.toFixed(0)}ms</p>
      <p>FID: {vitals.FID?.toFixed(0)}ms</p>
      <p>CLS: {vitals.CLS?.toFixed(3)}</p>
      <p>FCP: {vitals.FCP?.toFixed(0)}ms</p>
      <p>TTFB: {vitals.TTFB?.toFixed(0)}ms</p>
    </div>
  );
}
```

### OpenTelemetry 統合

```tsx
import { trace } from '@opentelemetry/api';
import {
  WebTracerProvider,
  BatchSpanProcessor,
} from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { ObservabilityProvider, useTracing } from '@k1s0/observability';

// OpenTelemetry の初期化
const provider = new WebTracerProvider();
const exporter = new OTLPTraceExporter({
  url: 'https://otel-collector.example.com/v1/traces',
});
provider.addSpanProcessor(new BatchSpanProcessor(exporter));
provider.register();

function App() {
  return (
    <ObservabilityProvider
      config={{
        serviceName: 'my-frontend',
        env: 'prod',
        otlpEndpoint: 'https://otel-collector.example.com',
      }}
    >
      <OTelSetup />
      <YourApp />
    </ObservabilityProvider>
  );
}

// OTel をトレーシングサービスに設定
function OTelSetup() {
  const tracing = useTracing();

  React.useEffect(() => {
    const otelApi = { trace, context: require('@opentelemetry/api').context };
    tracing.setOpenTelemetry(otelApi);
  }, [tracing]);

  return null;
}
```

### カスタムログシンク

```tsx
import {
  ObservabilityProvider,
  useLogger,
  BufferedLogSink,
  type LogEntry,
} from '@k1s0/observability';

// バッファリングして外部サービスに送信
const customSink = new BufferedLogSink({
  maxSize: 50,
  flushIntervalMs: 10000,
  onFlush: async (entries) => {
    await fetch('/api/logs', {
      method: 'POST',
      body: JSON.stringify(entries),
    });
  },
});

function App() {
  return (
    <ObservabilityProvider config={{ serviceName: 'my-app', env: 'prod' }}>
      <LogSinkSetup />
      <YourApp />
    </ObservabilityProvider>
  );
}

function LogSinkSetup() {
  const logger = useLogger();

  React.useEffect(() => {
    logger.addSink(customSink);
    return () => {
      logger.removeSink(customSink);
    };
  }, [logger]);

  return null;
}
```

## API リファレンス

### ObservabilityProvider

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `config.serviceName` | `string` | (必須) | サービス名 |
| `config.env` | `'dev' \| 'stg' \| 'prod'` | (必須) | 環境名 |
| `config.version` | `string` | - | サービスバージョン |
| `config.otlpEndpoint` | `string` | - | OTLP エンドポイント |
| `config.samplingRate` | `number` | `1.0` | サンプリングレート（0.0-1.0） |
| `config.logLevel` | `LogLevel` | `'INFO'` | 最小ログレベル |
| `config.enableConsole` | `boolean` | `true` | コンソール出力 |
| `config.enableBatching` | `boolean` | `true` | バッチ送信 |
| `config.batchSize` | `number` | `100` | バッチサイズ |
| `config.batchIntervalMs` | `number` | `5000` | バッチ送信間隔 |
| `enableGlobalErrorHandling` | `boolean` | `true` | グローバルエラーハンドリング |
| `enableWebVitals` | `boolean` | `true` | Web Vitals 計測 |
| `onSpan` | `(span: SpanInfo) => void` | - | スパン完了時のコールバック |
| `onError` | `(event: ErrorEvent) => void` | - | エラー発生時のコールバック |
| `onMetric` | `(metric: PerformanceMetric) => void` | - | メトリクス記録時のコールバック |

### useLogger

```ts
interface Logger {
  debug(message: string, fields?: Record<string, unknown>): void;
  info(message: string, fields?: Record<string, unknown>): void;
  warn(message: string, fields?: Record<string, unknown>): void;
  error(message: string, error?: Error | unknown, fields?: Record<string, unknown>): void;
  addSink(sink: LogSink): void;
  removeSink(sink: LogSink): void;
  setMinLevel(level: LogLevel): void;
  flush(): Promise<void>;
}
```

### useTracing

```ts
interface TracingService {
  startSpan(name: string): SpanBuilder;
  withSpan<T>(name: string, fn: (span: SpanBuilder) => T | Promise<T>): Promise<T>;
  getCurrentTraceId(): string | undefined;
  getCurrentSpanId(): string | undefined;
  getTraceparent(): string | null;
  startContext(requestId?: string): ObservabilityContext;
  setContext(traceparent: string, requestId?: string): ObservabilityContext | null;
}
```

### useMetrics

```ts
interface MetricsCollector {
  counter(name: string, value?: number, tags?: Record<string, string>): void;
  timing(name: string, durationMs: number, tags?: Record<string, string>): void;
  size(name: string, bytes: number, tags?: Record<string, string>): void;
  percentage(name: string, percent: number, tags?: Record<string, string>): void;
  measureTime<T>(name: string, fn: () => T, tags?: Record<string, string>): T;
  measureTimeAsync<T>(name: string, fn: () => Promise<T>, tags?: Record<string, string>): Promise<T>;
  recordWebVital(name: keyof WebVitals, value: number): void;
  getWebVitals(): Partial<WebVitals>;
}
```

### useErrorTracker

```ts
interface ErrorTracker {
  captureError(error: Error, context?: Record<string, unknown>): ErrorEvent;
  captureException(error: unknown, context?: Record<string, unknown>): ErrorEvent;
  captureMessage(message: string, level?: 'error' | 'warning', context?: Record<string, unknown>): ErrorEvent;
  createErrorBoundaryHandler(componentName?: string): (error: Error, errorInfo: { componentStack?: string }) => void;
  enableGlobalHandling(): () => void;
}
```

## 必須ログフィールド

observability.md の規約に準拠した必須フィールド：

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（DEBUG/INFO/WARN/ERROR） |
| `service_name` | サービス名 |
| `env` | 環境名（dev/stg/prod） |
| `trace_id` | トレースID（リクエスト相関用） |
| `span_id` | スパンID |

## 設計原則

1. **OTel 互換**: OpenTelemetry Web SDK と統合可能
2. **フォールバック**: OTel なしでも動作
3. **trace_id 一貫性**: ログ、エラー、メトリクスすべてに trace_id を付与
4. **構造化**: JSON 形式の構造化ログ
5. **バッチ処理**: 効率的なデータ送信
