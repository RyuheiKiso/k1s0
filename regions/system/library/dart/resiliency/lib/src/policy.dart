class RetryConfig {
  final int maxAttempts;
  final Duration baseDelay;
  final Duration maxDelay;
  final bool jitter;

  const RetryConfig({
    this.maxAttempts = 3,
    this.baseDelay = const Duration(milliseconds: 100),
    this.maxDelay = const Duration(seconds: 5),
    this.jitter = true,
  });
}

class CircuitBreakerConfig {
  final int failureThreshold;
  final Duration recoveryTimeout;
  final int halfOpenMaxCalls;

  const CircuitBreakerConfig({
    this.failureThreshold = 5,
    this.recoveryTimeout = const Duration(seconds: 30),
    this.halfOpenMaxCalls = 2,
  });
}

class BulkheadConfig {
  final int maxConcurrentCalls;
  final Duration maxWaitDuration;

  const BulkheadConfig({
    this.maxConcurrentCalls = 20,
    this.maxWaitDuration = const Duration(milliseconds: 500),
  });
}

class ResiliencyPolicy {
  final RetryConfig? retry;
  final CircuitBreakerConfig? circuitBreaker;
  final BulkheadConfig? bulkhead;
  final Duration? timeout;

  const ResiliencyPolicy({
    this.retry,
    this.circuitBreaker,
    this.bulkhead,
    this.timeout,
  });
}
