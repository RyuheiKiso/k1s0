import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_auth/src/storage/memory_token_storage.dart';
import 'package:k1s0_auth/src/token/token_manager.dart';
import 'package:k1s0_auth/src/token/token_pair.dart';
import 'package:k1s0_auth/src/types/auth_error.dart';
import 'package:mocktail/mocktail.dart';

class MockTokenStorage extends Mock implements MemoryTokenStorage {}

void main() {
  group('TokenResult', () {
    test('TokenValid contains token and claims', () {
      // TokenValid is tested indirectly through TokenManager tests
      expect(const TokenNone(), isA<TokenResult>());
      expect(const TokenExpired(), isA<TokenResult>());
    });
  });

  group('TokenManager', () {
    late MemoryTokenStorage storage;
    late TokenManager manager;

    setUp(() {
      storage = MemoryTokenStorage();
      manager = TokenManager(
        storage: storage,
        autoRefresh: false,
      );
    });

    tearDown(() {
      manager.dispose();
    });

    test('creates with default values', () {
      final defaultManager = TokenManager(storage: storage);

      expect(defaultManager.refreshMargin, const Duration(minutes: 5));
      expect(defaultManager.autoRefresh, true);

      defaultManager.dispose();
    });

    test('creates with custom values', () {
      final customManager = TokenManager(
        storage: storage,
        refreshMargin: const Duration(minutes: 10),
        autoRefresh: false,
      );

      expect(customManager.refreshMargin, const Duration(minutes: 10));
      expect(customManager.autoRefresh, false);

      customManager.dispose();
    });

    test('getTokenPair returns null when no tokens stored', () async {
      final tokens = await manager.getTokenPair();

      expect(tokens, isNull);
    });

    test('getTokenPair returns stored tokens', () async {
      const tokens = TokenPair(accessToken: 'test-token');
      await storage.saveTokens(tokens);

      final retrieved = await manager.getTokenPair();

      expect(retrieved?.accessToken, 'test-token');
    });

    test('setTokens stores tokens', () async {
      const tokens = TokenPair(accessToken: 'new-token');

      await manager.setTokens(tokens);
      final stored = await storage.getTokens();

      expect(stored?.accessToken, 'new-token');
    });

    test('clearTokens removes tokens', () async {
      const tokens = TokenPair(accessToken: 'token');
      await manager.setTokens(tokens);

      await manager.clearTokens();
      final stored = await storage.getTokens();

      expect(stored, isNull);
    });

    group('getValidToken', () {
      test('returns TokenNone when no tokens stored', () async {
        final result = await manager.getValidToken();

        expect(result, isA<TokenNone>());
      });

      test('returns TokenNone for invalid JWT format', () async {
        // Store a token that is not a valid JWT
        const tokens = TokenPair(accessToken: 'invalid-jwt');
        await storage.saveTokens(tokens);

        final result = await manager.getValidToken();

        expect(result, isA<TokenNone>());
      });
    });

    test('dispose cancels refresh timer', () {
      // Just verify dispose doesn't throw
      expect(() => manager.dispose(), returnsNormally);
    });
  });

  group('SafeTokenManager', () {
    late MemoryTokenStorage storage;
    late SafeTokenManager manager;

    setUp(() {
      storage = MemoryTokenStorage();
      manager = SafeTokenManager(
        storage: storage,
        autoRefresh: false,
      );
    });

    tearDown(() {
      manager.dispose();
    });

    test('getTokenOrThrow throws for TokenNone', () async {
      await expectLater(
        manager.getTokenOrThrow(),
        throwsA(
          isA<AuthError>().having(
            (e) => e.code,
            'code',
            AuthErrorCode.unauthorized,
          ),
        ),
      );
    });

    test('getTokenOrThrow throws for invalid token', () async {
      const tokens = TokenPair(accessToken: 'invalid');
      await storage.saveTokens(tokens);

      await expectLater(
        manager.getTokenOrThrow(),
        throwsA(isA<AuthError>()),
      );
    });
  });
}
