import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

import 'client.dart';
import 'config.dart';
import 'error.dart';
import 'secret.dart';

class _CacheEntry {
  final Secret secret;
  final DateTime fetchedAt;

  const _CacheEntry(this.secret, this.fetchedAt);
}

/// vault-server の REST API を HTTP で呼び出すクライアント。
class HttpVaultClient implements VaultClient {
  final VaultClientConfig config;
  final http.Client _http;
  final Map<String, _CacheEntry> _cache = {};

  HttpVaultClient(this.config, {http.Client? httpClient})
      : _http = httpClient ?? http.Client();

  bool _isCacheValid(_CacheEntry entry) {
    return DateTime.now().difference(entry.fetchedAt) < config.cacheTtl;
  }

  void _evictIfNeeded() {
    while (_cache.length >= config.cacheMaxCapacity) {
      _cache.remove(_cache.keys.first);
    }
  }

  Secret _parseSecret(Map<String, dynamic> body) {
    return Secret(
      path: body['path'] as String,
      data: Map<String, String>.from(body['data'] as Map),
      version: (body['version'] as num).toInt(),
      createdAt: DateTime.parse(body['created_at'] as String),
    );
  }

  @override
  Future<Secret> getSecret(String path) async {
    final cached = _cache[path];
    if (cached != null && _isCacheValid(cached)) {
      return cached.secret;
    }

    final uri = Uri.parse('${config.serverUrl}/api/v1/secrets/$path');
    final response = await _http.get(uri);

    switch (response.statusCode) {
      case 200:
        final body = json.decode(response.body) as Map<String, dynamic>;
        final secret = _parseSecret(body);
        _evictIfNeeded();
        _cache[path] = _CacheEntry(secret, DateTime.now());
        return secret;
      case 404:
        throw VaultError(VaultErrorCode.notFound, path);
      case 403:
      case 401:
        throw VaultError(VaultErrorCode.permissionDenied, path);
      default:
        throw VaultError(
          VaultErrorCode.serverError,
          'HTTP ${response.statusCode}: ${response.body}',
        );
    }
  }

  @override
  Future<String> getSecretValue(String path, String key) async {
    final secret = await getSecret(path);
    final value = secret.data[key];
    if (value == null) {
      throw VaultError(VaultErrorCode.notFound, '$path/$key');
    }
    return value;
  }

  @override
  Future<List<String>> listSecrets(String pathPrefix) async {
    final uri = Uri.parse('${config.serverUrl}/api/v1/secrets')
        .replace(queryParameters: {'prefix': pathPrefix});
    final response = await _http.get(uri);

    if (response.statusCode != 200) {
      throw VaultError(
        VaultErrorCode.serverError,
        'list_secrets failed: ${response.statusCode}',
      );
    }

    final body = json.decode(response.body) as List<dynamic>;
    return body.cast<String>();
  }

  @override
  Stream<SecretRotatedEvent> watchSecret(String path) async* {
    int? lastVersion;

    while (true) {
      await Future<void>.delayed(config.cacheTtl);
      try {
        final secret = await getSecret(path);
        if (lastVersion != null && secret.version != lastVersion) {
          yield SecretRotatedEvent(path: path, version: secret.version);
        }
        lastVersion = secret.version;
      } catch (_) {
        // skip polling errors
      }
    }
  }
}
