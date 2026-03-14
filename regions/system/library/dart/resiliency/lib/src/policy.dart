/// Policy configuration types for the resiliency library.
/// [ResiliencyPolicy] combines retry, circuit breaker, bulkhead, and timeout.
library;

/// Retry policy configuration.
class RetryConfig {
  /// Maximum number of attempts, including the first execution.
  final int maxAttempts;

  /// Initial backoff delay.
  final Duration baseDelay;

  /// Maximum backoff delay.
  final Duration maxDelay;

  /// Whether randomized jitter is applied to retry delays.
  final bool jitter;

  const RetryConfig({
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 100),
    this.maxDelay = const Duration(seconds: 5),
    this.jitter = true,
  });
}

/// Circuit breaker configuration.
class CircuitBreakerConfig {
  /// Number of consecutive failures required to open the circuit.
  final int failureThreshold;

  /// Time to wait before entering half-open state.
  final Duration recoveryTimeout;

  /// Number of trial calls allowed while half-open.
  final int halfOpenMaxCalls;

  const CircuitBreakerConfig({
    this.failureThreshold = 5,
    this.recoveryTimeout = const Duration(seconds: 30),
    this.halfOpenMaxCalls = 2,
  });
}

/// Bulkhead configuration.
class BulkheadConfig {
  /// Maximum number of concurrent calls.
  final int maxConcurrentCalls;

  /// Maximum time to wait for an available slot.
  final Duration maxWaitDuration;

  const BulkheadConfig({
    this.maxConcurrentCalls = 20,
    this.maxWaitDuration = const Duration(milliseconds: 500),
  });
}

/// Combined resiliency policy.
/// Any field may be null to disable that policy.
class ResiliencyPolicy {
  /// Retry policy. When null, retries are disabled.
  final RetryConfig? retry;

  /// Circuit breaker policy. When null, it is disabled.
  final CircuitBreakerConfig? circuitBreaker;

  /// Bulkhead policy. When null, it is disabled.
  final BulkheadConfig? bulkhead;

  /// Timeout policy. When null, no timeout is applied.
  final Duration? timeout;

  const ResiliencyPolicy({
    this.retry,
    this.circuitBreaker,
    this.bulkhead,
    this.timeout,
  });
}
