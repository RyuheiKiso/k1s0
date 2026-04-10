/// OAuth2 Authorization Code + PKCE クライアント
/// Keycloak 対応のクライアント側認証ライブラリ
library;

import 'dart:convert';
// CRIT-001 対応: JWK の base64url バイト列を BigInt に変換するために dart:typed_data を使用する。
// _base64UrlToBigInt 関数内で Uint8List を扱うために必要。
import 'dart:typed_data';

import 'package:dart_jsonwebtoken/dart_jsonwebtoken.dart';
import 'package:http/http.dart' as http;
// CRIT-001 対応: JWK の n/e コンポーネントから RSA 公開鍵を構築するために pointycastle を使用する。
// dart_jsonwebtoken ^2.14.0 では RSAPublicKey.fromComponents(n, e) が廃止され、
// RSAPublicKey.raw(pc.RSAPublicKey) で pointycastle オブジェクトを直接受け取る形式に変更された。
import 'package:pointycastle/asymmetric/api.dart' as pc;

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

    // H-008 監査対応: await を追加して code_verifier の永続化完了を保証する
    await _tokenStore.setCodeVerifier(codeVerifier);
    // MED-016 監査対応: setState も await して永続化完了を保証する
    await _tokenStore.setState(state);

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

    // H-008 監査対応: await を追加して code_verifier の永続化完了を保証する
    await _tokenStore.setCodeVerifier(codeVerifier);
    // MED-016 監査対応: setState も await して永続化完了を保証する
    await _tokenStore.setState(state);

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

    // CRITICAL-FE-001 対応: トークン格納前に id_token の RS256 署名を JWKS 公開鍵で検証する。
    // 改ざんされた id_token や中間者攻撃を早期に検知するため、保存前に必ず検証する。
    // 参考: ADR-0100、OIDC Core §3.1.3.7
    await _verifyIdToken(tokenSet.idToken, discovery);
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

    // H-007 監査対応: await を追加してトークン永続化の完了を保証する。
    // await なしだと fire-and-forget となり、アプリ終了時にトークンが保存されないリスクがある。
    await _tokenStore.setTokenSet(newTokenSet);
    _notifyListeners(true);
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

  /// JWKS エンドポイントから公開鍵を取得して id_token の RS256 署名を検証する。
  /// CRITICAL-FE-001 対応: Keycloak から受け取ったトークンの改ざん検知を実装する。
  /// issuer と audience を検証することで OIDC Core 仕様 §3.1.3.7 に準拠する。
  /// kid（Key ID）でマッチする JWK を検索し、RS256 公開鍵で署名検証を行う。
  Future<void> _verifyIdToken(String idToken, OIDCDiscovery discovery) async {
    // JWKS エンドポイントから公開鍵セットを取得する
    final jwksResp = await _httpGet(Uri.parse(discovery.jwksUri));
    if (jwksResp.statusCode != 200) {
      throw AuthError('JWKS fetch failed: ${jwksResp.statusCode}');
    }
    final jwksData = jsonDecode(jwksResp.body) as Map<String, dynamic>;
    final keys = jwksData['keys'] as List<dynamic>;

    // id_token のヘッダーから kid を取得する（検証前にヘッダーのみデコードする）
    final parts = idToken.split('.');
    if (parts.length != 3) {
      throw AuthError('Invalid id_token format');
    }
    // Base64URL パディングを補完してヘッダーをデコードする
    final headerPadded = base64Url.normalize(parts[0]);
    final headerJson = jsonDecode(utf8.decode(base64Url.decode(headerPadded))) as Map<String, dynamic>;
    final tokenKid = headerJson['kid'] as String?;
    final tokenAlg = headerJson['alg'] as String? ?? 'RS256';

    // アルゴリズムが RS256 であることを確認する（それ以外のアルゴリズムは拒否する）
    if (tokenAlg != 'RS256') {
      throw AuthError('Unsupported algorithm: $tokenAlg. Only RS256 is accepted.');
    }

    // kid でマッチする JWK を検索する（kid がない場合は最初の RSA 鍵を使用する）
    Map<String, dynamic>? matchedKey;
    for (final key in keys) {
      final k = key as Map<String, dynamic>;
      if (k['kty'] == 'RSA') {
        if (tokenKid == null || k['kid'] == tokenKid) {
          matchedKey = k;
          break;
        }
      }
    }

    if (matchedKey == null) {
      throw AuthError('No matching JWK found for kid: $tokenKid');
    }

    // JWK の n/e コンポーネントから RSA 公開鍵を構築して署名を検証する
    final n = matchedKey['n'] as String;
    final e = matchedKey['e'] as String;
    try {
      // CRIT-001 対応: RSAPublicKey.fromComponents(n, e) は dart_jsonwebtoken ^2.14.0 に存在しない。
      // JWK の base64url 文字列を BigInt に変換し、pointycastle の RSAPublicKey を経由して
      // dart_jsonwebtoken の RSAPublicKey.raw() で公開鍵オブジェクトを構築する。
      final modulus = _base64UrlToBigInt(n);
      final exponent = _base64UrlToBigInt(e);
      final publicKey = RSAPublicKey.raw(pc.RSAPublicKey(modulus, exponent));
      final jwt = JWT.verify(
        idToken,
        publicKey,
        // 指数バックオフを考慮して30秒の時刻ずれを許容する（サーバー時刻差吸収）
        checkHeaderType: false,
      );
      // iss クレームを検証する（発行者の偽装を防ぐ）
      final payload = jwt.payload as Map<String, dynamic>;
      final iss = payload['iss'] as String?;
      if (iss == null || iss != discovery.issuer) {
        throw AuthError('Invalid issuer: expected ${discovery.issuer}, got $iss');
      }
      // aud クレームを検証する（このクライアント宛てのトークンであることを確認する）
      final aud = payload['aud'];
      final audList = switch (aud) {
        final List<dynamic> list => list.cast<String>(),
        final String s => [s],
        _ => <String>[],
      };
      if (!audList.contains(_config.clientId)) {
        throw AuthError('Invalid audience: clientId ${_config.clientId} not in $audList');
      }
      // exp クレームを検証する（期限切れトークンを拒否する）
      final exp = payload['exp'] as int?;
      if (exp == null || DateTime.fromMillisecondsSinceEpoch(exp * 1000).isBefore(DateTime.now())) {
        throw AuthError('Token has expired');
      }
    } on JWTExpiredException {
      throw AuthError('Token has expired');
    } on JWTException catch (e) {
      throw AuthError('JWT verification failed: ${e.message}');
    }
  }

  /// OIDC Discovery ドキュメントを取得する（TTL 付きキャッシュ + 指数バックオフリトライあり）。
  /// L-20 監査対応: キャッシュが1時間を超えた場合は再取得して最新エンドポイントを使用する
  /// M-016-dart 監査対応: ネットワークエラーおよび 5xx レスポンス時に指数バックオフで最大3回リトライする
  ///   - 1回目リトライ: 500ms 待機後
  ///   - 2回目リトライ: 1000ms 待機後
  Future<OIDCDiscovery> fetchDiscovery() async {
    // キャッシュが有効期限内であればそのまま返す
    if (_discoveryCache != null &&
        _discoveryCacheTime != null &&
        DateTime.now().difference(_discoveryCacheTime!) < _discoveryCacheTTL) {
      return _discoveryCache!;
    }

    // 指数バックオフリトライ待機時間（ミリ秒）
    // 1回目: 500ms、2回目: 1000ms
    const retryDelays = [500, 1000];
    // 最大試行回数（初回 + リトライ2回 = 計3回）
    const maxAttempts = 3;

    http.Response? lastResponse;
    for (var attempt = 0; attempt < maxAttempts; attempt++) {
      if (attempt > 0) {
        // リトライ前に指数バックオフで待機する
        await Future<void>.delayed(
          Duration(milliseconds: retryDelays[attempt - 1]),
        );
      }

      try {
        final resp = await _httpGet(Uri.parse(_config.discoveryUrl));
        // 成功または 4xx（クライアントエラー）はリトライしない
        if (resp.statusCode == 200) {
          _discoveryCache = OIDCDiscovery.fromJson(
            jsonDecode(resp.body) as Map<String, dynamic>,
          );
          // キャッシュ取得時刻を記録する
          _discoveryCacheTime = DateTime.now();
          return _discoveryCache!;
        }
        // 5xx（サーバーエラー）はリトライ対象
        if (resp.statusCode >= 500) {
          lastResponse = resp;
          continue;
        }
        // 4xx はリトライしない
        throw AuthError('Discovery fetch failed: ${resp.statusCode}');
      } on AuthError {
        rethrow;
      } catch (e) {
        // ネットワークエラーはリトライ対象。最後の試行であれば上位に伝播する
        if (attempt == maxAttempts - 1) {
          rethrow;
        }
      }
    }

    // すべてのリトライが 5xx で失敗した場合
    throw AuthError(
      'Discovery fetch failed after $maxAttempts attempts: ${lastResponse?.statusCode}',
    );
  }
}

/// Base64URLエンコードされたバイト列をBigIntに変換する（JWK RSA鍵コンポーネント用）。
/// JWK の n（modulus）と e（public exponent）の変換に使用する。
/// JWK 仕様（RFC 7518 §6.3）では n/e はパディングなし Base64URL でエンコードされるため、
/// デコード前にパディングを補完する必要がある。
BigInt _base64UrlToBigInt(String base64UrlStr) {
  // Base64URL パディングを補完する（JWK はパディングなしが標準）
  final padded = base64UrlStr.padRight(
    base64UrlStr.length + (4 - base64UrlStr.length % 4) % 4,
    '=',
  );
  // Base64URL 文字列をバイト列にデコードする
  final Uint8List bytes = base64Url.decode(padded);
  // ビッグエンディアンバイト列を BigInt に変換する（JWK は常にビッグエンディアン）
  return bytes.fold(
    BigInt.zero,
    (acc, byte) => (acc << 8) | BigInt.from(byte),
  );
}
