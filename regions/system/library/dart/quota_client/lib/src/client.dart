import 'dart:convert';
import 'package:http/http.dart' as http;
import 'config.dart';
import 'error.dart';
import 'model.dart';

abstract class QuotaClient {
  Future<QuotaStatus> check(String quotaId, int amount);
  Future<QuotaUsage> increment(String quotaId, int amount);
  Future<QuotaUsage> getUsage(String quotaId);
  Future<QuotaPolicy> getPolicy(String quotaId);
}

/// quota-server への HTTP REST クライアント実装。
class HttpQuotaClient implements QuotaClient {
  final String _serverUrl;
  final http.Client _httpClient;
  final Duration _timeout;

  HttpQuotaClient(
    String serverUrl, {
    http.Client? httpClient,
    Duration? timeout,
    Duration? policyCacheTtl,
  })  : _serverUrl = serverUrl.replaceAll(RegExp(r'/$'), ''),
        _httpClient = httpClient ?? http.Client(),
        _timeout = timeout ?? const Duration(seconds: 5);

  /// 設定オブジェクトからクライアントを生成するファクトリコンストラクタ。
  factory HttpQuotaClient.fromConfig(
    QuotaClientConfig config, {
    http.Client? httpClient,
  }) {
    return HttpQuotaClient(
      config.serverUrl,
      httpClient: httpClient,
      timeout: config.timeout,
      policyCacheTtl: config.policyCacheTtl,
    );
  }

  Future<Map<String, dynamic>> _request(
    String method,
    String path,
    String quotaId, {
    Map<String, dynamic>? body,
  }) async {
    final uri = Uri.parse('$_serverUrl$path');
    late http.Response response;

    try {
      if (method == 'GET') {
        response = await _httpClient.get(uri).timeout(_timeout);
      } else {
        response = await _httpClient
            .post(
              uri,
              headers: {'Content-Type': 'application/json'},
              body: body != null ? jsonEncode(body) : null,
            )
            .timeout(_timeout);
      }
    } catch (e) {
      throw QuotaConnectionError(e.toString());
    }

    if (response.statusCode == 404) {
      throw QuotaNotFoundError(quotaId);
    }
    if (response.statusCode != 200) {
      throw QuotaConnectionError('Unexpected status: ${response.statusCode}');
    }

    try {
      return jsonDecode(response.body) as Map<String, dynamic>;
    } catch (e) {
      throw QuotaClientError('Invalid response: ${e.toString()}');
    }
  }

  QuotaPeriod _parsePeriod(String value) {
    switch (value) {
      case 'hourly':
        return QuotaPeriod.hourly;
      case 'daily':
        return QuotaPeriod.daily;
      case 'monthly':
        return QuotaPeriod.monthly;
      default:
        return QuotaPeriod.custom;
    }
  }

  @override
  Future<QuotaStatus> check(String quotaId, int amount) async {
    final json = await _request(
      'POST',
      '/api/v1/quotas/${Uri.encodeComponent(quotaId)}/check',
      quotaId,
      body: {'amount': amount},
    );
    return QuotaStatus(
      allowed: json['allowed'] as bool,
      remaining: (json['remaining'] as num).toInt(),
      limit: (json['limit'] as num).toInt(),
      resetAt: DateTime.parse(json['reset_at'] as String),
    );
  }

  @override
  Future<QuotaUsage> increment(String quotaId, int amount) async {
    final json = await _request(
      'POST',
      '/api/v1/quotas/${Uri.encodeComponent(quotaId)}/increment',
      quotaId,
      body: {'amount': amount},
    );
    return QuotaUsage(
      quotaId: json['quota_id'] as String,
      used: (json['used'] as num).toInt(),
      limit: (json['limit'] as num).toInt(),
      period: _parsePeriod(json['period'] as String),
      resetAt: DateTime.parse(json['reset_at'] as String),
    );
  }

  @override
  Future<QuotaUsage> getUsage(String quotaId) async {
    final json = await _request(
      'GET',
      '/api/v1/quotas/${Uri.encodeComponent(quotaId)}/usage',
      quotaId,
    );
    return QuotaUsage(
      quotaId: json['quota_id'] as String,
      used: (json['used'] as num).toInt(),
      limit: (json['limit'] as num).toInt(),
      period: _parsePeriod(json['period'] as String),
      resetAt: DateTime.parse(json['reset_at'] as String),
    );
  }

  @override
  Future<QuotaPolicy> getPolicy(String quotaId) async {
    final json = await _request(
      'GET',
      '/api/v1/quotas/${Uri.encodeComponent(quotaId)}/policy',
      quotaId,
    );
    return QuotaPolicy(
      quotaId: json['quota_id'] as String,
      limit: (json['limit'] as num).toInt(),
      period: _parsePeriod(json['period'] as String),
      resetStrategy: json['reset_strategy'] as String,
    );
  }
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
