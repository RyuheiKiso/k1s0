import 'dart:async';
import 'dart:collection';
import 'dart:convert';

import 'package:flutter/foundation.dart';

import 'log_entry.dart';

/// Abstract log sink interface
abstract class LogSink {
  /// Write a log entry
  void write(LogEntry entry);

  /// Flush buffered entries
  Future<void> flush();

  /// Dispose resources
  void dispose();
}

/// Console log sink
class ConsoleLogSink implements LogSink {
  /// Creates a console log sink
  ConsoleLogSink({
    this.useColors = true,
    this.prettyPrint = false,
  });

  /// Whether to use colors in output
  final bool useColors;

  /// Whether to pretty print JSON
  final bool prettyPrint;

  @override
  void write(LogEntry entry) {
    String output;

    if (prettyPrint) {
      const encoder = JsonEncoder.withIndent('  ');
      output = encoder.convert(entry.toJson());
    } else {
      output = entry.toJsonString();
    }

    if (useColors && !kReleaseMode) {
      output = _colorize(output, entry.level);
    }

    debugPrint(output);
  }

  @override
  Future<void> flush() async {
    // Console doesn't need flushing
  }

  @override
  void dispose() {
    // Nothing to dispose
  }

  String _colorize(String text, dynamic level) {
    // ANSI color codes
    const reset = '\x1B[0m';
    const red = '\x1B[31m';
    const yellow = '\x1B[33m';
    const blue = '\x1B[34m';
    const gray = '\x1B[90m';

    String color;
    switch (level.toString()) {
      case 'LogLevel.error':
        color = red;
        break;
      case 'LogLevel.warn':
        color = yellow;
        break;
      case 'LogLevel.info':
        color = blue;
        break;
      case 'LogLevel.debug':
      default:
        color = gray;
    }

    return '$color$text$reset';
  }
}

/// Buffered log sink that batches entries
class BufferedLogSink implements LogSink {
  /// Creates a buffered log sink
  BufferedLogSink({
    required this.delegate,
    this.bufferSize = 100,
    this.flushInterval = const Duration(seconds: 5),
  }) {
    _startFlushTimer();
  }

  /// Delegate sink to write to
  final LogSink delegate;

  /// Maximum buffer size before auto-flush
  final int bufferSize;

  /// Interval between automatic flushes
  final Duration flushInterval;

  final Queue<LogEntry> _buffer = Queue<LogEntry>();
  Timer? _flushTimer;
  bool _disposed = false;

  void _startFlushTimer() {
    _flushTimer = Timer.periodic(flushInterval, (_) => flush());
  }

  @override
  void write(LogEntry entry) {
    if (_disposed) return;

    _buffer.add(entry);

    if (_buffer.length >= bufferSize) {
      flush();
    }
  }

  @override
  Future<void> flush() async {
    if (_disposed || _buffer.isEmpty) return;

    final entries = List<LogEntry>.from(_buffer);
    _buffer.clear();

    for (final entry in entries) {
      delegate.write(entry);
    }

    await delegate.flush();
  }

  @override
  void dispose() {
    _disposed = true;
    _flushTimer?.cancel();
    flush();
    delegate.dispose();
  }
}

/// Remote log sink that sends entries to a server
class RemoteLogSink implements LogSink {
  /// Creates a remote log sink
  RemoteLogSink({
    required this.endpoint,
    this.headers = const {},
    this.batchSize = 50,
    this.flushInterval = const Duration(seconds: 10),
    this.onError,
  }) {
    _startFlushTimer();
  }

  /// Remote endpoint URL
  final String endpoint;

  /// Additional headers for requests
  final Map<String, String> headers;

  /// Batch size for sending
  final int batchSize;

  /// Interval between automatic sends
  final Duration flushInterval;

  /// Error callback
  final void Function(Object error, StackTrace stackTrace)? onError;

  final Queue<LogEntry> _buffer = Queue<LogEntry>();
  Timer? _flushTimer;
  bool _disposed = false;

  void _startFlushTimer() {
    _flushTimer = Timer.periodic(flushInterval, (_) => flush());
  }

  @override
  void write(LogEntry entry) {
    if (_disposed) return;

    _buffer.add(entry);

    if (_buffer.length >= batchSize) {
      flush();
    }
  }

  @override
  Future<void> flush() async {
    if (_disposed || _buffer.isEmpty) return;

    final entries = <LogEntry>[];
    while (_buffer.isNotEmpty && entries.length < batchSize) {
      entries.add(_buffer.removeFirst());
    }

    try {
      // Note: In production, use dio or http package to send
      // This is a placeholder for the actual implementation
      debugPrint(
        '[RemoteLogSink] Would send ${entries.length} entries to $endpoint',
      );
    } catch (e, st) {
      // Put entries back in buffer on failure
      for (final entry in entries.reversed) {
        _buffer.addFirst(entry);
      }
      onError?.call(e, st);
    }
  }

  @override
  void dispose() {
    _disposed = true;
    _flushTimer?.cancel();
  }
}

/// Composite log sink that writes to multiple sinks
class CompositeLogSink implements LogSink {
  /// Creates a composite log sink
  CompositeLogSink(this.sinks);

  /// Child sinks
  final List<LogSink> sinks;

  @override
  void write(LogEntry entry) {
    for (final sink in sinks) {
      sink.write(entry);
    }
  }

  @override
  Future<void> flush() async {
    await Future.wait(sinks.map((s) => s.flush()));
  }

  @override
  void dispose() {
    for (final sink in sinks) {
      sink.dispose();
    }
  }
}
