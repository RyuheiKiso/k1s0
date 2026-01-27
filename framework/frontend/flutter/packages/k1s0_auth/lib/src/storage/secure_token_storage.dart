import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import '../token/token_pair.dart';
import 'token_storage.dart';

/// Secure token storage using flutter_secure_storage
class SecureTokenStorage implements TokenStorage {
  /// Creates a secure token storage
  SecureTokenStorage({
    FlutterSecureStorage? storage,
    this.keyPrefix = 'k1s0_auth',
  }) : _storage = storage ??
            const FlutterSecureStorage(
              aOptions: AndroidOptions(
                encryptedSharedPreferences: true,
              ),
              iOptions: IOSOptions(
                accessibility: KeychainAccessibility.first_unlock_this_device,
              ),
            );

  final FlutterSecureStorage _storage;

  /// Key prefix for stored values
  final String keyPrefix;

  String get _accessTokenKey => '${keyPrefix}_access_token';
  String get _refreshTokenKey => '${keyPrefix}_refresh_token';
  String get _idTokenKey => '${keyPrefix}_id_token';
  String get _expiresAtKey => '${keyPrefix}_expires_at';
  String get _tokenTypeKey => '${keyPrefix}_token_type';
  String get _scopeKey => '${keyPrefix}_scope';

  @override
  Future<TokenPair?> getTokens() async {
    final accessToken = await _storage.read(key: _accessTokenKey);
    if (accessToken == null) return null;

    final refreshToken = await _storage.read(key: _refreshTokenKey);
    final idToken = await _storage.read(key: _idTokenKey);
    final expiresAtStr = await _storage.read(key: _expiresAtKey);
    final tokenType = await _storage.read(key: _tokenTypeKey);
    final scope = await _storage.read(key: _scopeKey);

    return TokenPair(
      accessToken: accessToken,
      refreshToken: refreshToken,
      idToken: idToken,
      expiresAt: expiresAtStr != null ? int.tryParse(expiresAtStr) : null,
      tokenType: tokenType ?? 'Bearer',
      scope: scope,
    );
  }

  @override
  Future<void> saveTokens(TokenPair tokens) async {
    await _storage.write(key: _accessTokenKey, value: tokens.accessToken);

    if (tokens.refreshToken != null) {
      await _storage.write(key: _refreshTokenKey, value: tokens.refreshToken);
    }

    if (tokens.idToken != null) {
      await _storage.write(key: _idTokenKey, value: tokens.idToken);
    }

    if (tokens.expiresAt != null) {
      await _storage.write(
        key: _expiresAtKey,
        value: tokens.expiresAt.toString(),
      );
    }

    await _storage.write(key: _tokenTypeKey, value: tokens.tokenType);

    if (tokens.scope != null) {
      await _storage.write(key: _scopeKey, value: tokens.scope);
    }
  }

  @override
  Future<void> clearTokens() async {
    await _storage.delete(key: _accessTokenKey);
    await _storage.delete(key: _refreshTokenKey);
    await _storage.delete(key: _idTokenKey);
    await _storage.delete(key: _expiresAtKey);
    await _storage.delete(key: _tokenTypeKey);
    await _storage.delete(key: _scopeKey);
  }

  @override
  Future<bool> hasTokens() async {
    final accessToken = await _storage.read(key: _accessTokenKey);
    return accessToken != null;
  }
}

/// JSON-based secure token storage (stores entire TokenPair as JSON)
class JsonSecureTokenStorage implements TokenStorage {
  /// Creates a JSON secure token storage
  JsonSecureTokenStorage({
    FlutterSecureStorage? storage,
    this.key = 'k1s0_auth_tokens',
  }) : _storage = storage ??
            const FlutterSecureStorage(
              aOptions: AndroidOptions(
                encryptedSharedPreferences: true,
              ),
              iOptions: IOSOptions(
                accessibility: KeychainAccessibility.first_unlock_this_device,
              ),
            );

  final FlutterSecureStorage _storage;

  /// Storage key
  final String key;

  @override
  Future<TokenPair?> getTokens() async {
    final json = await _storage.read(key: key);
    if (json == null) return null;

    try {
      final map = jsonDecode(json) as Map<String, dynamic>;
      return TokenPair.fromJson(map);
    } on FormatException {
      return null;
    } on Exception {
      return null;
    }
  }

  @override
  Future<void> saveTokens(TokenPair tokens) async {
    final json = jsonEncode(tokens.toJson());
    await _storage.write(key: key, value: json);
  }

  @override
  Future<void> clearTokens() async {
    await _storage.delete(key: key);
  }

  @override
  Future<bool> hasTokens() async {
    final json = await _storage.read(key: key);
    return json != null;
  }
}
