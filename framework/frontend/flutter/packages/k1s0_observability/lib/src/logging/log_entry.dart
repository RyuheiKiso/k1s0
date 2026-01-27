import 'dart:convert';

import 'package:freezed_annotation/freezed_annotation.dart';

import 'log_level.dart';

part 'log_entry.freezed.dart';
part 'log_entry.g.dart';

/// Structured log entry
///
/// Contains all required fields as per k1s0 observability standards:
/// - timestamp
/// - level
/// - message
/// - service_name
/// - env
/// - trace_id
/// - span_id
@freezed
class LogEntry with _$LogEntry {
  /// Creates a log entry
  const factory LogEntry({
    /// ISO 8601 timestamp
    required String timestamp,

    /// Log level
    required LogLevel level,

    /// Log message
    required String message,

    /// Service name
    @JsonKey(name: 'service_name') required String serviceName,

    /// Environment (dev/stg/prod)
    required String env,

    /// Trace ID for request correlation
    @JsonKey(name: 'trace_id') required String traceId,

    /// Span ID
    @JsonKey(name: 'span_id') required String spanId,

    /// Request ID
    @JsonKey(name: 'request_id') String? requestId,

    /// Error information
    @JsonKey(name: 'error') Map<String, dynamic>? errorInfo,

    /// Additional fields
    @Default({}) Map<String, dynamic> extra,
  }) = _LogEntry;

  const LogEntry._();

  /// Creates a log entry from JSON
  factory LogEntry.fromJson(Map<String, dynamic> json) =>
      _$LogEntryFromJson(json);

  /// Convert to JSON string
  String toJsonString() => jsonEncode(toJson());

  /// Create a debug log entry
  factory LogEntry.debug({
    required String message,
    required String serviceName,
    required String env,
    required String traceId,
    required String spanId,
    String? requestId,
    Map<String, dynamic>? extra,
  }) {
    return LogEntry(
      timestamp: DateTime.now().toUtc().toIso8601String(),
      level: LogLevel.debug,
      message: message,
      serviceName: serviceName,
      env: env,
      traceId: traceId,
      spanId: spanId,
      requestId: requestId,
      extra: extra ?? {},
    );
  }

  /// Create an info log entry
  factory LogEntry.info({
    required String message,
    required String serviceName,
    required String env,
    required String traceId,
    required String spanId,
    String? requestId,
    Map<String, dynamic>? extra,
  }) {
    return LogEntry(
      timestamp: DateTime.now().toUtc().toIso8601String(),
      level: LogLevel.info,
      message: message,
      serviceName: serviceName,
      env: env,
      traceId: traceId,
      spanId: spanId,
      requestId: requestId,
      extra: extra ?? {},
    );
  }

  /// Create a warning log entry
  factory LogEntry.warn({
    required String message,
    required String serviceName,
    required String env,
    required String traceId,
    required String spanId,
    String? requestId,
    Map<String, dynamic>? extra,
  }) {
    return LogEntry(
      timestamp: DateTime.now().toUtc().toIso8601String(),
      level: LogLevel.warn,
      message: message,
      serviceName: serviceName,
      env: env,
      traceId: traceId,
      spanId: spanId,
      requestId: requestId,
      extra: extra ?? {},
    );
  }

  /// Create an error log entry
  factory LogEntry.error({
    required String message,
    required String serviceName,
    required String env,
    required String traceId,
    required String spanId,
    String? requestId,
    Map<String, dynamic>? errorInfo,
    Map<String, dynamic>? extra,
  }) {
    return LogEntry(
      timestamp: DateTime.now().toUtc().toIso8601String(),
      level: LogLevel.error,
      message: message,
      serviceName: serviceName,
      env: env,
      traceId: traceId,
      spanId: spanId,
      requestId: requestId,
      errorInfo: errorInfo,
      extra: extra ?? {},
    );
  }
}
