import 'dart:async';

import 'span.dart';
import 'trace_context.dart';

/// Span exporter interface
abstract class SpanExporter {
  /// Export spans
  Future<void> export(List<SpanInfo> spans);

  /// Shutdown the exporter
  Future<void> shutdown();
}

/// Console span exporter for debugging
class ConsoleSpanExporter implements SpanExporter {
  @override
  Future<void> export(List<SpanInfo> spans) async {
    for (final span in spans) {
      final duration = span.durationMs;
      // ignore: avoid_print
      print(
        '[SPAN] ${span.name} '
        '[${span.traceId.substring(0, 8)}...] '
        '${duration != null ? "${duration}ms" : "ongoing"} '
        '${span.status.name}',
      );
    }
  }

  @override
  Future<void> shutdown() async {}
}

/// Buffered span exporter
class BufferedSpanExporter implements SpanExporter {
  /// Creates a buffered span exporter
  BufferedSpanExporter({
    required this.delegate,
    this.batchSize = 50,
    this.flushInterval = const Duration(seconds: 10),
  }) {
    _startFlushTimer();
  }

  /// Delegate exporter
  final SpanExporter delegate;

  /// Batch size
  final int batchSize;

  /// Flush interval
  final Duration flushInterval;

  final List<SpanInfo> _buffer = [];
  Timer? _flushTimer;
  bool _shutdown = false;

  void _startFlushTimer() {
    _flushTimer = Timer.periodic(flushInterval, (_) => _flush());
  }

  @override
  Future<void> export(List<SpanInfo> spans) async {
    if (_shutdown) return;

    _buffer.addAll(spans);

    if (_buffer.length >= batchSize) {
      await _flush();
    }
  }

  Future<void> _flush() async {
    if (_buffer.isEmpty) return;

    final spans = List<SpanInfo>.from(_buffer);
    _buffer.clear();

    await delegate.export(spans);
  }

  @override
  Future<void> shutdown() async {
    _shutdown = true;
    _flushTimer?.cancel();
    await _flush();
    await delegate.shutdown();
  }
}

/// Tracer for creating and managing spans
class Tracer {
  /// Creates a tracer
  Tracer({
    required this.serviceName,
    this.exporter,
    this.samplingRate = 1.0,
  });

  /// Service name
  final String serviceName;

  /// Span exporter
  final SpanExporter? exporter;

  /// Sampling rate (0.0 - 1.0)
  final double samplingRate;

  TraceContext? _currentContext;

  /// Get the current trace context
  TraceContext get currentContext =>
      _currentContext ?? TraceContext.create();

  /// Set the current trace context
  set currentContext(TraceContext context) => _currentContext = context;

  /// Start a new span
  ActiveSpan startSpan(
    String name, {
    TraceContext? parent,
    Map<String, Object>? attributes,
  }) {
    final parentContext = parent ?? _currentContext;
    final context = parentContext?.createChild(name: name) ??
        TraceContext.create();

    _currentContext = context;

    return ActiveSpan(
      context: context,
      name: name,
      attributes: {
        'service.name': serviceName,
        ...?attributes,
      },
      onEnd: _onSpanEnd,
    );
  }

  /// Execute a function within a span
  Future<T> trace<T>(
    String name,
    Future<T> Function(ActiveSpan span) fn, {
    TraceContext? parent,
    Map<String, Object>? attributes,
  }) async {
    final span = startSpan(name, parent: parent, attributes: attributes);

    try {
      final result = await fn(span);
      span.setOk();
      return result;
    } catch (e, st) {
      span.setError(e.toString(), e, st);
      rethrow;
    } finally {
      span.end();
    }
  }

  /// Execute a synchronous function within a span
  T traceSync<T>(
    String name,
    T Function(ActiveSpan span) fn, {
    TraceContext? parent,
    Map<String, Object>? attributes,
  }) {
    final span = startSpan(name, parent: parent, attributes: attributes);

    try {
      final result = fn(span);
      span.setOk();
      return result;
    } catch (e, st) {
      span.setError(e.toString(), e, st);
      rethrow;
    } finally {
      span.end();
    }
  }

  void _onSpanEnd(SpanInfo span) {
    // Check sampling rate
    if (samplingRate < 1.0) {
      final hash = span.traceId.hashCode;
      if ((hash.abs() % 100) >= (samplingRate * 100)) {
        return;
      }
    }

    exporter?.export([span]);
  }

  /// Shutdown the tracer
  Future<void> shutdown() async {
    await exporter?.shutdown();
  }
}

/// Create a tracer with console exporter
Tracer createConsoleTracer({
  required String serviceName,
  double samplingRate = 1.0,
}) =>
    Tracer(
      serviceName: serviceName,
      exporter: ConsoleSpanExporter(),
      samplingRate: samplingRate,
    );

/// Create a tracer with buffered exporter
Tracer createBufferedTracer({
  required String serviceName,
  required SpanExporter delegate,
  double samplingRate = 1.0,
  int batchSize = 50,
  Duration flushInterval = const Duration(seconds: 10),
}) =>
    Tracer(
      serviceName: serviceName,
      exporter: BufferedSpanExporter(
        delegate: delegate,
        batchSize: batchSize,
        flushInterval: flushInterval,
      ),
      samplingRate: samplingRate,
    );

// Keep TracerFactory for backward compatibility
/// Tracer factory
///
/// Deprecated: Use top-level functions [createConsoleTracer] and
/// [createBufferedTracer] instead.
@Deprecated('Use createConsoleTracer and createBufferedTracer functions instead')
final class TracerFactory {
  @Deprecated('Use createConsoleTracer and createBufferedTracer functions instead')
  TracerFactory._();
  /// Create a tracer with console exporter
  static Tracer createConsole({
    required String serviceName,
    double samplingRate = 1.0,
  }) =>
      createConsoleTracer(serviceName: serviceName, samplingRate: samplingRate);

  /// Create a tracer with buffered exporter
  static Tracer createBuffered({
    required String serviceName,
    required SpanExporter delegate,
    double samplingRate = 1.0,
    int batchSize = 50,
    Duration flushInterval = const Duration(seconds: 10),
  }) =>
      createBufferedTracer(
        serviceName: serviceName,
        delegate: delegate,
        samplingRate: samplingRate,
        batchSize: batchSize,
        flushInterval: flushInterval,
      );
}
