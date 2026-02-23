import 'model.dart';

abstract class QuotaClient {
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

class InMemoryQuotaClient implements QuotaClient {
  final Map<String, _UsageEntry> _usages = {};
  final Map<String, QuotaPolicy> _policies = {};

  void setPolicy(String quotaId, QuotaPolicy policy) {
    _policies[quotaId] = policy;
  }

  _UsageEntry _getOrCreateUsage(String quotaId) {
    return _usages.putIfAbsent(quotaId, () {
      final policy = _policies[quotaId];
      return _UsageEntry(
        quotaId: quotaId,
        used: 0,
        limit: policy?.limit ?? 1000,
        period: policy?.period ?? QuotaPeriod.daily,
        resetAt: DateTime.now().add(const Duration(days: 1)),
      );
    });
  }

  @override
  Future<QuotaStatus> check(String quotaId, int amount) async {
    final usage = _getOrCreateUsage(quotaId);
    final remaining = usage.limit - usage.used;
    return QuotaStatus(
      allowed: amount <= remaining,
      remaining: remaining,
      limit: usage.limit,
      resetAt: usage.resetAt,
    );
  }

  @override
  Future<QuotaUsage> increment(String quotaId, int amount) async {
    final usage = _getOrCreateUsage(quotaId);
    usage.used += amount;
    return QuotaUsage(
      quotaId: usage.quotaId,
      used: usage.used,
      limit: usage.limit,
      period: usage.period,
      resetAt: usage.resetAt,
    );
  }

  @override
  Future<QuotaUsage> getUsage(String quotaId) async {
    final usage = _getOrCreateUsage(quotaId);
    return QuotaUsage(
      quotaId: usage.quotaId,
      used: usage.used,
      limit: usage.limit,
      period: usage.period,
      resetAt: usage.resetAt,
    );
  }

  @override
  Future<QuotaPolicy> getPolicy(String quotaId) async {
    final policy = _policies[quotaId];
    if (policy != null) return policy;
    return QuotaPolicy(
      quotaId: quotaId,
      limit: 1000,
      period: QuotaPeriod.daily,
      resetStrategy: 'fixed',
    );
  }
}

class CachedQuotaClient implements QuotaClient {
  final QuotaClient _inner;
  final Duration _policyTtl;
  final Map<String, _PolicyCacheEntry> _cache = {};

  CachedQuotaClient(this._inner, this._policyTtl);

  @override
  Future<QuotaStatus> check(String quotaId, int amount) =>
      _inner.check(quotaId, amount);

  @override
  Future<QuotaUsage> increment(String quotaId, int amount) =>
      _inner.increment(quotaId, amount);

  @override
  Future<QuotaUsage> getUsage(String quotaId) => _inner.getUsage(quotaId);

  @override
  Future<QuotaPolicy> getPolicy(String quotaId) async {
    final cached = _cache[quotaId];
    if (cached != null && DateTime.now().isBefore(cached.expiresAt)) {
      return cached.policy;
    }
    final policy = await _inner.getPolicy(quotaId);
    _cache[quotaId] = _PolicyCacheEntry(
      policy: policy,
      expiresAt: DateTime.now().add(_policyTtl),
    );
    return policy;
  }
}

class _UsageEntry {
  final String quotaId;
  int used;
  final int limit;
  final QuotaPeriod period;
  final DateTime resetAt;

  _UsageEntry({
    required this.quotaId,
    required this.used,
    required this.limit,
    required this.period,
    required this.resetAt,
  });
}

class _PolicyCacheEntry {
  final QuotaPolicy policy;
  final DateTime expiresAt;

  _PolicyCacheEntry({required this.policy, required this.expiresAt});
}
