import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('AuthState', () {
    test('初期状態は unauthenticated', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final state = container.read(authProvider);
      expect(state, isA<AuthUnauthenticated>());
    });

    test('login 後は authenticated になる', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      await container.read(authProvider.notifier).login(
        username: 'user@example.com',
        password: 'password',
      );

      final state = container.read(authProvider);
      expect(state, isA<AuthAuthenticated>());
    });

    test('logout 後は unauthenticated になる', () async {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      // まずログイン
      await container.read(authProvider.notifier).login(
        username: 'user@example.com',
        password: 'password',
      );

      // ログアウト
      await container.read(authProvider.notifier).logout();

      final state = container.read(authProvider);
      expect(state, isA<AuthUnauthenticated>());
    });
  });
}
