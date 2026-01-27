import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/storage/memory_token_storage.dart';
import 'package:k1s0_auth/src/storage/token_storage.dart';
import 'package:k1s0_auth/src/token/token_pair.dart';

void main() {
  group('MemoryTokenStorage', () {
    late TokenStorage storage;

    setUp(() {
      storage = MemoryTokenStorage();
    });

    test('getTokens returns null initially', () async {
      final tokens = await storage.getTokens();

      expect(tokens, isNull);
    });

    test('hasTokens returns false initially', () async {
      final hasTokens = await storage.hasTokens();

      expect(hasTokens, false);
    });

    test('saveTokens stores tokens', () async {
      const tokens = TokenPair(
        accessToken: 'access-123',
        refreshToken: 'refresh-456',
      );

      await storage.saveTokens(tokens);
      final retrieved = await storage.getTokens();

      expect(retrieved, isNotNull);
      expect(retrieved!.accessToken, 'access-123');
      expect(retrieved.refreshToken, 'refresh-456');
    });

    test('hasTokens returns true after saving tokens', () async {
      const tokens = TokenPair(accessToken: 'access');

      await storage.saveTokens(tokens);
      final hasTokens = await storage.hasTokens();

      expect(hasTokens, true);
    });

    test('clearTokens removes tokens', () async {
      const tokens = TokenPair(accessToken: 'access');
      await storage.saveTokens(tokens);

      await storage.clearTokens();
      final retrieved = await storage.getTokens();
      final hasTokens = await storage.hasTokens();

      expect(retrieved, isNull);
      expect(hasTokens, false);
    });

    test('saveTokens overwrites existing tokens', () async {
      const tokens1 = TokenPair(accessToken: 'access-1');
      const tokens2 = TokenPair(accessToken: 'access-2');

      await storage.saveTokens(tokens1);
      await storage.saveTokens(tokens2);
      final retrieved = await storage.getTokens();

      expect(retrieved!.accessToken, 'access-2');
    });
  });
}
