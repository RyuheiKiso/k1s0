/// OAuth2 Authorization Code + PKCE クライアント
/// Keycloak 対応のクライアント側認証ライブラリ
library;

import 'dart:convert';

import 'package:http/http.dart' as http;

import 'pkce.dart';
import 'token_store.dart';
import 'types.dart';

/// HTTP クライアントの抽象化（テスト用に注入可能）
typedef HttpPost = Future<http.Response> Function(
  Uri url, {
  Map<String, String>? headers,
  Object? body,
});

typedef HttpGet = Future<http.Response> Function(
  Uri url, {
  Map<String, String>? headers,
});

/// AuthClient のオプション
class AuthClientOptions {
  final AuthConfig config;
  final TokenStore? tokenStore;
  final HttpPost? httpPost;
  final HttpGet? httpGet;
  final void Function(String url)? redirect;
  final String Function()? generateState;

  AuthClientOptions({
    required this.config,
    this.tokenStore,
    this.httpPost,
    this.httpGet,
    this.redirect,
    this.generateState,
  });
}

/// OAuth2 Authorization Code + PKCE クライアント
class AuthClient {
  final AuthConfig _config;
  final TokenStore _tokenStore;
  final HttpPost _httpPost;
  final HttpGet _httpGet;
  final void Function(String url) _redirect;
  final String Function() _generateState;
  final List<AuthStateCallback> _listeners = [];
  OIDCDiscovery? _discoveryCache;
  // L-20 監査対応: キャッシュ有効期限（1時間）- OIDC Discovery の更新に対応
  // IdP 側でエンドポイントが変更された際に古いエンドポイントを使い続けないようにする
  static const Duration _discoveryCacheTTL = Duration(hours: 1);
  DateTime? _discoveryCacheTime;

  AuthClient(AuthClientOptions options)
      : _config = options.config,
        _tokenStore = options.tokenStore ?? MemoryTokenStore(),
        _httpPost = options.httpPost ?? _defaultPost,
        _httpGet = options.httpGet ?? _defaultGet,
        _redirect = options.redirect ?? _defaultRedirect,
        _generateState = options.generateState ?? _defaultGenerateState;

  static Future<http.Response> _defaultPost(
    Uri url, {
    Map<String, String>? headers,
    Object? body,
  }) {
    return http.post(url, headers: headers, body: body);
  }

  static Future<http.Response> _defaultGet(
    Uri url, {
    Map<String, String>? headers,
  }) {
    return http.get(url, headers: headers);
  }

  // デフォルトのリダイレクトハンドラ。
  // redirect オプションが未設定の場合に呼ばれ、開発者に設定漏れを気づかせるために例外を投げる。
  // サイレントに失敗すると認証フローが無音で止まるため、必ず明示的なエラーとして通知する。
  static void _defaultRedirect(String url) {
    // デフォルトのリダイレクト実装が設定されていません。
    // AuthClientOptions.redirect を使用してリダイレクトハンドラを注入してください。
    throw UnimplementedError(
      'redirect handler not configured. '
      'Please provide a redirect handler via AuthClientOptions.redirect. '
      'Example: AuthClientOptions(redirect: (url) => launchUrl(Uri.parse(url)))',
    );
  }

  static String _defaultGenerateState() {
    return generateCodeVerifier();
  }

  /// OAuth2 Authorization Code + PKCE フローを開始する。
  /// 1. code_verifier を生成
  /// 2. code_challenge を計算
  /// 3. authorize URL を構築
  /// 4. リダイレクト
  Future<void> login() async {
    final codeVerifier = generateCodeVerifier();
    final codeChallenge = generateCodeChallenge(codeVerifier);
    final state = _generateState();

    _tokenStore.setCodeVerifier(codeVerifier);
    _tokenStore.setState(state);

    final discovery = await fetchDiscovery();
    final params = {
      'response_type': 'code',
      'client_id': _config.clientId,
      'redirect_uri': _config.redirectUri,
      'scope': _config.scopes.join(' '),
      'code_challenge': codeChallenge,
      'code_challenge_method': 'S256',
      'state': state,
    };

    final uri = Uri.parse(discovery.authorizationEndpoint)
        .replace(queryParameters: params);
    _redirect(uri.toString());
  }

