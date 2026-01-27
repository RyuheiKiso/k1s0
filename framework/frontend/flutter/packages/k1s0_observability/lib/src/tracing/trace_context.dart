import 'package:meta/meta.dart';

import '../utils/id_generator.dart';

/// Trace context for distributed tracing
@immutable
class TraceContext {
  /// Creates a trace context
  const TraceContext({
    required this.traceId,
    required this.spanId,
    this.parentSpanId,
    this.requestId,
    this.baggage = const {},
  });

  /// Create a new trace context
  factory TraceContext.create({
    String? traceId,
    String? spanId,
    String? requestId,
    Map<String, String> baggage = const {},
  }) =>
      TraceContext(
        traceId: traceId ?? generateTraceId(),
        spanId: spanId ?? generateSpanId(),
        requestId: requestId,
        baggage: baggage,
      );

  /// Parse from W3C traceparent header
  /// Format: 00-<trace-id>-<span-id>-<trace-flags>
  factory TraceContext.fromTraceparent(String traceparent) {
    final parts = traceparent.split('-');
    if (parts.length != 4) {
      throw FormatException('Invalid traceparent format: $traceparent');
    }

    return TraceContext(
      traceId: parts[1],
      spanId: parts[2],
    );
  }

  /// Try to parse from W3C traceparent header
  static TraceContext? tryFromTraceparent(String? traceparent) {
    if (traceparent == null) return null;
    try {
      return TraceContext.fromTraceparent(traceparent);
    } on FormatException {
      return null;
    } on Exception {
      return null;
    }
  }

  /// Trace ID (32-character hex string)
  final String traceId;

  /// Span ID (16-character hex string)
  final String spanId;

  /// Parent span ID
  final String? parentSpanId;

  /// Request ID
  final String? requestId;

  /// Baggage items for propagation
  final Map<String, String> baggage;

  /// Create a child context with a new span
  TraceContext createChild({String? name}) => TraceContext(
        traceId: traceId,
        spanId: generateSpanId(),
        parentSpanId: spanId,
        requestId: requestId,
        baggage: baggage,
      );

  /// Create a copy with updated values
  TraceContext copyWith({
    String? traceId,
    String? spanId,
    String? parentSpanId,
    String? requestId,
    Map<String, String>? baggage,
  }) =>
      TraceContext(
        traceId: traceId ?? this.traceId,
        spanId: spanId ?? this.spanId,
        parentSpanId: parentSpanId ?? this.parentSpanId,
        requestId: requestId ?? this.requestId,
        baggage: baggage ?? this.baggage,
      );

  /// Add baggage item
  TraceContext withBaggage(String key, String value) => copyWith(
        baggage: {...baggage, key: value},
      );

  /// Get W3C traceparent header value
  /// Format: 00-<trace-id>-<span-id>-01
  String toTraceparent() => '00-$traceId-$spanId-01';

  /// Convert to map for headers
  Map<String, String> toHeaders() => {
        'traceparent': toTraceparent(),
        'x-trace-id': traceId,
        'x-span-id': spanId,
        if (requestId != null) 'x-request-id': requestId!,
      };

  @override
  String toString() =>
      'TraceContext(traceId: $traceId, spanId: $spanId, parentSpanId: $parentSpanId)';

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    return other is TraceContext &&
        other.traceId == traceId &&
        other.spanId == spanId;
  }

  @override
  int get hashCode => Object.hash(traceId, spanId);
}
