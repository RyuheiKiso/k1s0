import 'package:dio/dio.dart';
import 'package:uuid/uuid.dart';

/// Interceptor that adds trace context to requests
///
/// Adds the following headers:
/// - x-trace-id: Unique trace ID for request correlation
/// - x-span-id: Unique span ID for this request
/// - traceparent: W3C Trace Context header
class TraceInterceptor extends Interceptor {
  /// Creates a trace interceptor
  TraceInterceptor({
    this.serviceName = 'k1s0-flutter',
    Uuid? uuid,
  }) : _uuid = uuid ?? const Uuid();

  /// Service name for tracing
  final String serviceName;

  final Uuid _uuid;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    // Get or generate trace ID
    final traceId = options.extra['traceId'] as String? ?? _generateTraceId();
    final spanId = _generateSpanId();

    // Store in extra for later use
    options.extra['traceId'] = traceId;
    options.extra['spanId'] = spanId;
    options.extra['requestStartTime'] = DateTime.now().millisecondsSinceEpoch;

    // Add trace headers
    options.headers['x-trace-id'] = traceId;
    options.headers['x-span-id'] = spanId;
    options.headers['x-request-id'] = _uuid.v4();

    // Add W3C Trace Context header
    // Format: 00-<trace-id>-<span-id>-<trace-flags>
    final traceparent = '00-$traceId-$spanId-01';
    options.headers['traceparent'] = traceparent;

    handler.next(options);
  }

  @override
  void onResponse(Response<dynamic> response, ResponseInterceptorHandler handler) {
    // Extract trace info from response for logging
    final requestTraceId = response.requestOptions.extra['traceId'] as String?;
    final responseTraceId = response.headers.value('x-trace-id');

    // Use server trace ID if provided, otherwise keep client trace ID
    if (responseTraceId != null && responseTraceId != requestTraceId) {
      response.requestOptions.extra['serverTraceId'] = responseTraceId;
    }

    handler.next(response);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    // Trace ID is preserved in error for debugging
    handler.next(err);
  }

  /// Generate a 32-character hex trace ID
  String _generateTraceId() {
    final uuid = _uuid.v4().replaceAll('-', '');
    return uuid;
  }

  /// Generate a 16-character hex span ID
  String _generateSpanId() {
    final uuid = _uuid.v4().replaceAll('-', '');
    return uuid.substring(0, 16);
  }
}

/// Extension to get trace context from RequestOptions
extension TraceContextExtension on RequestOptions {
  /// Get the trace ID
  String? get traceId => extra['traceId'] as String?;

  /// Get the span ID
  String? get spanId => extra['spanId'] as String?;

  /// Get the request start time
  int? get requestStartTime => extra['requestStartTime'] as int?;

  /// Get the request duration
  Duration? get requestDuration {
    final startTime = requestStartTime;
    if (startTime == null) return null;
    return Duration(
      milliseconds: DateTime.now().millisecondsSinceEpoch - startTime,
    );
  }
}