  /// 認可コールバック URL を生成して返す（リダイレクトせずに URL だけ返すバージョン）。
  Future<String> getAuthorizationUrl() async {
    final codeVerifier = generateCodeVerifier();
    final codeChallenge = generateCodeChallenge(codeVerifier);
    final state = _generateState();

    _tokenStore.setCodeVerifier(codeVerifier);
    _tokenStore.setState(state);

    final discovery = await fetchDiscovery();
    final params = {
      'response_type': 'code',
      'client_id': _config.clientId,
      'redirect_uri': _config.redirectUri,
      'scope': _config.scopes.join(' '),
      'code_challenge': codeChallenge,
      'code_challenge_method': 'S256',
      'state': state,
    };

    return Uri.parse(discovery.authorizationEndpoint)
        .replace(queryParameters: params)
        .toString();
  }

  /// 認可コールバックを処理する。
  /// code + code_verifier で token endpoint に POST してトークンを取得・保存する。
  Future<TokenSet> handleCallback(String code, String state) async {
    final savedState = _tokenStore.getState();
    if (state != savedState) {
      throw AuthError('State mismatch');
    }

    final codeVerifier = _tokenStore.getCodeVerifier();
    if (codeVerifier == null) {
      throw AuthError('Missing PKCE verifier');
    }

    final discovery = await fetchDiscovery();
    final resp = await _httpPost(
      Uri.parse(discovery.tokenEndpoint),
      headers: {'Content-Type': 'application/x-www-form-urlencoded'},
      body: {
        'grant_type': 'authorization_code',
        'client_id': _config.clientId,
        'code': code,
        'redirect_uri': _config.redirectUri,
        'code_verifier': codeVerifier,
      },
    );

    if (resp.statusCode != 200) {
      throw AuthError('Token request failed: ${resp.statusCode}');
    }

    final data = jsonDecode(resp.body) as Map<String, dynamic>;
    // POLY-006 監査対応: refresh_token は一部 IdP では省略されるため null のまま保持する。
    // 空文字への変換を廃止し、TokenSet.refreshToken が String? (nullable) になったことで
    // 「リフレッシュトークンなし」を明示的に表現できるようになった。
    final tokenSet = TokenSet(
      accessToken: data['access_token'] as String,
      refreshToken: data['refresh_token'] as String?,
      idToken: data['id_token'] as String,
      expiresAt:
          DateTime.now().add(Duration(seconds: data['expires_in'] as int)),
    );

    await _tokenStore.setTokenSet(tokenSet); // POLY-007 監査対応: await で永続化完了を保証
    _tokenStore.clearCodeVerifier();
    _tokenStore.clearState();
    _notifyListeners(true);

    return tokenSet;
  }

  /// refresh_token で新しいアクセストークンを取得する。
  Future<void> refreshToken() async {
    final tokenSet = _tokenStore.getTokenSet();
    if (tokenSet == null) {
      throw AuthError('No token set');
    }
    // POLY-006 監査対応: refreshToken が null の場合はリフレッシュ不可能なため明示的にエラーをスローする。
    // 以前は空文字が渡されていたが、IdP が空文字の refresh_token を拒否するまでエラーが検出されなかった。
    final currentRefreshToken = tokenSet.refreshToken;
    if (currentRefreshToken == null || currentRefreshToken.isEmpty) {
      throw AuthError('No refresh token available');
    }

    final discovery = await fetchDiscovery();
    final resp = await _httpPost(
      Uri.parse(discovery.tokenEndpoint),
      headers: {'Content-Type': 'application/x-www-form-urlencoded'},
      body: {
        'grant_type': 'refresh_token',
        'client_id': _config.clientId,
        'refresh_token': currentRefreshToken,
      },
    );

    if (resp.statusCode != 200) {
      _tokenStore.clearTokenSet();
      _notifyListeners(false);
      throw AuthError('Token refresh failed: ${resp.statusCode}');
    }

    final data = jsonDecode(resp.body) as Map<String, dynamic>;
    final newTokenSet = TokenSet(
      accessToken: data['access_token'] as String,
      refreshToken: data['refresh_token'] as String?,
      idToken: data['id_token'] as String,
      expiresAt:
          DateTime.now().add(Duration(seconds: data['expires_in'] as int)),
    );

    _tokenStore.setTokenSet(newTokenSet);
  }

