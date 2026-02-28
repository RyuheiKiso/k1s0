import 'types.dart';
import 'error.dart';

abstract class RateLimitClient {
  Future<RateLimitStatus> check(String key, int cost);
  Future<RateLimitResult> consume(String key, int cost);
  Future<RateLimitPolicy> getLimit(String key);
}

class InMemoryRateLimitClient implements RateLimitClient {
  final Map<String, int> _counters = {};
  final Map<String, RateLimitPolicy> _policies = {};

  static const _defaultPolicy = RateLimitPolicy(
    key: 'default',
    limit: 100,
    windowSecs: 3600,
    algorithm: 'token_bucket',
  );

  void setPolicy(String key, RateLimitPolicy policy) {
    _policies[key] = policy;
  }

  RateLimitPolicy _getPolicy(String key) {
    return _policies[key] ?? _defaultPolicy;
  }

  @override
  Future<RateLimitStatus> check(String key, int cost) async {
    final policy = _getPolicy(key);
    final used = _counters[key] ?? 0;
    final resetAt = DateTime.now().add(Duration(seconds: policy.windowSecs));

    if (used + cost > policy.limit) {
      return RateLimitStatus(
        allowed: false,
        remaining: 0,
        resetAt: resetAt,
        retryAfterSecs: policy.windowSecs,
      );
    }

    return RateLimitStatus(
      allowed: true,
      remaining: policy.limit - used - cost,
      resetAt: resetAt,
    );
  }

  @override
  Future<RateLimitResult> consume(String key, int cost) async {
    final policy = _getPolicy(key);
    final used = _counters[key] ?? 0;

    if (used + cost > policy.limit) {
      throw RateLimitError(
        'Rate limit exceeded for key: $key',
        code: 'LIMIT_EXCEEDED',
        retryAfterSecs: policy.windowSecs,
      );
    }

    _counters[key] = used + cost;
    final remaining = policy.limit - (used + cost);
    final resetAt = DateTime.now().add(Duration(seconds: policy.windowSecs));

    return RateLimitResult(remaining: remaining, resetAt: resetAt);
  }

  @override
  Future<RateLimitPolicy> getLimit(String key) async {
    return _getPolicy(key);
  }

  int getUsedCount(String key) {
    return _counters[key] ?? 0;
  }
}

/// gRPC 経由で ratelimit-server に接続するクライアント。
/// [serverAddress] には "host:port" 形式のアドレスを指定する（例: "ratelimit-server:8080"）。
class GrpcRateLimitClient implements RateLimitClient {
  final String serverAddress;

  GrpcRateLimitClient(this.serverAddress);

  @override
  Future<RateLimitStatus> check(String key, int cost) async {
    throw RateLimitError(
      'gRPC client not yet connected',
      code: 'SERVER_ERROR',
    );
  }

  @override
  Future<RateLimitResult> consume(String key, int cost) async {
    throw RateLimitError(
      'gRPC client not yet connected',
      code: 'SERVER_ERROR',
    );
  }

  @override
  Future<RateLimitPolicy> getLimit(String key) async {
    throw RateLimitError(
      'gRPC client not yet connected',
      code: 'SERVER_ERROR',
    );
  }

  Future<void> close() async {
    // 接続クリーンアップ用プレースホルダー
  }
}
