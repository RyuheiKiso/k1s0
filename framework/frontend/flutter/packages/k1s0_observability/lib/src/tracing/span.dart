import 'package:freezed_annotation/freezed_annotation.dart';

import 'trace_context.dart';

part 'span.freezed.dart';
part 'span.g.dart';

/// Span status
enum SpanStatus {
  /// Span completed successfully
  ok,

  /// Span completed with an error
  error,

  /// Status not set
  unset,
}

/// Span information
@freezed
class SpanInfo with _$SpanInfo {
  /// Creates span info
  const factory SpanInfo({
    /// Trace ID
    required String traceId,

    /// Span ID
    required String spanId,

    /// Span name
    required String name,

    /// Start time (Unix timestamp in milliseconds)
    required int startTime,

    /// Parent span ID
    String? parentSpanId,

    /// End time (Unix timestamp in milliseconds)
    int? endTime,

    /// Span status
    @Default(SpanStatus.unset) SpanStatus status,

    /// Status message
    String? statusMessage,

    /// Attributes
    @Default({}) Map<String, Object> attributes,
  }) = _SpanInfo;

  const SpanInfo._();

  /// Creates span info from JSON
  factory SpanInfo.fromJson(Map<String, dynamic> json) =>
      _$SpanInfoFromJson(json);

  /// Duration in milliseconds
  int? get durationMs {
    if (endTime == null) return null;
    return endTime! - startTime;
  }

  /// Whether the span has ended
  bool get hasEnded => endTime != null;

  /// Whether the span has an error
  bool get hasError => status == SpanStatus.error;
}

/// Active span that can be ended
class ActiveSpan {
  /// Creates an active span
  ActiveSpan({
    required this.context,
    required this.name,
    Map<String, Object>? attributes,
    this.onEnd,
  })  : _startTime = DateTime.now().millisecondsSinceEpoch,
        _attributes = attributes ?? {};

  /// Trace context for this span
  final TraceContext context;

  /// Span name
  final String name;

  /// Callback when span ends
  final void Function(SpanInfo)? onEnd;

  final int _startTime;
  final Map<String, Object> _attributes;
  SpanStatus _status = SpanStatus.unset;
  String? _statusMessage;
  int? _endTime;
  bool _ended = false;

  /// Trace ID
  String get traceId => context.traceId;

  /// Span ID
  String get spanId => context.spanId;

  /// Parent span ID
  String? get parentSpanId => context.parentSpanId;

  /// Whether the span has ended
  bool get hasEnded => _ended;

  /// Set an attribute
  void setAttribute(String key, Object value) {
    if (_ended) return;
    _attributes[key] = value;
  }

  /// Set multiple attributes
  void setAttributes(Map<String, Object> attributes) {
    if (_ended) return;
    _attributes.addAll(attributes);
  }

  /// Mark the span as OK
  void setOk([String? message]) {
    if (_ended) return;
    _status = SpanStatus.ok;
    _statusMessage = message;
  }

  /// Mark the span as error
  void setError([String? message, Object? error, StackTrace? stackTrace]) {
    if (_ended) return;
    _status = SpanStatus.error;
    _statusMessage = message;
    if (error != null) {
      _attributes['error.type'] = error.runtimeType.toString();
      _attributes['error.message'] = error.toString();
    }
    if (stackTrace != null) {
      _attributes['error.stack_trace'] = stackTrace.toString();
    }
  }

  /// End the span
  SpanInfo end() {
    if (_ended) {
      return _toSpanInfo();
    }

    _ended = true;
    _endTime = DateTime.now().millisecondsSinceEpoch;

    if (_status == SpanStatus.unset) {
      _status = SpanStatus.ok;
    }

    final info = _toSpanInfo();
    onEnd?.call(info);
    return info;
  }

  SpanInfo _toSpanInfo() => SpanInfo(
        traceId: traceId,
        spanId: spanId,
        name: name,
        startTime: _startTime,
        parentSpanId: parentSpanId,
        endTime: _endTime,
        status: _status,
        statusMessage: _statusMessage,
        attributes: Map.unmodifiable(_attributes),
      );
}
