enum QuotaPeriod { hourly, daily, monthly, custom }

class QuotaStatus {
  final bool allowed;
  final int remaining;
  final int limit;
  final DateTime resetAt;

  const QuotaStatus({
    required this.allowed,
    required this.remaining,
    required this.limit,
    required this.resetAt,
  });
}

class QuotaUsage {
  final String quotaId;
  final int used;
  final int limit;
  final QuotaPeriod period;
  final DateTime resetAt;

  const QuotaUsage({
    required this.quotaId,
    required this.used,
    required this.limit,
    required this.period,
    required this.resetAt,
  });
}

class QuotaPolicy {
  final String quotaId;
  final int limit;
  final QuotaPeriod period;
  final String resetStrategy;

  const QuotaPolicy({
    required this.quotaId,
    required this.limit,
    required this.period,
    required this.resetStrategy,
  });
}
