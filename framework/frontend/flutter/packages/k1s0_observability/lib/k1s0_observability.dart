/// k1s0 Observability Library
///
/// Provides structured logging, tracing, error tracking, and performance
/// measurement for k1s0 Flutter applications.
library k1s0_observability;

export 'src/config/observability_config.dart';
export 'src/error/error_info.dart';
export 'src/error/error_tracker.dart';
export 'src/logging/log_entry.dart';
export 'src/logging/log_level.dart';
export 'src/logging/log_sink.dart';
export 'src/logging/logger.dart';
export 'src/metrics/metrics_collector.dart';
export 'src/metrics/performance_metric.dart';
export 'src/provider/observability_provider.dart';
export 'src/tracing/span.dart';
export 'src/tracing/trace_context.dart';
export 'src/tracing/tracer.dart';
export 'src/utils/id_generator.dart';
