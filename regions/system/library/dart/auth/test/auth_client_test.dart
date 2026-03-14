import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:k1s0_auth/auth.dart';
import 'package:test/test.dart';

final _testDiscovery = {
  'authorization_endpoint': 'https://auth.example.com/authorize',
  'token_endpoint': 'https://auth.example.com/token',
  'end_session_endpoint': 'https://auth.example.com/logout',
  'jwks_uri': 'https://auth.example.com/certs',
  'issuer': 'https://auth.example.com/realms/k1s0',
};

final _testConfig = AuthConfig(
  discoveryUrl: 'https://auth.example.com/.well-known/openid-configuration',
  clientId: 'test-client',
  redirectUri: 'https://app.example.com/callback',
  scopes: ['openid', 'profile', 'email'],
  postLogoutRedirectUri: 'https://app.example.com/',
);

Map<String, dynamic> _defaultTokenResponse() => {
      'access_token': 'mock-access-token',
      'refresh_token': 'mock-refresh-token',
      'id_token': 'mock-id-token',
      'expires_in': 900,
      'token_type': 'Bearer',
    };

void main() {
  late MemoryTokenStore tokenStore;
  late String? redirectedUrl;
  late List<String> httpGetUrls;
  late List<String> httpPostUrls;
  late Map<String, dynamic> Function() tokenResponseFactory;

  Future<http.Response> mockGet(Uri url, {Map<String, String>? headers}) async {
    httpGetUrls.add(url.toString());
    if (url.toString() == _testConfig.discoveryUrl) {
      return http.Response(jsonEncode(_testDiscovery), 200);
    }
    return http.Response('Not Found', 404);
  }

  Future<http.Response> mockPost(Uri url,
      {Map<String, String>? headers, Object? body}) async {
    httpPostUrls.add(url.toString());
    if (url.toString() == _testDiscovery['token_endpoint']) {
      return http.Response(jsonEncode(tokenResponseFactory()), 200);
    }
    return http.Response('Not Found', 404);
  }

  AuthClient createClient({
    Future<http.Response> Function(Uri, {Map<String, String>? headers})?
        httpGet,
    Future<http.Response> Function(Uri,
            {Map<String, String>? headers, Object? body})?
        httpPost,
  }) {
    return AuthClient(AuthClientOptions(
      config: _testConfig,
      tokenStore: tokenStore,
      httpGet: httpGet ?? mockGet,
      httpPost: httpPost ?? mockPost,
      redirect: (url) => redirectedUrl = url,
      generateState: () => 'mock-state-value',
    ));
  }

  setUp(() {
    tokenStore = MemoryTokenStore();
    redirectedUrl = null;
    httpGetUrls = [];
    httpPostUrls = [];
    tokenResponseFactory = _defaultTokenResponse;
  });

  group('AuthClient', () {
    group('login', () {
      test('PKCE パラメータ付きで認可エンドポイントにリダイレクトすること',
          () async {
        final client = createClient();
        await client.login();

        expect(redirectedUrl, isNotNull);
        final uri = Uri.parse(redirectedUrl!);
        expect(
          '${uri.scheme}://${uri.host}${uri.path}',
          equals(_testDiscovery['authorization_endpoint']),
        );
        expect(uri.queryParameters['response_type'], equals('code'));
        expect(uri.queryParameters['client_id'], equals('test-client'));
        expect(uri.queryParameters['redirect_uri'],
            equals('https://app.example.com/callback'));
        expect(uri.queryParameters['scope'], equals('openid profile email'));
        expect(uri.queryParameters['code_challenge_method'], equals('S256'));
        expect(uri.queryParameters['code_challenge'], isNotEmpty);
        expect(uri.queryParameters['state'], equals('mock-state-value'));
      });

      test('code_verifier と state を保存すること', () async {
        final client = createClient();
        await client.login();

        expect(tokenStore.getCodeVerifier(), isNotNull);
        expect(tokenStore.getCodeVerifier(), isNotEmpty);
        expect(tokenStore.getState(), equals('mock-state-value'));
      });

      test('OIDC ディスカバリドキュメントを取得すること', () async {
        final client = createClient();
        await client.login();

        expect(httpGetUrls, contains(_testConfig.discoveryUrl));
      });
    });

    group('handleCallback', () {
      test('コードをトークンに交換すること', () async {
        final client = createClient();
        await client.login();

        final tokenSet =
            await client.handleCallback('auth-code-123', 'mock-state-value');

        expect(tokenSet.accessToken, equals('mock-access-token'));
        expect(tokenSet.refreshToken, equals('mock-refresh-token'));
        expect(tokenSet.idToken, equals('mock-id-token'));
        expect(tokenSet.isValid, isTrue);
      });

      test('state が一致しない場合に例外をスローすること', () async {
        final client = createClient();
        await client.login();

        expect(
          () => client.handleCallback('code', 'wrong-state'),
          throwsA(isA<AuthError>().having(
            (e) => e.message,
            'message',
            'State mismatch',
          )),
        );
      });

      test('PKCE ベリファイアが存在しない場合に例外をスローすること', () async {
        final client = createClient();
        tokenStore.setState('mock-state-value');
        // Don't set code verifier

        expect(
          () => client.handleCallback('code', 'mock-state-value'),
          throwsA(isA<AuthError>().having(
            (e) => e.message,
            'message',
            'Missing PKCE verifier',
          )),
        );
      });

      test('トークンリクエストが失敗した場合に例外をスローすること', () async {
        Future<http.Response> failPost(Uri url,
            {Map<String, String>? headers, Object? body}) async {
          httpPostUrls.add(url.toString());
          return http.Response('Unauthorized', 401);
        }

        final client = createClient(httpPost: failPost);
        await client.login();

        expect(
          () => client.handleCallback('code', 'mock-state-value'),
          throwsA(isA<AuthError>().having(
            (e) => e.message,
            'message',
            contains('Token request failed'),
          )),
        );
      });

      test('コールバック成功時にリスナーに通知すること', () async {
        final client = createClient();
        final states = <bool>[];
        client.onAuthStateChange(states.add);

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, contains(true));
      });

      test('コールバック成功後に code verifier と state をクリアすること',
          () async {
        final client = createClient();
        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(tokenStore.getCodeVerifier(), isNull);
        expect(tokenStore.getState(), isNull);
      });

      test('トークンセットを保存すること', () async {
        final client = createClient();
        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        final stored = tokenStore.getTokenSet();
        expect(stored, isNotNull);
        expect(stored!.accessToken, equals('mock-access-token'));
      });
    });

    group('getAccessToken', () {
      test('有効なアクセストークンを返すこと', () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'valid-token',
          refreshToken: 'refresh-token',
          idToken: 'id-token',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));

        final client = createClient();
        final token = await client.getAccessToken();
        expect(token, equals('valid-token'));
      });

      test('未認証の場合に例外をスローすること', () async {
        final client = createClient();
        expect(
          client.getAccessToken,
          throwsA(isA<AuthError>().having(
            (e) => e.message,
            'message',
            'Not authenticated',
          )),
        );
      });

      test('トークンの有効期限が 60 秒以内の場合に自動リフレッシュすること',
          () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'expiring-token',
          refreshToken: 'refresh-token',
          idToken: 'id-token',
          expiresAt: DateTime.now().add(const Duration(seconds: 30)),
        ));

        tokenResponseFactory = () => {
              'access_token': 'refreshed-token',
              'refresh_token': 'new-refresh-token',
              'id_token': 'new-id-token',
              'expires_in': 900,
              'token_type': 'Bearer',
            };

        final client = createClient();
        final token = await client.getAccessToken();
        expect(token, equals('refreshed-token'));
      });
    });

    group('refreshToken', () {
      test('リフレッシュトークンを新しいトークンと交換すること', () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'old-access',
          refreshToken: 'old-refresh',
          idToken: 'old-id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));

        final client = createClient();
        await client.refreshToken();

        final newTokenSet = tokenStore.getTokenSet();
        expect(newTokenSet!.accessToken, equals('mock-access-token'));
        expect(newTokenSet.refreshToken, equals('mock-refresh-token'));
      });

      test('リフレッシュトークンが存在しない場合に例外をスローすること', () async {
        final client = createClient();
        expect(
          client.refreshToken,
          throwsA(isA<AuthError>()),
        );
      });

      test('リフレッシュ失敗時にトークンをクリアしてリスナーに通知すること',
          () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'access',
          refreshToken: 'expired-refresh',
          idToken: 'id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));

        Future<http.Response> failPost(Uri url,
            {Map<String, String>? headers, Object? body}) async {
          httpPostUrls.add(url.toString());
          return http.Response('Forbidden', 403);
        }

        final client = createClient(httpPost: failPost);
        final states = <bool>[];
        client.onAuthStateChange(states.add);

        expect(
          client.refreshToken,
          throwsA(isA<AuthError>().having(
            (e) => e.message,
            'message',
            contains('Token refresh failed'),
          )),
        );
      });
    });

    group('isAuthenticated', () {
      test('トークンセットが存在しない場合に false を返すこと', () {
        final client = createClient();
        expect(client.isAuthenticated, isFalse);
      });

      test('トークンが有効な場合に true を返すこと', () {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        final client = createClient();
        expect(client.isAuthenticated, isTrue);
      });

      test('トークンが期限切れの場合に false を返すこと', () {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'id',
          expiresAt: DateTime.now().subtract(const Duration(seconds: 1)),
        ));
        final client = createClient();
        expect(client.isAuthenticated, isFalse);
      });
    });

    group('logout', () {
      test('トークンをクリアすること', () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        final client = createClient();
        await client.logout();

        expect(tokenStore.getTokenSet(), isNull);
      });

      test('リスナーに通知すること', () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        final client = createClient();
        final states = <bool>[];
        client.onAuthStateChange(states.add);
        await client.logout();

        expect(states, contains(false));
      });

      test(
          'id_token_hint 付きで end_session_endpoint にリダイレクトすること',
          () async {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'my-id-token',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        final client = createClient();
        await client.logout();

        expect(redirectedUrl, isNotNull);
        final uri = Uri.parse(redirectedUrl!);
        expect(
          '${uri.scheme}://${uri.host}${uri.path}',
          equals(_testDiscovery['end_session_endpoint']),
        );
        expect(uri.queryParameters['id_token_hint'], equals('my-id-token'));
        expect(
          uri.queryParameters['post_logout_redirect_uri'],
          equals(_testConfig.postLogoutRedirectUri),
        );
        expect(
          uri.queryParameters['client_id'],
          equals(_testConfig.clientId),
        );
      });

      test('トークンセットが存在しない場合はリダイレクトしないこと', () async {
        final client = createClient();
        await client.logout();

        expect(redirectedUrl, isNull);
      });
    });

    group('onAuthStateChange', () {
      test('リスナーを登録して通知すること', () async {
        final client = createClient();
        final states = <bool>[];
        client.onAuthStateChange(states.add);

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, contains(true));
      });

      test('購読解除関数を返すこと', () async {
        final client = createClient();
        final states = <bool>[];
        final unsubscribe =
            client.onAuthStateChange(states.add);

        unsubscribe();

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, isEmpty);
      });

      test('複数のリスナーをサポートすること', () async {
        final client = createClient();
        final states1 = <bool>[];
        final states2 = <bool>[];
        client.onAuthStateChange(states1.add);
        client.onAuthStateChange(states2.add);

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states1, contains(true));
        expect(states2, contains(true));
      });
    });

    group('getTokenSet', () {
      test('トークンが存在しない場合に null を返すこと', () {
        final client = createClient();
        expect(client.getTokenSet(), isNull);
      });

      test('保存されたトークンセットを返すこと', () {
        final ts = TokenSet(
          accessToken: 'a',
          refreshToken: 'r',
          idToken: 'i',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        );
        tokenStore.setTokenSet(ts);
        final client = createClient();
        final result = client.getTokenSet();
        expect(result, isNotNull);
        expect(result!.accessToken, equals('a'));
      });
    });

    group('getAuthorizationUrl', () {
      test('リダイレクトせずに認可 URL を返すこと', () async {
        final client = createClient();
        final url = await client.getAuthorizationUrl();

        expect(redirectedUrl, isNull); // should NOT redirect
        final uri = Uri.parse(url);
        expect(
          '${uri.scheme}://${uri.host}${uri.path}',
          equals(_testDiscovery['authorization_endpoint']),
        );
        expect(uri.queryParameters['response_type'], equals('code'));
        expect(uri.queryParameters['code_challenge_method'], equals('S256'));
      });
    });

    group('discovery caching', () {
      test('ディスカバリドキュメントをキャッシュすること', () async {
        final client = createClient();
        await client.login();
        redirectedUrl = null;
        await client.login();

        final discoveryCalls = httpGetUrls
            .where((url) => url == _testConfig.discoveryUrl)
            .length;
        expect(discoveryCalls, equals(1));
      });
    });

    group('AuthError', () {
      test('正しいメッセージを持つこと', () {
        final error = AuthError('test error');
        expect(error.message, equals('test error'));
        expect(error.toString(), equals('AuthError: test error'));
      });
    });
  });

  group('TokenSet', () {
    test('有効期限が未来の場合に isValid が true を返すこと', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(minutes: 5)),
      );
      expect(ts.isValid, isTrue);
    });

    test('有効期限が過去の場合に isValid が false を返すこと', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().subtract(const Duration(seconds: 1)),
      );
      expect(ts.isValid, isFalse);
    });

    test('閾値内の場合に isExpiringSoon が true を返すこと', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(seconds: 30)),
      );
      expect(ts.isExpiringSoon(), isTrue);
    });

    test('閾値外の場合に isExpiringSoon が false を返すこと', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(minutes: 5)),
      );
      expect(ts.isExpiringSoon(), isFalse);
    });

    test('toJson と fromJson でラウンドトリップできること', () {
      final ts = TokenSet(
        accessToken: 'access',
        refreshToken: 'refresh',
        idToken: 'id',
        expiresAt: DateTime.utc(2026, 1, 1),
      );
      final json = ts.toJson();
      final restored = TokenSet.fromJson(json);
      expect(restored.accessToken, equals('access'));
      expect(restored.refreshToken, equals('refresh'));
      expect(restored.idToken, equals('id'));
      expect(restored.expiresAt, equals(DateTime.utc(2026, 1, 1)));
    });
  });

  group('MemoryTokenStore', () {
    test('トークンセットを保存・取得できること', () {
      final store = MemoryTokenStore();
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(minutes: 5)),
      );
      store.setTokenSet(ts);
      expect(store.getTokenSet()?.accessToken, equals('a'));
    });

    test('トークンセットをクリアできること', () {
      final store = MemoryTokenStore();
      store.setTokenSet(TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now(),
      ));
      store.clearTokenSet();
      expect(store.getTokenSet(), isNull);
    });

    test('code verifier を保存・取得できること', () {
      final store = MemoryTokenStore();
      store.setCodeVerifier('test-verifier');
      expect(store.getCodeVerifier(), equals('test-verifier'));
    });

    test('state を保存・取得できること', () {
      final store = MemoryTokenStore();
      store.setState('test-state');
      expect(store.getState(), equals('test-state'));
    });

    test('clearAll で全データをクリアすること', () {
      final store = MemoryTokenStore();
      store.setTokenSet(TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now(),
      ));
      store.setCodeVerifier('verifier');
      store.setState('state');
      store.clearAll();
      expect(store.getTokenSet(), isNull);
      expect(store.getCodeVerifier(), isNull);
      expect(store.getState(), isNull);
    });
  });
}
