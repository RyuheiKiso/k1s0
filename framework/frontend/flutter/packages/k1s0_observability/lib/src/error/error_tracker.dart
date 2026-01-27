import 'dart:async';

import 'package:flutter/foundation.dart';

import '../logging/logger.dart';
import '../tracing/trace_context.dart';
import 'error_info.dart';

/// Error severity
enum ErrorSeverity {
  /// Low severity - minor issue
  low,

  /// Medium severity - important but not critical
  medium,

  /// High severity - critical issue
  high,

  /// Fatal - app crash or major failure
  fatal,
}

/// Error tracker for capturing and reporting errors
class ErrorTracker {
  /// Creates an error tracker
  ErrorTracker({
    this.logger,
    this.onError,
    this.reportToRemote = false,
    this.captureStackTrace = true,
  });

  /// Logger for error logging
  final Logger? logger;

  /// Error callback
  final void Function(ErrorInfo error, ErrorSeverity severity)? onError;

  /// Whether to report errors to remote
  final bool reportToRemote;

  /// Whether to capture stack traces
  final bool captureStackTrace;

  final List<ErrorInfo> _recentErrors = [];

  /// Maximum number of recent errors to keep
  static const int maxRecentErrors = 100;

  /// Get recent errors
  List<ErrorInfo> get recentErrors => List.unmodifiable(_recentErrors);

  /// Capture an error
  void captureError(
    Object error, {
    StackTrace? stackTrace,
    ErrorSeverity severity = ErrorSeverity.medium,
    String? code,
    TraceContext? traceContext,
    Map<String, dynamic>? context,
  }) {
    final errorInfo = ErrorInfo.fromException(
      error,
      stackTrace: captureStackTrace ? stackTrace : null,
      code: code,
      traceId: traceContext?.traceId,
      context: context,
    );

    _addError(errorInfo);
    _logError(errorInfo, severity);
    onError?.call(errorInfo, severity);
  }

  /// Capture an exception with a message
  void captureException(
    Object exception,
    StackTrace stackTrace, {
    String? message,
    ErrorSeverity severity = ErrorSeverity.medium,
    Map<String, dynamic>? context,
  }) {
    final errorInfo = ErrorInfo.fromException(
      exception,
      stackTrace: captureStackTrace ? stackTrace : null,
      context: {
        if (message != null) 'message': message,
        ...?context,
      },
    );

    _addError(errorInfo);
    _logError(errorInfo, severity);
    onError?.call(errorInfo, severity);
  }

  /// Capture a message as an error
  void captureMessage(
    String message, {
    ErrorSeverity severity = ErrorSeverity.low,
    String? code,
    Map<String, dynamic>? context,
  }) {
    final errorInfo = ErrorInfo(
      type: 'Message',
      message: message,
      code: code,
      timestamp: DateTime.now().toUtc().toIso8601String(),
      context: context ?? {},
    );

    _addError(errorInfo);
    _logError(errorInfo, severity);
    onError?.call(errorInfo, severity);
  }

  void _addError(ErrorInfo error) {
    _recentErrors.add(error);
    if (_recentErrors.length > maxRecentErrors) {
      _recentErrors.removeAt(0);
    }
  }

  void _logError(ErrorInfo error, ErrorSeverity severity) {
    if (logger == null) {
      debugPrint('[ERROR] ${error.type}: ${error.message}');
      return;
    }

    final extra = error.toLogMap();
    extra['severity'] = severity.name;

    logger!.error(
      error.message,
      extra: extra,
    );
  }

  /// Clear recent errors
  void clearErrors() {
    _recentErrors.clear();
  }

  /// Setup global error handling
  void setupGlobalErrorHandling() {
    // Capture Flutter errors
    FlutterError.onError = (details) {
      captureException(
        details.exception,
        details.stack ?? StackTrace.current,
        message: details.exceptionAsString(),
        severity: details.silent ? ErrorSeverity.low : ErrorSeverity.high,
        context: {
          'library': details.library ?? 'unknown',
          'context': details.context?.toString(),
        },
      );
    };

    // Capture platform dispatcher errors
    PlatformDispatcher.instance.onError = (error, stack) {
      captureException(
        error,
        stack,
        severity: ErrorSeverity.fatal,
      );
      return true;
    };
  }

  /// Run a zone with error capturing
  Future<T> runWithErrorCapturing<T>(Future<T> Function() body) async {
    T? result;
    await runZonedGuarded(() async {
      result = await body();
    }, (error, stackTrace) {
      captureException(
        error,
        stackTrace,
        severity: ErrorSeverity.high,
      );
    });
    return result as T;
  }
}

/// Create an error tracker with a logger
ErrorTracker createErrorTrackerWithLogger(Logger logger) => ErrorTracker(
      logger: logger,
    );

/// Create an error tracker with a callback
ErrorTracker createErrorTrackerWithCallback(
  void Function(ErrorInfo error, ErrorSeverity severity) onError,
) =>
    ErrorTracker(
      onError: onError,
    );
