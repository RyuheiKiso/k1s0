import '../tracing/trace_context.dart';
import 'log_entry.dart';
import 'log_level.dart';
import 'log_sink.dart';

/// Structured logger for k1s0 applications
class Logger {
  /// Creates a logger
  Logger({
    required this.serviceName,
    required this.env,
    required this.sink,
    this.minLevel = LogLevel.debug,
    TraceContext? traceContext,
  }) : traceContext = traceContext ?? TraceContext.create();

  /// Service name
  final String serviceName;

  /// Environment
  final String env;

  /// Log sink
  final LogSink sink;

  /// Minimum log level
  final LogLevel minLevel;

  /// Current trace context
  TraceContext traceContext;

  /// Create a child logger with a new span
  Logger child({String? spanName}) => Logger(
        serviceName: serviceName,
        env: env,
        sink: sink,
        minLevel: minLevel,
        traceContext: traceContext.createChild(name: spanName),
      );

  /// Log a debug message
  void debug(String message, [Map<String, dynamic>? extra]) {
    _log(LogLevel.debug, message, extra: extra);
  }

  /// Log an info message
  void info(String message, [Map<String, dynamic>? extra]) {
    _log(LogLevel.info, message, extra: extra);
  }

  /// Log a warning message
  void warn(String message, [Map<String, dynamic>? extra]) {
    _log(LogLevel.warn, message, extra: extra);
  }

  /// Log an error message
  void error(
    String message, {
    Object? error,
    StackTrace? stackTrace,
    Map<String, dynamic>? extra,
  }) {
    Map<String, dynamic>? errorInfo;
    if (error != null) {
      errorInfo = {
        'type': error.runtimeType.toString(),
        'message': error.toString(),
        if (stackTrace != null) 'stack_trace': stackTrace.toString(),
      };
    }

    _log(
      LogLevel.error,
      message,
      errorInfo: errorInfo,
      extra: extra,
    );
  }

  void _log(
    LogLevel level,
    String message, {
    Map<String, dynamic>? errorInfo,
    Map<String, dynamic>? extra,
  }) {
    if (!level.isAtLeast(minLevel)) {
      return;
    }

    final entry = LogEntry(
      timestamp: DateTime.now().toUtc().toIso8601String(),
      level: level,
      message: message,
      serviceName: serviceName,
      env: env,
      traceId: traceContext.traceId,
      spanId: traceContext.spanId,
      requestId: traceContext.requestId,
      errorInfo: errorInfo,
      extra: extra ?? {},
    );

    sink.write(entry);
  }

  /// Flush the log sink
  Future<void> flush() => sink.flush();

  /// Dispose the logger
  void dispose() => sink.dispose();
}

/// Create a console logger
Logger createConsoleLogger({
  required String serviceName,
  required String env,
  LogLevel minLevel = LogLevel.debug,
  bool prettyPrint = false,
}) =>
    Logger(
      serviceName: serviceName,
      env: env,
      sink: ConsoleLogSink(prettyPrint: prettyPrint),
      minLevel: minLevel,
    );

/// Create a buffered logger
Logger createBufferedLogger({
  required String serviceName,
  required String env,
  required LogSink delegate,
  LogLevel minLevel = LogLevel.info,
  int bufferSize = 100,
  Duration flushInterval = const Duration(seconds: 5),
}) =>
    Logger(
      serviceName: serviceName,
      env: env,
      sink: BufferedLogSink(
        delegate: delegate,
        bufferSize: bufferSize,
        flushInterval: flushInterval,
      ),
      minLevel: minLevel,
    );

/// Create a logger with multiple sinks
Logger createCompositeLogger({
  required String serviceName,
  required String env,
  required List<LogSink> sinks,
  LogLevel minLevel = LogLevel.info,
}) =>
    Logger(
      serviceName: serviceName,
      env: env,
      sink: CompositeLogSink(sinks),
      minLevel: minLevel,
    );
