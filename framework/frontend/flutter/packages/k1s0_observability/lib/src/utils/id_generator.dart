import 'package:uuid/uuid.dart';

const _uuid = Uuid();

/// Generate a trace ID (32-character hex string)
String generateTraceId() => _uuid.v4().replaceAll('-', '');

/// Generate a span ID (16-character hex string)
String generateSpanId() => _uuid.v4().replaceAll('-', '').substring(0, 16);

/// Generate a request ID
String generateRequestId() => _uuid.v4();

/// Generate a session ID
String generateSessionId() => _uuid.v4();

/// Create a traceparent header value
/// Format: 00-<trace-id>-<span-id>-01
String createTraceparent(String traceId, String spanId) =>
    '00-$traceId-$spanId-01';

// Keep IdGenerator for backward compatibility
/// ID generator for trace and span IDs
///
/// Deprecated: Use top-level functions instead.
@Deprecated('Use top-level functions instead')
final class IdGenerator {
  @Deprecated('Use top-level functions instead')
  IdGenerator._();
  /// Generate a trace ID (32-character hex string)
  static String generateTraceId() => _uuid.v4().replaceAll('-', '');

  /// Generate a span ID (16-character hex string)
  static String generateSpanId() =>
      _uuid.v4().replaceAll('-', '').substring(0, 16);

  /// Generate a request ID
  static String generateRequestId() => _uuid.v4();

  /// Generate a session ID
  static String generateSessionId() => _uuid.v4();

  /// Parse trace ID from traceparent header
  /// Format: 00-<trace-id>-<span-id>-<trace-flags>
  static String? parseTraceIdFromTraceparent(String traceparent) {
    final parts = traceparent.split('-');
    if (parts.length >= 2) {
      return parts[1];
    }
    return null;
  }

  /// Parse span ID from traceparent header
  static String? parseSpanIdFromTraceparent(String traceparent) {
    final parts = traceparent.split('-');
    if (parts.length >= 3) {
      return parts[2];
    }
    return null;
  }

  /// Create a traceparent header value
  /// Format: 00-<trace-id>-<span-id>-01
  static String createTraceparent(String traceId, String spanId) =>
      '00-$traceId-$spanId-01';
}
