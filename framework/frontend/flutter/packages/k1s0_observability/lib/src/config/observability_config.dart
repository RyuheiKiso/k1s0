import 'package:freezed_annotation/freezed_annotation.dart';

import '../logging/log_level.dart';

part 'observability_config.freezed.dart';
part 'observability_config.g.dart';

/// Observability configuration
@freezed
class ObservabilityConfig with _$ObservabilityConfig {
  /// Creates an observability configuration
  const factory ObservabilityConfig({
    /// Service name
    required String serviceName,

    /// Environment (dev/stg/prod)
    required String env,

    /// Service version
    String? version,

    /// Minimum log level
    @Default(LogLevel.info) LogLevel logLevel,

    /// Enable console logging
    @Default(true) bool enableConsole,

    /// Enable tracing
    @Default(true) bool enableTracing,

    /// Enable metrics collection
    @Default(true) bool enableMetrics,

    /// Enable error tracking
    @Default(true) bool enableErrorTracking,

    /// Sampling rate for tracing (0.0 - 1.0)
    @Default(1.0) double tracingSampleRate,

    /// OTLP endpoint for exporting
    String? otlpEndpoint,

    /// Batch size for exports
    @Default(50) int batchSize,

    /// Flush interval in seconds
    @Default(10) int flushIntervalSeconds,
  }) = _ObservabilityConfig;

  const ObservabilityConfig._();

  /// Creates an observability configuration from JSON
  factory ObservabilityConfig.fromJson(Map<String, dynamic> json) =>
      _$ObservabilityConfigFromJson(json);

  /// Get flush interval as Duration
  Duration get flushInterval => Duration(seconds: flushIntervalSeconds);

  /// Whether this is a production environment
  bool get isProduction => env == 'prod' || env == 'production';

  /// Whether this is a development environment
  bool get isDevelopment => env == 'dev' || env == 'development';
}
