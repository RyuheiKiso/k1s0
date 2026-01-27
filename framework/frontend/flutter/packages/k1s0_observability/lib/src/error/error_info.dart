import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:stack_trace/stack_trace.dart';

part 'error_info.freezed.dart';
part 'error_info.g.dart';

/// Error information
@freezed
class ErrorInfo with _$ErrorInfo {
  /// Creates error info
  const factory ErrorInfo({
    /// Error type name
    required String type,

    /// Error message
    required String message,

    /// Stack trace
    String? stackTrace,

    /// Error code
    String? code,

    /// Original error info (for chained errors)
    ErrorInfo? cause,

    /// Timestamp
    required String timestamp,

    /// Trace ID
    String? traceId,

    /// Additional context
    @Default({}) Map<String, dynamic> context,
  }) = _ErrorInfo;

  const ErrorInfo._();

  /// Creates error info from JSON
  factory ErrorInfo.fromJson(Map<String, dynamic> json) =>
      _$ErrorInfoFromJson(json);

  /// Create from an exception
  factory ErrorInfo.fromException(
    Object error, {
    StackTrace? stackTrace,
    String? code,
    String? traceId,
    Map<String, dynamic>? context,
  }) {
    String? formattedStackTrace;
    if (stackTrace != null) {
      // Format stack trace for readability
      final trace = Trace.from(stackTrace);
      formattedStackTrace = trace.terse.toString();
    }

    return ErrorInfo(
      type: error.runtimeType.toString(),
      message: error.toString(),
      stackTrace: formattedStackTrace,
      code: code,
      timestamp: DateTime.now().toUtc().toIso8601String(),
      traceId: traceId,
      context: context ?? {},
    );
  }

  /// Create with a cause
  ErrorInfo withCause(ErrorInfo cause) {
    return copyWith(cause: cause);
  }

  /// Add context
  ErrorInfo withContext(Map<String, dynamic> additionalContext) {
    return copyWith(
      context: {...context, ...additionalContext},
    );
  }

  /// Convert to log-friendly map
  Map<String, dynamic> toLogMap() {
    return {
      'error_type': type,
      'error_message': message,
      if (code != null) 'error_code': code,
      if (stackTrace != null) 'error_stack_trace': stackTrace,
      if (cause != null) 'error_cause': cause!.toLogMap(),
      ...context,
    };
  }
}
