/// Error types used by the resiliency library.
/// Consumers can catch [ResiliencyError] and its subclasses.
library;

/// Base error for resiliency operations.
sealed class ResiliencyError implements Exception {
  final String message;
  const ResiliencyError(this.message);

  @override
  String toString() => 'ResiliencyError: $message';
}

/// Raised when retry attempts are exhausted.
class MaxRetriesExceededError extends ResiliencyError {
  /// Total number of attempted executions.
  final int attempts;

  /// Last observed error, if any.
  final Object? lastError;

  const MaxRetriesExceededError(this.attempts, this.lastError)
      : super('max retries exceeded');
}

/// Raised when a circuit breaker rejects execution while open.
class CircuitBreakerOpenError extends ResiliencyError {
  /// Remaining time until the breaker may allow calls again.
  final Duration remainingDuration;

  const CircuitBreakerOpenError(this.remainingDuration)
      : super('circuit breaker is open');
}

/// Raised when the bulkhead has no remaining capacity.
class BulkheadFullError extends ResiliencyError {
  /// Configured maximum concurrent operations.
  final int maxConcurrent;

  const BulkheadFullError(this.maxConcurrent) : super('bulkhead full');
}

/// Raised when an operation exceeds the configured timeout.
class TimeoutError extends ResiliencyError {
  /// Timeout duration that was exceeded.
  final Duration after;

  const TimeoutError(this.after) : super('operation timed out');
}
