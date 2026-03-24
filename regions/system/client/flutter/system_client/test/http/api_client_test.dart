import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

/// FlutterSecureStorage のモッククラス（SessionCookieInterceptor テスト用）
class MockFlutterSecureStorage extends Mock implements FlutterSecureStorage {}

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
    late MockFlutterSecureStorage mockStorage;
    late SessionCookieInterceptor interceptor;

    setUp(() {
      mockStorage = MockFlutterSecureStorage();
      // FlutterSecureStorage を注入してテスト可能にする
      interceptor = SessionCookieInterceptor(storage: mockStorage);
    });

    test('セッション ID が存在する場合はリクエストに Cookie ヘッダーを付与する', () async {
      // セキュアストレージから 'test-session-123' を返すようにモックする
      when(() => mockStorage.read(key: any(named: 'key')))
          .thenAnswer((_) async => 'test-session-123');

      final options = RequestOptions(path: '/test');
      await interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('k1s0_session=test-session-123'),
      );
    });

    test('セッション ID が null の場合は Cookie ヘッダーを付与しない', () async {
      // ストレージにセッション ID がない状態をモックする
      when(() => mockStorage.read(key: any(named: 'key')))
          .thenAnswer((_) async => null);

      final options = RequestOptions(path: '/test');
      await interceptor.onRequest(options, RequestInterceptorHandler());

      expect(options.headers.containsKey('Cookie'), isFalse);
    });

    test('既存の Cookie ヘッダーがある場合はセミコロンで連結する', () async {
      when(() => mockStorage.read(key: any(named: 'key')))
          .thenAnswer((_) async => 'test-session-123');

      final options = RequestOptions(path: '/test');
      options.headers['Cookie'] = 'other=value';
      await interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('other=value; k1s0_session=test-session-123'),
      );
    });

    test('Set-Cookie ヘッダーからセッション ID を抽出してセキュアストレージに保存する', () async {
      // write が呼ばれることを検証するためのモック設定
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});

      final headers = Headers();
      headers.set(
        'set-cookie',
        ['k1s0_session=extracted-session-456; Path=/; HttpOnly'],
      );

      final response = Response(
        requestOptions: RequestOptions(path: '/test'),
        headers: headers,
      );
      await interceptor.onResponse(response, ResponseInterceptorHandler());

      // セキュアストレージへの書き込みが呼ばれたことを検証する
      verify(() => mockStorage.write(
            key: 'session_id',
            value: 'extracted-session-456',
          )).called(1);
    });

    test('関連しない Set-Cookie ヘッダーはセキュアストレージに書き込まない', () async {
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});

      final headers = Headers();
      headers.set('set-cookie', ['other_cookie=value; Path=/']);

      final response = Response(
        requestOptions: RequestOptions(path: '/test'),
        headers: headers,
      );
      await interceptor.onResponse(response, ResponseInterceptorHandler());

      // 関係ないクッキーなので書き込みが呼ばれないことを検証する
      verifyNever(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          ));
    });

    test('clearSession でセキュアストレージからセッション ID が削除される', () async {
      // delete が呼ばれることを検証するためのモック設定
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      await interceptor.clearSession();

      verify(() => mockStorage.delete(key: 'session_id')).called(1);
    });

    test('カスタムクッキー名で動作する', () async {
      final customInterceptor = SessionCookieInterceptor(
        cookieName: 'custom_session',
        storage: mockStorage,
      );
      when(() => mockStorage.read(key: any(named: 'key')))
          .thenAnswer((_) async => 'custom-123');

      final options = RequestOptions(path: '/test');
      await customInterceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('custom_session=custom-123'),
      );
    });
  });
}
