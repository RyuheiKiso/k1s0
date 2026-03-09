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
      bool nextCalled = false;

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
}
