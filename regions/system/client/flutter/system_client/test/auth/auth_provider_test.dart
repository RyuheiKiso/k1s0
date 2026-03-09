import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

class MockDio extends Mock implements Dio {
  @override
  BaseOptions get options => BaseOptions(baseUrl: 'https://api.example.com');

  @override
  Interceptors get interceptors => Interceptors();
}

class FakeRequestOptions extends Fake implements RequestOptions {}

void main() {
  setUpAll(() {
    registerFallbackValue(FakeRequestOptions());
  });

  group('AuthNotifier（API 統合）', () {
    test('初期状態は unauthenticated', () {
      final container = ProviderContainer(
        overrides: [
          authApiBaseUrlProvider.overrideWithValue('https://api.example.com'),
        ],
      );
      addTearDown(container.dispose);

      final state = container.read(authProvider);
      expect(state, isA<AuthUnauthenticated>());
    });

    test('login が DioException を投げた場合は状態が変わらない', () async {
      final container = ProviderContainer(
        overrides: [
          authApiBaseUrlProvider.overrideWithValue('https://api.example.com'),
        ],
      );
      addTearDown(container.dispose);

      // API 呼び出しは実際にはネットワークエラーになるが、
      // 状態は unauthenticated のまま
      try {
        await container.read(authProvider.notifier).login(
          username: 'user@example.com',
          password: 'password',
        );
      } catch (_) {
        // ネットワークエラーは想定内
      }

      final state = container.read(authProvider);
      expect(state, isA<AuthUnauthenticated>());
    });

    test('logout が DioException を投げた場合もエラーが伝播する', () async {
      final container = ProviderContainer(
        overrides: [
          authApiBaseUrlProvider.overrideWithValue('https://api.example.com'),
        ],
      );
      addTearDown(container.dispose);

      // logout は API 呼び出しが失敗するとエラーになる
      expect(
        () => container.read(authProvider.notifier).logout(),
        throwsA(isA<DioException>()),
      );
    });
  });

  group('AuthState', () {
    test('AuthUnauthenticated の等値性', () {
      expect(
        const AuthUnauthenticated(),
        equals(const AuthUnauthenticated()),
      );
    });

    test('AuthAuthenticated の等値性', () {
      expect(
        const AuthAuthenticated(userId: 'user-1'),
        equals(const AuthAuthenticated(userId: 'user-1')),
      );
      expect(
        const AuthAuthenticated(userId: 'user-1'),
        isNot(equals(const AuthAuthenticated(userId: 'user-2'))),
      );
    });
  });
}
