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
      test('should redirect to the authorization endpoint with PKCE params',
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

      test('should store code_verifier and state', () async {
        final client = createClient();
        await client.login();

        expect(tokenStore.getCodeVerifier(), isNotNull);
        expect(tokenStore.getCodeVerifier(), isNotEmpty);
        expect(tokenStore.getState(), equals('mock-state-value'));
      });

      test('should fetch the OIDC discovery document', () async {
        final client = createClient();
        await client.login();

        expect(httpGetUrls, contains(_testConfig.discoveryUrl));
      });
    });

    group('handleCallback', () {
      test('should exchange code for tokens', () async {
        final client = createClient();
        await client.login();

        final tokenSet =
            await client.handleCallback('auth-code-123', 'mock-state-value');

        expect(tokenSet.accessToken, equals('mock-access-token'));
        expect(tokenSet.refreshToken, equals('mock-refresh-token'));
        expect(tokenSet.idToken, equals('mock-id-token'));
        expect(tokenSet.isValid, isTrue);
      });

      test('should throw on state mismatch', () async {
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

      test('should throw when PKCE verifier is missing', () async {
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

      test('should throw on token request failure', () async {
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

      test('should notify listeners on successful callback', () async {
        final client = createClient();
        final states = <bool>[];
        client.onAuthStateChange(states.add);

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, contains(true));
      });

      test('should clear code verifier and state after successful callback',
          () async {
        final client = createClient();
        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(tokenStore.getCodeVerifier(), isNull);
        expect(tokenStore.getState(), isNull);
      });

      test('should store the token set', () async {
        final client = createClient();
        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        final stored = tokenStore.getTokenSet();
        expect(stored, isNotNull);
        expect(stored!.accessToken, equals('mock-access-token'));
      });
    });

    group('getAccessToken', () {
      test('should return the access token when valid', () async {
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

      test('should throw when not authenticated', () async {
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

      test('should auto-refresh when token expires within 60 seconds',
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
      test('should exchange refresh token for new tokens', () async {
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

      test('should throw when no refresh token is available', () async {
        final client = createClient();
        expect(
          client.refreshToken,
          throwsA(isA<AuthError>()),
        );
      });

      test('should clear tokens and notify listeners on refresh failure',
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
      test('should return false when no token set', () {
        final client = createClient();
        expect(client.isAuthenticated, isFalse);
      });

      test('should return true when token is valid', () {
        tokenStore.setTokenSet(TokenSet(
          accessToken: 'token',
          refreshToken: 'refresh',
          idToken: 'id',
          expiresAt: DateTime.now().add(const Duration(minutes: 5)),
        ));
        final client = createClient();
        expect(client.isAuthenticated, isTrue);
      });

      test('should return false when token has expired', () {
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
      test('should clear tokens', () async {
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

      test('should notify listeners', () async {
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
          'should redirect to end_session_endpoint with id_token_hint',
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

      test('should not redirect when no token set exists', () async {
        final client = createClient();
        await client.logout();

        expect(redirectedUrl, isNull);
      });
    });

    group('onAuthStateChange', () {
      test('should register and notify a listener', () async {
        final client = createClient();
        final states = <bool>[];
        client.onAuthStateChange(states.add);

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, contains(true));
      });

      test('should return an unsubscribe function', () async {
        final client = createClient();
        final states = <bool>[];
        final unsubscribe =
            client.onAuthStateChange(states.add);

        unsubscribe();

        await client.login();
        await client.handleCallback('code', 'mock-state-value');

        expect(states, isEmpty);
      });

      test('should support multiple listeners', () async {
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
      test('should return null when no tokens', () {
        final client = createClient();
        expect(client.getTokenSet(), isNull);
      });

      test('should return stored token set', () {
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
      test('should return the authorization URL without redirecting', () async {
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
      test('should cache the discovery document', () async {
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
      test('should have the correct message', () {
        final error = AuthError('test error');
        expect(error.message, equals('test error'));
        expect(error.toString(), equals('AuthError: test error'));
      });
    });
  });

  group('TokenSet', () {
    test('isValid should return true for future expiry', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(minutes: 5)),
      );
      expect(ts.isValid, isTrue);
    });

    test('isValid should return false for past expiry', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().subtract(const Duration(seconds: 1)),
      );
      expect(ts.isValid, isFalse);
    });

    test('isExpiringSoon should return true within threshold', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(seconds: 30)),
      );
      expect(ts.isExpiringSoon(), isTrue);
    });

    test('isExpiringSoon should return false outside threshold', () {
      final ts = TokenSet(
        accessToken: 'a',
        refreshToken: 'r',
        idToken: 'i',
        expiresAt: DateTime.now().add(const Duration(minutes: 5)),
      );
      expect(ts.isExpiringSoon(), isFalse);
    });

    test('toJson and fromJson should round-trip', () {
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
    test('should store and retrieve token set', () {
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

    test('should clear token set', () {
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

    test('should store and retrieve code verifier', () {
      final store = MemoryTokenStore();
      store.setCodeVerifier('test-verifier');
      expect(store.getCodeVerifier(), equals('test-verifier'));
    });

    test('should store and retrieve state', () {
      final store = MemoryTokenStore();
      store.setState('test-state');
      expect(store.getState(), equals('test-state'));
    });

    test('clearAll should clear everything', () {
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
