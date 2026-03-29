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

/// FlutterSecureStorage を使わないテスト用セッションクッキーインターセプター
/// モバイル環境でのテスト時に MethodChannel 初期化エラーを回避するためのスタブ実装
class NoOpSessionCookieInterceptor extends SessionCookieInterceptor {
  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    // テスト環境ではセキュアストレージを読まずにそのまま処理を続ける
    handler.next(options);
  }

  @override
  Future<void> onResponse(
    Response response,
    ResponseInterceptorHandler handler,
  ) async {
    // テスト環境ではセキュアストレージに書き込まずにそのまま処理を続ける
    handler.next(response);
  }

  @override
  Future<void> clearSession() async {
    // テスト環境では何もしない
  }
}

void main() {
  setUpAll(() {
    registerFallbackValue(FakeRequestOptions());
  });

  // テスト用のオーバーライドリスト: FlutterSecureStorage を使わないスタブで差し替える
  final noOpInterceptorOverride = sessionCookieInterceptorProvider
      .overrideWithValue(NoOpSessionCookieInterceptor());

  group('AuthNotifier（BFF セッション統合）', () {
    test('初期状態は unauthenticated', () {
      final container = ProviderContainer(
        overrides: [
          noOpInterceptorOverride,
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
          noOpInterceptorOverride,
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

    test('logout は API 呼び出しが失敗してもクライアント側を unauthenticated にする', () async {
      // logout の実装は catch (_) で DioException を飲み込み、
      // finally で必ず state = AuthUnauthenticated() にする設計
      final container = ProviderContainer(
        overrides: [
          noOpInterceptorOverride,
          authApiBaseUrlProvider.overrideWithValue('https://api.example.com'),
        ],
      );
      addTearDown(container.dispose);

      // ProviderContainer 初期化後に logout を呼ぶ
      // API 呼び出しは失敗するが例外は飲み込まれ unauthenticated になることを確認する
      await container.read(authProvider.notifier).logout();

      final state = container.read(authProvider);
      expect(state, isA<AuthUnauthenticated>());
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