  /// 有効なアクセストークンを返す。
  /// 期限切れ 60 秒前なら自動リフレッシュする。
  Future<String> getAccessToken() async {
    final tokenSet = _tokenStore.getTokenSet();
    if (tokenSet == null) {
      throw AuthError('Not authenticated');
    }

    if (tokenSet.isExpiringSoon()) {
      await refreshToken();
      final refreshed = _tokenStore.getTokenSet();
      if (refreshed == null) {
        throw AuthError('Token refresh failed');
      }
      return refreshed.accessToken;
    }

    return tokenSet.accessToken;
  }

  /// 認証状態を返す。
  bool get isAuthenticated {
    final tokenSet = _tokenStore.getTokenSet();
    return tokenSet != null && tokenSet.isValid;
  }

  /// ログアウト処理。
  /// トークンを削除し、end_session エンドポイントにリダイレクトする。
  /// logoutUrl が設定されている場合はそれを優先し、
  /// 未設定の場合は OIDC Discovery の end_session_endpoint を使用する。
  Future<void> logout() async {
    final tokenSet = _tokenStore.getTokenSet();
    _tokenStore.clearAll();
    _notifyListeners(false);

    if (tokenSet != null) {
      try {
        // logoutUrl が設定されている場合はそれを優先し、
        // 未設定の場合は OIDC Discovery の end_session_endpoint を使用する。
        String endSessionUrl;
        if (_config.logoutUrl != null && _config.logoutUrl!.isNotEmpty) {
          endSessionUrl = _config.logoutUrl!;
        } else {
          final discovery = await fetchDiscovery();
          endSessionUrl = discovery.endSessionEndpoint;
        }
        final params = <String, String>{};
        params['id_token_hint'] = tokenSet.idToken;
        if (_config.postLogoutRedirectUri != null) {
          params['post_logout_redirect_uri'] = _config.postLogoutRedirectUri!;
          params['client_id'] = _config.clientId;
        }
        final uri = Uri.parse(endSessionUrl).replace(queryParameters: params);
        _redirect(uri.toString());
      } catch (_) {
        // Discovery 取得に失敗してもログアウト自体は成功とする
      }
    }
  }

  /// 現在のトークンセットを取得する。
  TokenSet? getTokenSet() => _tokenStore.getTokenSet();

  /// 認証状態の変更を監視するリスナーを登録する。
  /// 返り値の関数を呼ぶとリスナーが解除される。
  void Function() onAuthStateChange(AuthStateCallback callback) {
    _listeners.add(callback);
    return () => _listeners.remove(callback);
  }

  void _notifyListeners(bool authenticated) {
    for (final cb in _listeners) {
      cb(authenticated);
    }
  }

  /// OIDC Discovery ドキュメントを取得する（TTL 付きキャッシュあり）。
  /// L-20 監査対応: キャッシュが1時間を超えた場合は再取得して最新エンドポイントを使用する
  Future<OIDCDiscovery> fetchDiscovery() async {
    // キャッシュが有効期限内であればそのまま返す
    if (_discoveryCache != null &&
        _discoveryCacheTime != null &&
        DateTime.now().difference(_discoveryCacheTime!) < _discoveryCacheTTL) {
      return _discoveryCache!;
    }

    final resp = await _httpGet(Uri.parse(_config.discoveryUrl));
    if (resp.statusCode != 200) {
      throw AuthError('Discovery fetch failed: ${resp.statusCode}');
    }

    _discoveryCache = OIDCDiscovery.fromJson(
      jsonDecode(resp.body) as Map<String, dynamic>,
    );
    // キャッシュ取得時刻を記録する
    _discoveryCacheTime = DateTime.now();
    return _discoveryCache!;
  }
}
