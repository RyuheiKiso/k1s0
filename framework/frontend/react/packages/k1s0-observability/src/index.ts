// Types
export type {
  LogLevel,
  ObservabilityConfig,
  RequiredLogFields,
  LogEntry,
  SpanInfo,
  SpanStatus,
  ErrorInfo,
  PerformanceMetric,
  WebVitals,
  LogSink,
  TraceExporter,
  MetricsExporter,
  ObservabilityContext,
} from './types.js';

export { ObservabilityConfigSchema, LOG_LEVEL_PRIORITY } from './types.js';

// Utils
export {
  generateTraceId,
  generateSpanId,
  generateRequestId,
  generateTimestamp,
  generateTraceparent,
  parseTraceparent,
} from './utils/index.js';

// Tracing
export { TracingService, SpanBuilder } from './tracing/index.js';

// Logging
export { Logger, ConsoleLogSink, BufferedLogSink } from './logging/index.js';

// Metrics
export { MetricsCollector, type MetricsListener } from './metrics/index.js';

// Errors
export {
  ErrorTracker,
  type ErrorEvent,
  type ErrorListener,
} from './errors/index.js';

// Provider
export {
  ObservabilityProvider,
  useObservability,
  useTracing,
  useLogger,
  useMetrics,
  useErrorTracker,
  useSpan,
  useTraceContext,
  type ObservabilityContextValue,
} from './provider/index.js';
