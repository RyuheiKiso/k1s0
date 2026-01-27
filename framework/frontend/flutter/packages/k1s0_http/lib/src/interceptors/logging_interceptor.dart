import 'package:dio/dio.dart';

import 'trace_interceptor.dart';

/// Log level for HTTP logging
enum HttpLogLevel {
  /// No logging
  none,

  /// Log basic request/response info
  basic,

  /// Log headers
  headers,

  /// Log body (be careful with sensitive data)
  body,
}

/// Interceptor that logs HTTP requests and responses
class LoggingInterceptor extends Interceptor {
  /// Creates a logging interceptor
  LoggingInterceptor({
    this.logLevel = HttpLogLevel.basic,
    this.logger,
    this.logRequestBody = false,
    this.logResponseBody = false,
    this.maxBodyLength = 1000,
  });

  /// Log level
  final HttpLogLevel logLevel;

  /// Custom logger function
  final void Function(String message)? logger;

  /// Whether to log request body (use with caution)
  final bool logRequestBody;

  /// Whether to log response body (use with caution)
  final bool logResponseBody;

  /// Maximum body length to log
  final int maxBodyLength;

  void _log(String message) {
    if (logger != null) {
      logger!(message);
    } else {
      // ignore: avoid_print
      print('[HTTP] $message');
    }
  }

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (logLevel == HttpLogLevel.none) {
      handler.next(options);
      return;
    }

    final buffer = StringBuffer();
    buffer.writeln('--> ${options.method} ${options.uri}');

    if (logLevel.index >= HttpLogLevel.headers.index) {
      // Log trace context
      final traceId = options.traceId;
      if (traceId != null) {
        buffer.writeln('x-trace-id: $traceId');
      }

      // Log headers (excluding sensitive ones)
      options.headers.forEach((key, value) {
        if (!_isSensitiveHeader(key)) {
          buffer.writeln('$key: $value');
        } else {
          buffer.writeln('$key: [REDACTED]');
        }
      });
    }

    if (logRequestBody &&
        logLevel == HttpLogLevel.body &&
        options.data != null) {
      buffer.writeln(_truncateBody(options.data.toString()));
    }

    _log(buffer.toString());
    handler.next(options);
  }

  @override
  void onResponse(Response response, ResponseInterceptorHandler handler) {
    if (logLevel == HttpLogLevel.none) {
      handler.next(response);
      return;
    }

    final buffer = StringBuffer();
    final duration = response.requestOptions.requestDuration;
    final durationStr = duration != null ? ' (${duration.inMilliseconds}ms)' : '';

    buffer.writeln(
      '<-- ${response.statusCode} ${response.requestOptions.uri}$durationStr',
    );

    if (logLevel.index >= HttpLogLevel.headers.index) {
      // Log trace ID
      final traceId = response.headers.value('x-trace-id');
      if (traceId != null) {
        buffer.writeln('x-trace-id: $traceId');
      }
    }

    if (logResponseBody && logLevel == HttpLogLevel.body) {
      buffer.writeln(_truncateBody(response.data.toString()));
    }

    _log(buffer.toString());
    handler.next(response);
  }

  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (logLevel == HttpLogLevel.none) {
      handler.next(err);
      return;
    }

    final buffer = StringBuffer();
    final duration = err.requestOptions.requestDuration;
    final durationStr = duration != null ? ' (${duration.inMilliseconds}ms)' : '';

    buffer.writeln(
      '<-- ERROR ${err.response?.statusCode ?? 'NETWORK'} '
      '${err.requestOptions.uri}$durationStr',
    );
    buffer.writeln('${err.type}: ${err.message}');

    // Log trace ID
    final traceId = err.requestOptions.traceId;
    if (traceId != null) {
      buffer.writeln('x-trace-id: $traceId');
    }

    _log(buffer.toString());
    handler.next(err);
  }

  bool _isSensitiveHeader(String header) {
    final lower = header.toLowerCase();
    return lower == 'authorization' ||
        lower == 'cookie' ||
        lower == 'set-cookie' ||
        lower.contains('token') ||
        lower.contains('secret') ||
        lower.contains('password');
  }

  String _truncateBody(String body) {
    if (body.length <= maxBodyLength) {
      return body;
    }
    return '${body.substring(0, maxBodyLength)}... [truncated]';
  }
}
