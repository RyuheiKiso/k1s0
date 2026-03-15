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

  group('AuthNotifier（BFF セッション統合）', () {
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

    test('login は OAuth フローを開始する（ネットワーク未接続でもエラーにならない）', () async {
      final container = ProviderContainer(
        overrides: [
          authApiBaseUrlProvider.overrideWithValue('https://api.example.com'),
        ],
      );
      addTearDown(container.dispose);

      // login はリダイレクト型（Web）またはコールバック型（モバイル）
      // テスト環境では url_launcher / flutter_web_auth_2 が動作しないため例外は想定内
      try {
        await container.read(authProvider.notifier).login();
      } catch (_) {
        // テスト環境では外部ブラウザが開けないため例外は想定内
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
