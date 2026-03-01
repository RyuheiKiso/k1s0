import 'dart:convert';

import 'package:http/http.dart' as http;

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

RateLimitError _parseError(int statusCode, String body, String op) {
  final msg = body.trim().isNotEmpty ? body.trim() : 'status $statusCode';
  if (statusCode == 404) {
    return RateLimitError('Key not found: $msg', code: 'KEY_NOT_FOUND');
  }
  if (statusCode == 429) {
    Map<String, dynamic>? parsed;
    try {
      parsed = json.decode(body) as Map<String, dynamic>?;
    } catch (_) {}
    final retry = (parsed?['retry_after_secs'] as num?)?.toInt();
    return RateLimitError(
      '$op rate limit exceeded: $msg',
      code: 'LIMIT_EXCEEDED',
      retryAfterSecs: retry,
    );
  }
  if (statusCode == 408 || statusCode == 504) {
    return RateLimitError('$op timed out', code: 'TIMEOUT');
  }
  return RateLimitError('$op failed ($statusCode): $msg', code: 'SERVER_ERROR');
}

/// ratelimit-server に HTTP で接続するクライアント。
/// [serverAddress] には "http://host:port" または "host:port" 形式を指定する。
class GrpcRateLimitClient implements RateLimitClient {
  final String _baseUrl;
  final http.Client _httpClient;

  GrpcRateLimitClient(String serverAddress, {http.Client? httpClient})
      : _baseUrl = serverAddress.startsWith('http')
            ? serverAddress.replaceAll(RegExp(r'/$'), '')
            : 'http://${serverAddress.replaceAll(RegExp(r'/$'), '')}',
        _httpClient = httpClient ?? http.Client();

  Future<Map<String, dynamic>> _post(String path, Map<String, dynamic> body) async {
    final uri = Uri.parse('$_baseUrl$path');
    final response = await _httpClient.post(
      uri,
      headers: {'Content-Type': 'application/json'},
      body: json.encode(body),
    );
    if (response.statusCode >= 200 && response.statusCode < 300) {
      return json.decode(response.body) as Map<String, dynamic>;
    }
    throw _parseError(response.statusCode, response.body, 'POST $path');
  }

  Future<Map<String, dynamic>> _get(String path) async {
    final uri = Uri.parse('$_baseUrl$path');
    final response = await _httpClient.get(uri);
    if (response.statusCode >= 200 && response.statusCode < 300) {
      return json.decode(response.body) as Map<String, dynamic>;
    }
    throw _parseError(response.statusCode, response.body, 'GET $path');
  }

  @override
  Future<RateLimitStatus> check(String key, int cost) async {
    final encodedKey = Uri.encodeComponent(key);
    final data = await _post('/api/v1/ratelimit/$encodedKey/check', {'cost': cost});
    return RateLimitStatus(
      allowed: data['allowed'] as bool,
      remaining: (data['remaining'] as num).toInt(),
      resetAt: DateTime.parse(data['reset_at'] as String),
      retryAfterSecs: (data['retry_after_secs'] as num?)?.toInt(),
    );
  }

  @override
  Future<RateLimitResult> consume(String key, int cost) async {
    final encodedKey = Uri.encodeComponent(key);
    final data = await _post('/api/v1/ratelimit/$encodedKey/consume', {'cost': cost});
    return RateLimitResult(
      remaining: (data['remaining'] as num).toInt(),
      resetAt: DateTime.parse(data['reset_at'] as String),
    );
  }

  @override
  Future<RateLimitPolicy> getLimit(String key) async {
    final encodedKey = Uri.encodeComponent(key);
    final data = await _get('/api/v1/ratelimit/$encodedKey/policy');
    return RateLimitPolicy(
      key: data['key'] as String,
      limit: (data['limit'] as num).toInt(),
      windowSecs: (data['window_secs'] as num).toInt(),
      algorithm: data['algorithm'] as String,
    );
  }

  Future<void> close() async {
    _httpClient.close();
  }
}
