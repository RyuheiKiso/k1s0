import 'package:meta/meta.dart';

import '../interceptors/logging_interceptor.dart';
import '../types/request_options.dart';

/// HTTP client configuration
@immutable
class HttpClientConfig {
  /// Creates an HTTP client configuration
  const HttpClientConfig({
    required this.baseUrl,
    this.timeout = const Duration(seconds: 30),
    this.connectTimeout = const Duration(seconds: 10),
    this.retryPolicy = RetryPolicy.none,
    this.defaultHeaders = const {},
    this.logLevel = HttpLogLevel.basic,
    this.validateStatus,
  });

  /// Base URL for all requests
  final String baseUrl;

  /// Request timeout
  final Duration timeout;

  /// Connection timeout
  final Duration connectTimeout;

  /// Default retry policy
  final RetryPolicy retryPolicy;

  /// Default headers for all requests
  final Map<String, String> defaultHeaders;

  /// Log level for HTTP logging
  final HttpLogLevel logLevel;

  /// Custom status code validator
  /// Returns true if the status code should be considered successful
  final bool Function(int? statusCode)? validateStatus;

  /// Create a copy with updated values
  HttpClientConfig copyWith({
    String? baseUrl,
    Duration? timeout,
    Duration? connectTimeout,
    RetryPolicy? retryPolicy,
    Map<String, String>? defaultHeaders,
    HttpLogLevel? logLevel,
    bool Function(int? statusCode)? validateStatus,
  }) {
    return HttpClientConfig(
      baseUrl: baseUrl ?? this.baseUrl,
      timeout: timeout ?? this.timeout,
      connectTimeout: connectTimeout ?? this.connectTimeout,
      retryPolicy: retryPolicy ?? this.retryPolicy,
      defaultHeaders: defaultHeaders ?? this.defaultHeaders,
      logLevel: logLevel ?? this.logLevel,
      validateStatus: validateStatus ?? this.validateStatus,
    );
  }
}
