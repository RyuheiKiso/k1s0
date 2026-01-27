import '../token/token_pair.dart';
import 'token_storage.dart';

/// In-memory token storage
///
/// Useful for testing or when persistent storage is not needed.
/// Tokens are lost when the app is closed.
class MemoryTokenStorage implements TokenStorage {
  TokenPair? _tokens;

  @override
  Future<TokenPair?> getTokens() async => _tokens;

  @override
  Future<void> saveTokens(TokenPair tokens) async {
    _tokens = tokens;
  }

  @override
  Future<void> clearTokens() async {
    _tokens = null;
  }

  @override
  Future<bool> hasTokens() async => _tokens != null;
}
