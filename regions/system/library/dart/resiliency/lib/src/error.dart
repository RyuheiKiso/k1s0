sealed class ResiliencyError implements Exception {
  final String message;
  const ResiliencyError(this.message);

  @override
  String toString() => 'ResiliencyError: $message';
}

class MaxRetriesExceededError extends ResiliencyError {
  final int attempts;
  final Object? lastError;
  const MaxRetriesExceededError(this.attempts, this.lastError)
      : super('max retries exceeded');
}

class CircuitBreakerOpenError extends ResiliencyError {
  final Duration remainingDuration;
  const CircuitBreakerOpenError(this.remainingDuration)
      : super('circuit breaker is open');
}

class BulkheadFullError extends ResiliencyError {
  final int maxConcurrent;
  const BulkheadFullError(this.maxConcurrent) : super('bulkhead full');
}

class TimeoutError extends ResiliencyError {
  final Duration after;
  const TimeoutError(this.after) : super('operation timed out');
}
