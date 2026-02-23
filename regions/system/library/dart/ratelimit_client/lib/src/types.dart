class RateLimitStatus {
  final bool allowed;
  final int remaining;
  final DateTime resetAt;
  final int? retryAfterSecs;

  const RateLimitStatus({
    required this.allowed,
    required this.remaining,
    required this.resetAt,
    this.retryAfterSecs,
  });
}

class RateLimitResult {
  final int remaining;
  final DateTime resetAt;

  const RateLimitResult({
    required this.remaining,
    required this.resetAt,
  });
}

class RateLimitPolicy {
  final String key;
  final int limit;
  final int windowSecs;
  final String algorithm;

  const RateLimitPolicy({
    required this.key,
    required this.limit,
    required this.windowSecs,
    required this.algorithm,
  });
}
