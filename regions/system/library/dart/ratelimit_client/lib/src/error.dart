class RateLimitError implements Exception {
  final String message;
  final String code;
  final int? retryAfterSecs;

  const RateLimitError(this.message, {this.code = 'UNKNOWN', this.retryAfterSecs});

  @override
  String toString() => 'RateLimitError($code): $message';
}
