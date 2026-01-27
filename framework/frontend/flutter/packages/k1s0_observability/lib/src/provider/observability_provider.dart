import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config/observability_config.dart';
import '../error/error_tracker.dart';
import '../logging/log_level.dart';
import '../logging/log_sink.dart';
import '../logging/logger.dart';
import '../metrics/metrics_collector.dart';
import '../tracing/tracer.dart';

/// Observability service containing logger, tracer, and metrics
class ObservabilityService {
  /// Creates an observability service
  ObservabilityService({
    required this.config,
    required this.logger,
    required this.tracer,
    required this.metrics,
    required this.errorTracker,
  });

  /// Configuration
  final ObservabilityConfig config;

  /// Logger
  final Logger logger;

  /// Tracer
  final Tracer tracer;

  /// Metrics collector
  final MetricsCollector metrics;

  /// Error tracker
  final ErrorTracker errorTracker;

  /// Dispose all resources
  Future<void> dispose() async {
    logger.dispose();
    await tracer.shutdown();
    await metrics.dispose();
  }
}

/// Provider for observability configuration
final observabilityConfigProvider = Provider<ObservabilityConfig>((ref) {
  // Override this in your app with actual configuration
  return const ObservabilityConfig(
    serviceName: 'k1s0-flutter',
    env: 'dev',
  );
});

/// Provider for observability service
final observabilityServiceProvider = Provider<ObservabilityService>((ref) {
  final config = ref.watch(observabilityConfigProvider);

  // Create log sink
  LogSink logSink;
  if (config.enableConsole) {
    logSink = ConsoleLogSink(prettyPrint: config.isDevelopment);
  } else {
    logSink = ConsoleLogSink(); // Fallback
  }

  // Create logger
  final logger = Logger(
    serviceName: config.serviceName,
    env: config.env,
    sink: logSink,
    minLevel: config.logLevel,
  );

  // Create tracer
  final tracer = config.enableTracing
      ? TracerFactory.createConsole(
          serviceName: config.serviceName,
          samplingRate: config.tracingSampleRate,
        )
      : Tracer(serviceName: config.serviceName);

  // Create metrics collector
  final metrics = config.enableMetrics
      ? MetricsCollector(
          exporter: ConsoleMetricsExporter(),
          batchSize: config.batchSize,
          flushInterval: config.flushInterval,
        )
      : MetricsCollector();

  // Create error tracker
  final errorTracker = ErrorTracker(
    logger: logger,
  );

  if (config.enableErrorTracking) {
    errorTracker.setupGlobalErrorHandling();
  }

  final service = ObservabilityService(
    config: config,
    logger: logger,
    tracer: tracer,
    metrics: metrics,
    errorTracker: errorTracker,
  );

  ref.onDispose(() => service.dispose());

  return service;
});

/// Provider for logger
final loggerProvider = Provider<Logger>((ref) {
  return ref.watch(observabilityServiceProvider).logger;
});

/// Provider for tracer
final tracerProvider = Provider<Tracer>((ref) {
  return ref.watch(observabilityServiceProvider).tracer;
});

/// Provider for metrics collector
final metricsProvider = Provider<MetricsCollector>((ref) {
  return ref.watch(observabilityServiceProvider).metrics;
});

/// Provider for error tracker
final errorTrackerProvider = Provider<ErrorTracker>((ref) {
  return ref.watch(observabilityServiceProvider).errorTracker;
});

/// Widget that sets up observability for child widgets
class ObservabilityScope extends ConsumerWidget {
  /// Creates an observability scope
  const ObservabilityScope({
    required this.child,
    super.key,
  });

  /// Child widget
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Initialize observability service
    ref.watch(observabilityServiceProvider);
    return child;
  }
}

/// Extension methods for using observability with WidgetRef
extension ObservabilityRef on WidgetRef {
  /// Get the logger
  Logger get logger => watch(loggerProvider);

  /// Get the tracer
  Tracer get tracer => watch(tracerProvider);

  /// Get the metrics collector
  MetricsCollector get metrics => watch(metricsProvider);

  /// Get the error tracker
  ErrorTracker get errorTracker => watch(errorTrackerProvider);

  /// Log a debug message
  void logDebug(String message, [Map<String, dynamic>? extra]) {
    read(loggerProvider).debug(message, extra);
  }

  /// Log an info message
  void logInfo(String message, [Map<String, dynamic>? extra]) {
    read(loggerProvider).info(message, extra);
  }

  /// Log a warning message
  void logWarn(String message, [Map<String, dynamic>? extra]) {
    read(loggerProvider).warn(message, extra);
  }

  /// Log an error message
  void logError(
    String message, {
    Object? error,
    StackTrace? stackTrace,
    Map<String, dynamic>? extra,
  }) {
    read(loggerProvider).error(
      message,
      error: error,
      stackTrace: stackTrace,
      extra: extra,
    );
  }

  /// Track an error
  void trackError(
    Object error, {
    StackTrace? stackTrace,
    ErrorSeverity severity = ErrorSeverity.medium,
    Map<String, dynamic>? context,
  }) {
    read(errorTrackerProvider).captureError(
      error,
      stackTrace: stackTrace,
      severity: severity,
      context: context,
    );
  }
}
