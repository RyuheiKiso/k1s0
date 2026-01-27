import '../token/token_pair.dart';

/// Abstract token storage interface
abstract class TokenStorage {
  /// Get stored tokens
  Future<TokenPair?> getTokens();

  /// Save tokens
  Future<void> saveTokens(TokenPair tokens);

  /// Clear stored tokens
  Future<void> clearTokens();

  /// Check if tokens are stored
  Future<bool> hasTokens();
}
