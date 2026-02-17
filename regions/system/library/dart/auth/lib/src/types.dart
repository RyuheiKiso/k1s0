/// OAuth2 PKCE クライアント用の型定義
/// 認証認可設計.md の JWT Claims 構造に準拠
library;

/// AuthClient の設定
class AuthConfig {
  /// OIDC Discovery URL
  final String discoveryUrl;

  /// OAuth2 クライアント ID
  final String clientId;

  /// リダイレクト URI
  final String redirectUri;

  /// 要求するスコープ
  final List<String> scopes;

  /// Keycloak の logout endpoint URL（オプション）
  final String? logoutUrl;

  /// post_logout_redirect_uri（オプション）
  final String? postLogoutRedirectUri;

  AuthConfig({
    required this.discoveryUrl,
    required this.clientId,
    required this.redirectUri,
    required this.scopes,
    this.logoutUrl,
    this.postLogoutRedirectUri,
  });
}

/// 保存用のトークンセット
class TokenSet {
  final String accessToken;
  final String refreshToken;
  final String idToken;
  final DateTime expiresAt;

  TokenSet({
    required this.accessToken,
    required this.refreshToken,
    required this.idToken,
    required this.expiresAt,
  });

  /// トークンが有効かどうか
  bool get isValid => DateTime.now().isBefore(expiresAt);

  /// トークンが指定秒数以内に期限切れになるかどうか
  bool isExpiringSoon({Duration threshold = const Duration(seconds: 60)}) {
    return DateTime.now().isAfter(expiresAt.subtract(threshold));
  }

  Map<String, dynamic> toJson() => {
        'accessToken': accessToken,
        'refreshToken': refreshToken,
        'idToken': idToken,
        'expiresAt': expiresAt.toIso8601String(),
      };

  factory TokenSet.fromJson(Map<String, dynamic> json) => TokenSet(
        accessToken: json['accessToken'] as String,
        refreshToken: json['refreshToken'] as String,
        idToken: json['idToken'] as String,
        expiresAt: DateTime.parse(json['expiresAt'] as String),
      );
}

/// JWT Claims 構造（認証認可設計.md 準拠）
class Claims {
  final String sub;
  final String iss;
  final String aud;
  final int exp;
  final int iat;
  final String jti;
  final String typ;
  final String azp;
  final String scope;
  final String preferredUsername;
  final String email;
  final RealmAccess realmAccess;
  final Map<String, ResourceRoles> resourceAccess;
  final List<String> tierAccess;

  Claims({
    required this.sub,
    required this.iss,
    required this.aud,
    required this.exp,
    required this.iat,
    required this.jti,
    required this.typ,
    required this.azp,
    required this.scope,
    required this.preferredUsername,
    required this.email,
    required this.realmAccess,
    required this.resourceAccess,
    required this.tierAccess,
  });

  factory Claims.fromJson(Map<String, dynamic> json) => Claims(
        sub: json['sub'] as String,
        iss: json['iss'] as String,
        aud: json['aud'] as String,
        exp: json['exp'] as int,
        iat: json['iat'] as int,
        jti: json['jti'] as String,
        typ: json['typ'] as String,
        azp: json['azp'] as String,
        scope: json['scope'] as String,
        preferredUsername: json['preferred_username'] as String,
        email: json['email'] as String,
        realmAccess: RealmAccess.fromJson(
            json['realm_access'] as Map<String, dynamic>),
        resourceAccess:
            (json['resource_access'] as Map<String, dynamic>).map(
          (k, v) =>
              MapEntry(k, ResourceRoles.fromJson(v as Map<String, dynamic>)),
        ),
        tierAccess: (json['tier_access'] as List).cast<String>(),
      );
}

/// Realm アクセスロール
class RealmAccess {
  final List<String> roles;

  RealmAccess({required this.roles});

  factory RealmAccess.fromJson(Map<String, dynamic> json) => RealmAccess(
        roles: (json['roles'] as List).cast<String>(),
      );
}

/// リソース別ロール
class ResourceRoles {
  final List<String> roles;

  ResourceRoles({required this.roles});

  factory ResourceRoles.fromJson(Map<String, dynamic> json) => ResourceRoles(
        roles: (json['roles'] as List).cast<String>(),
      );
}

/// OIDC Discovery レスポンス
class OIDCDiscovery {
  final String authorizationEndpoint;
  final String tokenEndpoint;
  final String endSessionEndpoint;
  final String jwksUri;
  final String issuer;

  OIDCDiscovery({
    required this.authorizationEndpoint,
    required this.tokenEndpoint,
    required this.endSessionEndpoint,
    required this.jwksUri,
    required this.issuer,
  });

  factory OIDCDiscovery.fromJson(Map<String, dynamic> json) => OIDCDiscovery(
        authorizationEndpoint: json['authorization_endpoint'] as String,
        tokenEndpoint: json['token_endpoint'] as String,
        endSessionEndpoint: json['end_session_endpoint'] as String,
        jwksUri: json['jwks_uri'] as String,
        issuer: json['issuer'] as String,
      );
}

/// 認証状態変更コールバック
typedef AuthStateCallback = void Function(bool authenticated);

/// 認証エラー
class AuthError implements Exception {
  final String message;

  AuthError(this.message);

  @override
  String toString() => 'AuthError: $message';
}
