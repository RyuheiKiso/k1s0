import '../tracing/trace_context.dart';
import '../utils/id_generator.dart';
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
  }) : _traceContext = traceContext ?? TraceContext.create();

  /// Service name
  final String serviceName;

  /// Environment
  final String env;

  /// Log sink
  final LogSink sink;

  /// Minimum log level
  final LogLevel minLevel;

  TraceContext _traceContext;

  /// Get the current trace context
  TraceContext get traceContext => _traceContext;

  /// Set the trace context
  set traceContext(TraceContext context) => _traceContext = context;

  /// Create a child logger with a new span
  Logger child({String? spanName}) {
    return Logger(
      serviceName: serviceName,
      env: env,
      sink: sink,
      minLevel: minLevel,
      traceContext: _traceContext.createChild(name: spanName),
    );
  }

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
      traceId: _traceContext.traceId,
      spanId: _traceContext.spanId,
      requestId: _traceContext.requestId,
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

/// Logger factory
class LoggerFactory {
  /// Create a console logger
  static Logger createConsole({
    required String serviceName,
    required String env,
    LogLevel minLevel = LogLevel.debug,
    bool prettyPrint = false,
  }) {
    return Logger(
      serviceName: serviceName,
      env: env,
      sink: ConsoleLogSink(prettyPrint: prettyPrint),
      minLevel: minLevel,
    );
  }

  /// Create a buffered logger
  static Logger createBuffered({
    required String serviceName,
    required String env,
    required LogSink delegate,
    LogLevel minLevel = LogLevel.info,
    int bufferSize = 100,
    Duration flushInterval = const Duration(seconds: 5),
  }) {
    return Logger(
      serviceName: serviceName,
      env: env,
      sink: BufferedLogSink(
        delegate: delegate,
        bufferSize: bufferSize,
        flushInterval: flushInterval,
      ),
      minLevel: minLevel,
    );
  }

  /// Create a logger with multiple sinks
  static Logger createComposite({
    required String serviceName,
    required String env,
    required List<LogSink> sinks,
    LogLevel minLevel = LogLevel.info,
  }) {
    return Logger(
      serviceName: serviceName,
      env: env,
      sink: CompositeLogSink(sinks),
      minLevel: minLevel,
    );
  }
}
