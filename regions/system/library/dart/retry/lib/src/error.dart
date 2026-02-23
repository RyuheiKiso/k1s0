class RetryError implements Exception {
  final int attempts;
  final Object lastError;

  const RetryError(this.attempts, this.lastError);

  @override
  String toString() => 'RetryError: exhausted $attempts retries: $lastError';
}
