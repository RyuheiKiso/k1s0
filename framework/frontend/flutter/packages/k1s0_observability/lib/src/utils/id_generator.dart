import 'package:uuid/uuid.dart';

/// ID generator for trace and span IDs
class IdGenerator {
  static const _uuid = Uuid();

  /// Generate a trace ID (32-character hex string)
  static String generateTraceId() {
    return _uuid.v4().replaceAll('-', '');
  }

  /// Generate a span ID (16-character hex string)
  static String generateSpanId() {
    return _uuid.v4().replaceAll('-', '').substring(0, 16);
  }

  /// Generate a request ID
  static String generateRequestId() {
    return _uuid.v4();
  }

  /// Generate a session ID
  static String generateSessionId() {
    return _uuid.v4();
  }

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
  static String createTraceparent(String traceId, String spanId) {
    return '00-$traceId-$spanId-01';
  }
}
