import 'package:meta/meta.dart';

/// HTTP request options
@immutable
class K1s0RequestOptions {
  /// Creates request options
  const K1s0RequestOptions({
    this.headers,
    this.queryParameters,
    this.timeout,
    this.skipAuth = false,
    this.retry = false,
    this.retryCount,
    this.traceId,
    this.extra,
  });

  /// Additional headers for this request
  final Map<String, String>? headers;

  /// Query parameters
  final Map<String, dynamic>? queryParameters;

  /// Request timeout in milliseconds (overrides client default)
  final int? timeout;

  /// Skip authentication for this request
  final bool skipAuth;

  /// Enable retry for this request
  final bool retry;

  /// Number of retry attempts (overrides client default)
  final int? retryCount;

  /// Trace ID for this request (auto-generated if not provided)
  final String? traceId;

  /// Extra data to pass through interceptors
  final Map<String, dynamic>? extra;

  /// Create a copy with updated values
  K1s0RequestOptions copyWith({
    Map<String, String>? headers,
    Map<String, dynamic>? queryParameters,
    int? timeout,
    bool? skipAuth,
    bool? retry,
    int? retryCount,
    String? traceId,
    Map<String, dynamic>? extra,
  }) =>
      K1s0RequestOptions(
        headers: headers ?? this.headers,
        queryParameters: queryParameters ?? this.queryParameters,
        timeout: timeout ?? this.timeout,
        skipAuth: skipAuth ?? this.skipAuth,
        retry: retry ?? this.retry,
        retryCount: retryCount ?? this.retryCount,
        traceId: traceId ?? this.traceId,
        extra: extra ?? this.extra,
      );
}

/// Retry policy configuration
@immutable
class RetryPolicy {
  /// Creates a retry policy
  const RetryPolicy({
    this.maxAttempts = 3,
    this.initialDelay = const Duration(milliseconds: 1000),
    this.maxDelay = const Duration(seconds: 30),
    this.backoffMultiplier = 2.0,
    this.retryStatusCodes = const [502, 503, 504],
    this.retryOnTimeout = true,
    this.retryOnConnectionError = true,
  });

  /// No retry policy
  static const none = RetryPolicy(maxAttempts: 0);

  /// Maximum number of retry attempts
  final int maxAttempts;

  /// Initial delay between retries
  final Duration initialDelay;

  /// Maximum delay between retries
  final Duration maxDelay;

  /// Multiplier for exponential backoff
  final double backoffMultiplier;

  /// HTTP status codes that trigger a retry
  final List<int> retryStatusCodes;

  /// Whether to retry on timeout
  final bool retryOnTimeout;

  /// Whether to retry on connection error
  final bool retryOnConnectionError;

  /// Calculate delay for a given attempt number (0-indexed)
  Duration delayForAttempt(int attempt) {
    if (attempt <= 0) return Duration.zero;

    final delay = initialDelay.inMilliseconds *
        (backoffMultiplier == 1.0 ? 1 : _pow(backoffMultiplier, attempt - 1));

    final clampedDelay = delay.clamp(0, maxDelay.inMilliseconds);
    return Duration(milliseconds: clampedDelay.toInt());
  }

  double _pow(double base, int exponent) {
    var result = 1.0;
    for (var i = 0; i < exponent; i++) {
      result *= base;
    }
    return result;
  }
}
