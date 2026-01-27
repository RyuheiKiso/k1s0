import 'dart:async';

import 'package:flutter/foundation.dart';

import '../storage/token_storage.dart';
import '../types/auth_error.dart';
import 'claims.dart';
import 'token_decoder.dart';
import 'token_pair.dart';

/// Token refresh callback
typedef TokenRefresher = Future<TokenPair?> Function(String refreshToken);

/// Token result type
sealed class TokenResult {
  const TokenResult();
}

/// Token is valid
class TokenValid extends TokenResult {
  /// Creates a valid token result
  const TokenValid(this.token, this.claims);

  /// The access token
  final String token;

  /// The decoded claims
  final Claims claims;
}

/// Token was refreshed
class TokenRefreshed extends TokenResult {
  /// Creates a refreshed token result
  const TokenRefreshed(this.token, this.claims);

  /// The new access token
  final String token;

  /// The decoded claims from the new token
  final Claims claims;
}

/// Token is expired and cannot be refreshed
class TokenExpired extends TokenResult {
  /// Creates an expired token result
  const TokenExpired();
}

/// No token available
class TokenNone extends TokenResult {
  /// Creates a no token result
  const TokenNone();
}

/// Token manager for handling token lifecycle
class TokenManager {
  /// Creates a token manager
  TokenManager({
    required this.storage,
    this.refresher,
    this.refreshMargin = const Duration(minutes: 5),
    this.autoRefresh = true,
  });

  /// Token storage
  final TokenStorage storage;

  /// Token refresh callback
  final TokenRefresher? refresher;

  /// Time before expiration to trigger refresh
  final Duration refreshMargin;

  /// Whether to automatically refresh tokens
  final bool autoRefresh;

  Timer? _refreshTimer;
  bool _isRefreshing = false;
  final _refreshLock = Completer<void>.sync();

  /// Get the current token pair
  Future<TokenPair?> getTokenPair() => storage.getTokens();

  /// Get a valid access token
  ///
  /// Will attempt to refresh if the token is expired or about to expire.
  Future<TokenResult> getValidToken() async {
    final tokens = await storage.getTokens();

    if (tokens == null) {
      return const TokenNone();
    }

    // Decode the token to get claims
    final claims = TokenDecoder.tryDecode(tokens.accessToken);
    if (claims == null) {
      return const TokenNone();
    }

    // Check if token is valid
    if (!claims.isExpired && !claims.willExpireIn(refreshMargin)) {
      return TokenValid(tokens.accessToken, claims);
    }

    // Token is expired or about to expire, try to refresh
    if (tokens.hasRefreshToken && refresher != null) {
      try {
        final newTokens = await _refreshToken(tokens.refreshToken!);
        if (newTokens != null) {
          final newClaims = TokenDecoder.tryDecode(newTokens.accessToken);
          if (newClaims != null && !newClaims.isExpired) {
            return TokenRefreshed(newTokens.accessToken, newClaims);
          }
        }
      } on Exception catch (e) {
        debugPrint('Token refresh failed: $e');
      }
    }

    // Token is expired and cannot be refreshed
    if (claims.isExpired) {
      return const TokenExpired();
    }

    // Token is about to expire but refresh failed, return it anyway
    return TokenValid(tokens.accessToken, claims);
  }

  /// Set tokens
  Future<void> setTokens(TokenPair tokens) async {
    await storage.saveTokens(tokens);
    _scheduleRefresh(tokens);
  }

  /// Clear tokens
  Future<void> clearTokens() async {
    _cancelRefreshTimer();
    await storage.clearTokens();
  }

  /// Refresh the token manually
  Future<TokenPair?> refreshNow() async {
    final tokens = await storage.getTokens();
    if (tokens?.refreshToken == null) {
      return null;
    }
    return _refreshToken(tokens!.refreshToken!);
  }

  Future<TokenPair?> _refreshToken(String refreshToken) async {
    if (_isRefreshing) {
      // Wait for ongoing refresh to complete
      await _refreshLock.future;
      return storage.getTokens();
    }

    _isRefreshing = true;
    try {
      final newTokens = await refresher?.call(refreshToken);
      if (newTokens != null) {
        await storage.saveTokens(newTokens);
        _scheduleRefresh(newTokens);
        return newTokens;
      }
      return null;
    } finally {
      _isRefreshing = false;
    }
  }

  void _scheduleRefresh(TokenPair tokens) {
    if (!autoRefresh || refresher == null) return;

    _cancelRefreshTimer();

    // Calculate when to refresh
    final claims = TokenDecoder.tryDecode(tokens.accessToken);
    if (claims == null) return;

    final expiresAt = claims.expirationTime;
    final refreshAt = expiresAt.subtract(refreshMargin);
    final delay = refreshAt.difference(DateTime.now());

    if (delay.isNegative) {
      // Token already needs refresh
      if (tokens.hasRefreshToken) {
        _refreshToken(tokens.refreshToken!);
      }
      return;
    }

    _refreshTimer = Timer(delay, () {
      if (tokens.hasRefreshToken) {
        _refreshToken(tokens.refreshToken!);
      }
    });
  }

  void _cancelRefreshTimer() {
    _refreshTimer?.cancel();
    _refreshTimer = null;
  }

  /// Dispose the token manager
  void dispose() {
    _cancelRefreshTimer();
  }
}

/// Token manager that throws AuthError on failures
class SafeTokenManager extends TokenManager {
  /// Creates a safe token manager
  SafeTokenManager({
    required super.storage,
    super.refresher,
    super.refreshMargin,
    super.autoRefresh,
  });

  /// Get a valid token or throw an AuthError
  Future<String> getTokenOrThrow() async {
    final result = await getValidToken();

    switch (result) {
      case TokenValid(:final token):
        return token;
      case TokenRefreshed(:final token):
        return token;
      case TokenExpired():
        throw AuthError(
          code: AuthErrorCode.tokenExpired,
          message: 'Token has expired',
        );
      case TokenNone():
        throw AuthError(
          code: AuthErrorCode.unauthorized,
          message: 'No authentication token',
        );
    }
  }
}
