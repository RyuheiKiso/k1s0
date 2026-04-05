import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:system_client/system_client.dart';

/// FlutterSecureStorage のモッククラス（SessionCookieInterceptor テスト用）
class MockFlutterSecureStorage extends Mock implements FlutterSecureStorage {}

/// セッション期限切れコールバックの呼び出し記録用ヘルパー
class SessionExpiredTracker {
  int callCount = 0;
  void call() => callCount++;
}

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
      // session_id は存在するが session_expiry はない（サーバー管理セッション）状態をモックする
      // session_expiry が null なら有効期限チェックをスキップしてセッションは有効とみなす
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'test-session-123');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => null);

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
      // session_id は存在するが session_expiry はない（サーバー管理セッション）状態をモックする
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'test-session-123');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => null);

      final options = RequestOptions(path: '/test');
      options.headers['Cookie'] = 'other=value';
      await interceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('other=value; k1s0_session=test-session-123'),
      );
    });

    test('Set-Cookie ヘッダーからセッション ID を抽出してセキュアストレージに保存する', () async {
      // write と delete が呼ばれることを検証するためのモック設定
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

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

      // セッション ID の書き込みが呼ばれたことを検証する
      verify(() => mockStorage.write(
            key: 'session_id',
            value: 'extracted-session-456',
          )).called(1);

      // 有効期限情報がないため session_expiry が削除されることを検証する
      verify(() => mockStorage.delete(key: 'session_expiry')).called(1);
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

    test('clearSession でセキュアストレージからセッション ID と有効期限が削除される', () async {
      // delete が呼ばれることを検証するためのモック設定
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      await interceptor.clearSession();

      // セッション ID と有効期限の両方が削除されることを検証する
      verify(() => mockStorage.delete(key: 'session_id')).called(1);
      verify(() => mockStorage.delete(key: 'session_expiry')).called(1);
    });

    test('カスタムクッキー名で動作する', () async {
      final customInterceptor = SessionCookieInterceptor(
        cookieName: 'custom_session',
        storage: mockStorage,
      );
      // session_id は存在するが session_expiry はない状態をモックする
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'custom-123');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => null);

      final options = RequestOptions(path: '/test');
      await customInterceptor.onRequest(options, RequestInterceptorHandler());

      expect(
        options.headers['Cookie'],
        equals('custom_session=custom-123'),
      );
    });

    // --- セッション有効期限管理テスト (FE-001) ---

    test('max-age 付き Set-Cookie でセッション ID と有効期限が保存される', () async {
      // write 呼び出しを記録するモック設定
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});

      final headers = Headers();
      headers.set(
        'set-cookie',
        ['k1s0_session=abc123; Path=/; HttpOnly; max-age=3600'],
      );

      final response = Response(
        requestOptions: RequestOptions(path: '/auth/exchange'),
        headers: headers,
      );
      await interceptor.onResponse(response, ResponseInterceptorHandler());

      // セッション ID の書き込みを検証する
      verify(() => mockStorage.write(
            key: 'session_id',
            value: 'abc123',
          )).called(1);

      // session_expiry も書き込まれることを検証する
      verify(() => mockStorage.write(
            key: 'session_expiry',
            value: any(named: 'value'),
          )).called(1);
    });

    test('expires 付き Set-Cookie でセッション ID と有効期限が保存される', () async {
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});

      final headers = Headers();
      headers.set(
        'set-cookie',
        ['k1s0_session=xyz789; Path=/; HttpOnly; Expires=Thu, 01 Jan 2099 00:00:00 GMT'],
      );

      final response = Response(
        requestOptions: RequestOptions(path: '/auth/exchange'),
        headers: headers,
      );
      await interceptor.onResponse(response, ResponseInterceptorHandler());

      // セッション ID の書き込みを検証する
      verify(() => mockStorage.write(
            key: 'session_id',
            value: 'xyz789',
          )).called(1);

      // session_expiry も書き込まれることを検証する
      verify(() => mockStorage.write(
            key: 'session_expiry',
            value: any(named: 'value'),
          )).called(1);
    });

    test('有効期限なし Set-Cookie では session_expiry が削除される', () async {
      when(() => mockStorage.write(
            key: any(named: 'key'),
            value: any(named: 'value'),
          )).thenAnswer((_) async {});
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      final headers = Headers();
      headers.set(
        'set-cookie',
        ['k1s0_session=noexpiry; Path=/; HttpOnly'],
      );

      final response = Response(
        requestOptions: RequestOptions(path: '/auth/exchange'),
        headers: headers,
      );
      await interceptor.onResponse(response, ResponseInterceptorHandler());

      // session_expiry が削除されることを検証する
      verify(() => mockStorage.delete(key: 'session_expiry')).called(1);
    });

    test('有効期限が未来の場合はセッション ID を Cookie に付与する', () async {
      // session_id は通常値、session_expiry は未来の時刻を返すモック設定
      final futureExpiry = DateTime.now().add(const Duration(hours: 1));
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'valid-session');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => futureExpiry.toIso8601String());

      final options = RequestOptions(path: '/test');
      await interceptor.onRequest(options, RequestInterceptorHandler());

      // 有効なセッションは Cookie ヘッダーに付与される
      expect(
        options.headers['Cookie'],
        equals('k1s0_session=valid-session'),
      );
    });

    test('有効期限切れの場合はセッションをクリアして Cookie を付与せず期限切れコールバックを呼ぶ',
        () async {
      final tracker = SessionExpiredTracker();
      final expiredInterceptor = SessionCookieInterceptor(
        storage: mockStorage,
        onSessionExpired: tracker.call,
      );

      // session_id は存在するが、有効期限は過去の時刻を返すモック設定
      final pastExpiry = DateTime.now().subtract(const Duration(hours: 1));
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'expired-session');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => pastExpiry.toIso8601String());
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      final options = RequestOptions(path: '/test');
      await expiredInterceptor.onRequest(options, RequestInterceptorHandler());

      // 期限切れセッションは Cookie ヘッダーに付与されない
      expect(options.headers.containsKey('Cookie'), isFalse);

      // セッションがクリアされることを検証する
      verify(() => mockStorage.delete(key: 'session_id')).called(1);
      verify(() => mockStorage.delete(key: 'session_expiry')).called(1);

      // 期限切れコールバックが呼ばれることを検証する
      expect(tracker.callCount, equals(1));
    });

    test('session_expiry が null の場合は期限切れとみなさずセッション ID を付与する', () async {
      // session_id は存在するが、session_expiry は保存されていない状態のモック設定
      when(() => mockStorage.read(key: 'session_id'))
          .thenAnswer((_) async => 'server-managed-session');
      when(() => mockStorage.read(key: 'session_expiry'))
          .thenAnswer((_) async => null);

      final options = RequestOptions(path: '/test');
      await interceptor.onRequest(options, RequestInterceptorHandler());

      // 期限情報なしはサーバー管理とみなし Cookie を付与する
      expect(
        options.headers['Cookie'],
        equals('k1s0_session=server-managed-session'),
      );
    });

    test('clearSession でセッション ID と有効期限が両方削除される', () async {
      when(() => mockStorage.delete(key: any(named: 'key')))
          .thenAnswer((_) async {});

      await interceptor.clearSession();

      // session_id と session_expiry の両方が削除されることを検証する
      verify(() => mockStorage.delete(key: 'session_id')).called(1);
      verify(() => mockStorage.delete(key: 'session_expiry')).called(1);
    });
  });
}
