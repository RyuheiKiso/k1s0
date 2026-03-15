import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('ApiClient', () {
    test('Dio インスタンスを返す', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(client, isA<Dio>());
    });

    test('baseUrl が設定される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(client.options.baseUrl, equals('https://api.example.com'));
    });

    test('Content-Type ヘッダーが設定される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      expect(
        client.options.headers['Content-Type'],
        equals('application/json'),
      );
    });

    test('csrfTokenProvider なしではエラーハンドラのみ追加される', () {
      final client = ApiClient.create(baseUrl: 'https://api.example.com');
      final customInterceptors = client.interceptors
          .where((i) => i is InterceptorsWrapper || i is CsrfTokenInterceptor)
          .toList();
      expect(customInterceptors.length, equals(1));
      expect(customInterceptors.first, isA<InterceptorsWrapper>());
    });

    test('csrfTokenProvider ありでは CSRF + エラーハンドラの2つが追加される', () {
      final client = ApiClient.create(
        baseUrl: 'https://api.example.com',
        csrfTokenProvider: () async => 'test-token',
      );
      final customInterceptors = client.interceptors
          .where((i) => i is InterceptorsWrapper || i is CsrfTokenInterceptor)
          .toList();
      expect(customInterceptors.length, equals(2));
    });

    test('sessionCookieInterceptor ありでは3つのインターセプターが追加される', () {
      final sessionInterceptor = SessionCookieInterceptor();
      final client = ApiClient.create(
        baseUrl: 'https://api.example.com',
        csrfTokenProvider: () async => 'test-token',
        sessionCookieInterceptor: sessionInterceptor,
      );
      final customInterceptors = client.interceptors
          .where((i) =>
              i is InterceptorsWrapper ||
              i is CsrfTokenInterceptor ||
              i is SessionCookieInterceptor)
          .toList();
      expect(customInterceptors.length, equals(3));
    });
  });

  group('CsrfTokenInterceptor', () {
    test('トークンがリクエストヘッダーに追加される', () async {
      final client = ApiClient.create(
        baseUrl: 'https://api.example.com',
        csrfTokenProvider: () async => 'csrf-token-123',
      );

      // インターセプターの動作を検証するため、リクエストインターセプターを直接テスト
      final interceptor = client.interceptors
          .whereType<CsrfTokenInterceptor>()
          .first;
      expect(interceptor, isNotNull);

      final options = RequestOptions(path: '/test');
      await interceptor.onRequest(
        options,
        RequestInterceptorHandler(),
      );

      expect(options.headers['X-CSRF-Token'], equals('csrf-token-123'));
    });

    test('トークンが null の場合はヘッダーに追加されない', () async {
      final interceptor = CsrfTokenInterceptor(
        tokenProvider: () async => null,
      );

      final options = RequestOptions(path: '/test');

      await interceptor.onRequest(
        options,
        RequestInterceptorHandler(),
      );

      expect(options.headers.containsKey('X-CSRF-Token'), isFalse);
    });

    test('トークンが空文字の場合はヘッダーに追加されない', () async {
      final interceptor = CsrfTokenInterceptor(
        tokenProvider: () async => '',
      );

      final options = RequestOptions(path: '/test');

      await interceptor.onRequest(
        options,
        RequestInterceptorHandler(),
      );

      expect(options.headers.containsKey('X-CSRF-Token'), isFalse);
    });
  });

  group('SessionCookieInterceptor', () {
    test('初期状態では sessionId は null', () {
      final interceptor = SessionCookieInterceptor();
      expect(interceptor.sessionId, isNull);
    });

    test('sessionId が設定されている場合はリクエストに Cookie ヘッダーを付与する', () {
      final interceptor = SessionCookieInterceptor();
      interceptor.sessionId = 'test-session-123';

      final options = RequestOptions(path: '/test');
      interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('k1s0_session=test-session-123'),
      );
    });

    test('既存の Cookie ヘッダーがある場合はセミコロンで連結する', () {
      final interceptor = SessionCookieInterceptor();
      interceptor.sessionId = 'test-session-123';

      final options = RequestOptions(path: '/test');
      options.headers['Cookie'] = 'other=value';
      interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('other=value; k1s0_session=test-session-123'),
      );
    });

    test('Set-Cookie ヘッダーからセッション ID を抽出する', () {
      final interceptor = SessionCookieInterceptor();

      final headers = Headers();
      headers.set(
        'set-cookie',
        ['k1s0_session=extracted-session-456; Path=/; HttpOnly'],
      );

      final response = Response(
        requestOptions: RequestOptions(path: '/test'),
        headers: headers,
      );
      interceptor.onResponse(response, ResponseInterceptorHandler());

      expect(interceptor.sessionId, equals('extracted-session-456'));
    });

    test('関連しない Set-Cookie ヘッダーは無視する', () {
      final interceptor = SessionCookieInterceptor();

      final headers = Headers();
      headers.set('set-cookie', ['other_cookie=value; Path=/']);

      final response = Response(
        requestOptions: RequestOptions(path: '/test'),
        headers: headers,
      );
      interceptor.onResponse(response, ResponseInterceptorHandler());

      expect(interceptor.sessionId, isNull);
    });

    test('clearSession でセッション ID がクリアされる', () {
      final interceptor = SessionCookieInterceptor();
      interceptor.sessionId = 'some-session';
      interceptor.clearSession();

      expect(interceptor.sessionId, isNull);
    });

    test('カスタムクッキー名で動作する', () {
      final interceptor =
          SessionCookieInterceptor(cookieName: 'custom_session');
      interceptor.sessionId = 'custom-123';

      final options = RequestOptions(path: '/test');
      interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('custom_session=custom-123'),
      );
    });
  });
}
