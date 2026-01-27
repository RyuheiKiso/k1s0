# k1s0_observability

Observability library for k1s0 Flutter applications with structured logging, tracing, error tracking, and performance measurement.

## Features

- Structured logging with required fields (timestamp, level, service_name, env, trace_id, span_id)
- Distributed tracing with W3C Trace Context support
- Error tracking with stack traces
- Performance metrics collection
- Riverpod-based state management
- Pluggable exporters (console, remote)

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  k1s0_observability:
    path: ../packages/k1s0_observability
```

## Basic Usage

### Setup with Riverpod

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:k1s0_observability/k1s0_observability.dart';

void main() {
  runApp(
    ProviderScope(
      overrides: [
        observabilityConfigProvider.overrideWithValue(
          const ObservabilityConfig(
            serviceName: 'my-app',
            env: 'dev',
            logLevel: LogLevel.debug,
          ),
        ),
      ],
      child: const ObservabilityScope(
        child: MyApp(),
      ),
    ),
  );
}
```

### Logging

```dart
class MyWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Using extension methods
    ref.logInfo('Widget built');
    ref.logDebug('Debug info', {'extra': 'data'});
    ref.logWarn('Warning message');
    ref.logError(
      'Error occurred',
      error: exception,
      stackTrace: stackTrace,
    );

    // Using logger directly
    final logger = ref.watch(loggerProvider);
    logger.info('Using logger directly');

    return Container();
  }
}
```

### Tracing

```dart
class ApiService {
  final Tracer tracer;

  Future<Data> fetchData() async {
    // Automatic span management
    return tracer.trace('fetchData', (span) async {
      span.setAttribute('url', 'https://api.example.com/data');

      final response = await http.get(...);

      span.setAttribute('status_code', response.statusCode);
      return Data.fromJson(response.body);
    });
  }

  // Manual span management
  Future<void> processItem(Item item) async {
    final span = tracer.startSpan('processItem');
    span.setAttribute('item_id', item.id);

    try {
      await doWork(item);
      span.setOk();
    } catch (e, st) {
      span.setError(e.toString(), e, st);
      rethrow;
    } finally {
      span.end();
    }
  }
}
```

### Error Tracking

```dart
class MyWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ElevatedButton(
      onPressed: () async {
        try {
          await riskyOperation();
        } catch (e, st) {
          ref.trackError(
            e,
            stackTrace: st,
            severity: ErrorSeverity.high,
            context: {'operation': 'riskyOperation'},
          );
        }
      },
      child: const Text('Do Something'),
    );
  }
}

// Global error handling is set up automatically
// when enableErrorTracking is true in config
```

### Metrics

```dart
class MyWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final metrics = ref.watch(metricsProvider);

    return ElevatedButton(
      onPressed: () async {
        // Measure async operation
        final result = await metrics.measureAsync(
          'button_click_operation',
          () async {
            await doWork();
          },
          tags: {'button': 'submit'},
        );

        // Record custom metrics
        metrics.recordTiming('custom_timing', 150.0);
        metrics.recordCount('button_clicks');
        metrics.recordGauge('queue_size', 42);
      },
      child: const Text('Click Me'),
    );
  }
}
```

### Trace Context Propagation

```dart
// Get headers for outgoing requests
final context = tracer.currentContext;
final headers = context.toHeaders();
// Contains: traceparent, x-trace-id, x-span-id, x-request-id

// Parse incoming trace context
final incomingContext = TraceContext.fromTraceparent(
  request.headers['traceparent']!,
);
tracer.currentContext = incomingContext;
```

## Configuration

```dart
ObservabilityConfig(
  // Required
  serviceName: 'my-app',
  env: 'dev',  // dev, stg, prod

  // Optional
  version: '1.0.0',
  logLevel: LogLevel.info,
  enableConsole: true,
  enableTracing: true,
  enableMetrics: true,
  enableErrorTracking: true,
  tracingSampleRate: 1.0,  // 0.0 - 1.0
  batchSize: 50,
  flushIntervalSeconds: 10,
)
```

## Log Entry Format

All log entries include these required fields:

```json
{
  "timestamp": "2024-01-01T12:00:00.000Z",
  "level": "INFO",
  "message": "User logged in",
  "service_name": "my-app",
  "env": "prod",
  "trace_id": "abc123...",
  "span_id": "def456...",
  "request_id": "req-789",
  "extra": {}
}
```

## Providers

| Provider | Type | Description |
|----------|------|-------------|
| `observabilityConfigProvider` | `ObservabilityConfig` | Configuration |
| `observabilityServiceProvider` | `ObservabilityService` | Main service |
| `loggerProvider` | `Logger` | Logger instance |
| `tracerProvider` | `Tracer` | Tracer instance |
| `metricsProvider` | `MetricsCollector` | Metrics collector |
| `errorTrackerProvider` | `ErrorTracker` | Error tracker |

## Custom Log Sinks

```dart
// Console sink
final consoleSink = ConsoleLogSink(
  useColors: true,
  prettyPrint: true,
);

// Buffered sink
final bufferedSink = BufferedLogSink(
  delegate: consoleSink,
  bufferSize: 100,
  flushInterval: Duration(seconds: 5),
);

// Composite sink (multiple destinations)
final compositeSink = CompositeLogSink([
  ConsoleLogSink(),
  RemoteLogSink(endpoint: 'https://logs.example.com'),
]);
```

## Custom Span Exporters

```dart
// Console exporter (default)
final exporter = ConsoleSpanExporter();

// Buffered exporter
final bufferedExporter = BufferedSpanExporter(
  delegate: ConsoleSpanExporter(),
  batchSize: 50,
  flushInterval: Duration(seconds: 10),
);

// Custom exporter
class MyExporter implements SpanExporter {
  @override
  Future<void> export(List<SpanInfo> spans) async {
    // Send to your backend
  }

  @override
  Future<void> shutdown() async {}
}
```

## Testing

```dart
// Create a logger for testing
final logger = LoggerFactory.createConsole(
  serviceName: 'test-service',
  env: 'test',
  minLevel: LogLevel.debug,
);

// Create a tracer for testing
final tracer = TracerFactory.createConsole(
  serviceName: 'test-service',
);

// Create metrics collector without exporter
final metrics = MetricsCollector();
```

## License

MIT
