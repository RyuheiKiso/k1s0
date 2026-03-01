class QuotaClientError implements Exception {
  final String message;
  const QuotaClientError(this.message);

  @override
  String toString() => 'QuotaClientError: $message';
}

class QuotaExceededError extends QuotaClientError {
  final String quotaId;
  final int remaining;

  QuotaExceededError(this.quotaId, this.remaining)
      : super('Quota exceeded: $quotaId, remaining=$remaining');
}

class QuotaNotFoundError extends QuotaClientError {
  final String quotaId;
  QuotaNotFoundError(this.quotaId) : super('Quota not found: $quotaId');
}

class QuotaConnectionError extends QuotaClientError {
  QuotaConnectionError(String message) : super('Connection error: $message');
}
