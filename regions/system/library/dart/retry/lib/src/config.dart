import 'dart:math';

class RetryConfig {
  final int maxAttempts;
  final int initialDelayMs;
  final int maxDelayMs;
  final double multiplier;
  final bool jitter;

  const RetryConfig({
    this.maxAttempts = 3,
    this.initialDelayMs = 100,
    this.maxDelayMs = 30000,
    this.multiplier = 2.0,
    this.jitter = true,
  });
}

int computeDelay(RetryConfig config, int attempt) {
  final base = config.initialDelayMs * pow(config.multiplier, attempt);
  final capped = min(base.toDouble(), config.maxDelayMs.toDouble());
  if (config.jitter) {
    final jitterRange = capped * 0.1;
    final random = Random();
    return (capped - jitterRange + random.nextDouble() * jitterRange * 2)
        .round();
  }
  return capped.round();
}
