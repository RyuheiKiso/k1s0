# authlib ライブラリ設計

> 詳細な認証設計は [認証認可設計.md](認証認可設計.md) を参照。

## サーバー用 API（Go / Rust）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| NewJWKSVerifier | `(jwksURL, cacheTTL) -> Verifier` | JWKS 検証器を生成 |
| VerifyToken | `(tokenString) -> Claims, Error` | JWT トークンを検証 |
| CheckPermission | `(claims, resource, action) -> bool` | RBAC 権限チェック |
| AuthMiddleware | `(verifier) -> Middleware` | HTTP/gRPC 認証ミドルウェア |

## クライアント用 API（TypeScript / Dart）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| createAuthClient | `(config) -> AuthClient` | OAuth2 PKCE クライアント生成 |
| login | `() -> TokenSet` | 認証フロー開始 |
| logout | `() -> void` | ログアウト |
| getAccessToken | `() -> string` | アクセストークン取得（自動リフレッシュ） |
| isAuthenticated | `() -> bool` | 認証状態確認 |

## Go 実装

**配置先**: `regions/system/library/go/auth/`

```
auth/
├── jwks.go            # JWKS 検証器
├── claims.go          # Claims 型定義
├── middleware.go       # gin ミドルウェア
├── grpc_interceptor.go # gRPC インターセプター
├── rbac.go            # RBAC 権限チェック
├── jwks_test.go
├── middleware_test.go
├── rbac_test.go
├── go.mod
└── go.sum
```

**依存関係**:

```
github.com/lestrrat-go/jwx/v2
github.com/gin-gonic/gin
google.golang.org/grpc
```

**主要コード**:

```go
package authlib

import (
    "context"
    "fmt"
    "sync"
    "time"

    "github.com/lestrrat-go/jwx/v2/jwk"
    "github.com/lestrrat-go/jwx/v2/jwt"
)

type Claims struct {
    Sub            string              `json:"sub"`
    Issuer         string              `json:"iss"`
    Audience       []string            `json:"aud"`
    ExpiresAt      time.Time           `json:"exp"`
    IssuedAt       time.Time           `json:"iat"`
    Jti            string              `json:"jti"`
    Typ            string              `json:"typ"`
    Azp            string              `json:"azp"`
    Scope          string              `json:"scope"`
    Username       string              `json:"preferred_username"`
    Email          string              `json:"email"`
    RealmAccess    RealmAccess         `json:"realm_access"`
    ResourceAccess map[string]RoleSet  `json:"resource_access"`
    TierAccess     []string            `json:"tier_access"`
}

type RealmAccess struct {
    Roles []string `json:"roles"`
}

type RoleSet struct {
    Roles []string `json:"roles"`
}

type JWKSVerifier struct {
    jwksURL   string
    cacheTTL  time.Duration
    issuer    string
    audience  string
    mu        sync.RWMutex
    keySet    jwk.Set
    lastFetch time.Time
}

func NewJWKSVerifier(jwksURL, issuer, audience string, cacheTTL time.Duration) *JWKSVerifier {
    return &JWKSVerifier{
        jwksURL:  jwksURL,
        issuer:   issuer,
        audience: audience,
        cacheTTL: cacheTTL,
    }
}

func (v *JWKSVerifier) VerifyToken(ctx context.Context, tokenString string) (*Claims, error) {
    keySet, err := v.getKeySet(ctx)
    if err != nil {
        return nil, fmt.Errorf("failed to get JWKS: %w", err)
    }

    token, err := jwt.Parse([]byte(tokenString),
        jwt.WithKeySet(keySet),
        jwt.WithIssuer(v.issuer),
        jwt.WithAudience(v.audience),
        jwt.WithValidate(true),
    )
    if err != nil {
        return nil, fmt.Errorf("token validation failed: %w", err)
    }

    return extractClaims(token)
}

func (v *JWKSVerifier) getKeySet(ctx context.Context) (jwk.Set, error) {
    v.mu.RLock()
    if v.keySet != nil && time.Since(v.lastFetch) < v.cacheTTL {
        defer v.mu.RUnlock()
        return v.keySet, nil
    }
    v.mu.RUnlock()

    v.mu.Lock()
    defer v.mu.Unlock()

    keySet, err := jwk.Fetch(ctx, v.jwksURL)
    if err != nil {
        return nil, err
    }
    v.keySet = keySet
    v.lastFetch = time.Now()
    return keySet, nil
}

func CheckPermission(claims *Claims, resource, action string) bool {
    for _, access := range claims.ResourceAccess {
        for _, role := range access.Roles {
            if role == action || role == "admin" {
                return true
            }
        }
    }
    for _, role := range claims.RealmAccess.Roles {
        if role == "admin" {
            return true
        }
    }
    return false
}
```

## Rust 実装

**配置先**: `regions/system/library/rust/auth/`

**Cargo.toml**:

```toml
[package]
name = "k1s0-auth"
version = "0.1.0"
edition = "2021"

[dependencies]
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync", "time"] }
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

**主要コード**:

```rust
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub exp: u64,
    pub iat: u64,
    pub jti: Option<String>,
    pub typ: Option<String>,
    pub azp: Option<String>,
    pub scope: Option<String>,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<HashMap<String, RoleSet>>,
    pub tier_access: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleSet {
    pub roles: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("token expired")]
    TokenExpired,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),
    #[error("permission denied")]
    PermissionDenied,
}

pub struct JwksVerifier {
    jwks_url: String,
    issuer: String,
    audience: String,
    cache_ttl: std::time::Duration,
    keys: Arc<RwLock<Option<(Vec<Jwk>, std::time::Instant)>>>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

impl JwksVerifier {
    pub fn new(jwks_url: &str, issuer: &str, audience: &str, cache_ttl: std::time::Duration) -> Self {
        Self {
            jwks_url: jwks_url.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            cache_ttl,
            keys: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let keys = self.get_keys().await?;
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let kid = header.kid.ok_or_else(|| AuthError::InvalidToken("missing kid".into()))?;

        let jwk = keys.iter().find(|k| k.kid == kid)
            .ok_or_else(|| AuthError::InvalidToken("key not found".into()))?;

        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let data = decode::<Claims>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(data.claims)
    }

    async fn get_keys(&self) -> Result<Vec<Jwk>, AuthError> {
        let cache = self.keys.read().await;
        if let Some((ref keys, ref fetched_at)) = *cache {
            if fetched_at.elapsed() < self.cache_ttl {
                return Ok(keys.clone());
            }
        }
        drop(cache);

        let resp: JwksResponse = reqwest::get(&self.jwks_url).await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?
            .json().await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        let mut cache = self.keys.write().await;
        *cache = Some((resp.keys.clone(), std::time::Instant::now()));
        Ok(resp.keys)
    }
}

pub fn check_permission(claims: &Claims, _resource: &str, action: &str) -> bool {
    if let Some(ref realm) = claims.realm_access {
        if realm.roles.contains(&"admin".to_string()) {
            return true;
        }
    }
    if let Some(ref resources) = claims.resource_access {
        for roles in resources.values() {
            if roles.roles.contains(&action.to_string()) || roles.roles.contains(&"admin".to_string()) {
                return true;
            }
        }
    }
    false
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/auth/`

**package.json**:

```json
{
  "name": "@k1s0/auth",
  "version": "0.1.0",
  "dependencies": {
    "axios": "^1.7.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vitest": "^2.0.0"
  }
}
```

**主要コード**:

```typescript
// src/auth-client.ts
export interface AuthConfig {
  discoveryUrl: string;
  clientId: string;
  redirectUri: string;
  scopes: string[];
}

export interface TokenSet {
  accessToken: string;
  refreshToken: string;
  idToken: string;
  expiresAt: number;
}

export type AuthStateCallback = (authenticated: boolean) => void;

export class AuthClient {
  private config: AuthConfig;
  private tokenSet: TokenSet | null = null;
  private listeners: AuthStateCallback[] = [];
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(config: AuthConfig) {
    this.config = config;
  }

  async login(): Promise<void> {
    const { codeVerifier, codeChallenge } = await this.generatePKCE();
    const state = crypto.randomUUID();

    sessionStorage.setItem('pkce_verifier', codeVerifier);
    sessionStorage.setItem('oauth_state', state);

    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.config.clientId,
      redirect_uri: this.config.redirectUri,
      scope: this.config.scopes.join(' '),
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
      state,
    });

    window.location.href = `${await this.getAuthorizationEndpoint()}?${params}`;
  }

  async handleCallback(code: string, state: string): Promise<TokenSet> {
    const savedState = sessionStorage.getItem('oauth_state');
    if (state !== savedState) throw new Error('State mismatch');

    const codeVerifier = sessionStorage.getItem('pkce_verifier');
    if (!codeVerifier) throw new Error('Missing PKCE verifier');

    const tokenEndpoint = await this.getTokenEndpoint();
    const resp = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: this.config.clientId,
        code,
        redirect_uri: this.config.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    const data = await resp.json();
    this.tokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };

    this.scheduleRefresh();
    this.notifyListeners(true);
    return this.tokenSet;
  }

  async getAccessToken(): Promise<string> {
    if (!this.tokenSet) throw new Error('Not authenticated');
    if (Date.now() >= this.tokenSet.expiresAt - 60000) {
      await this.refreshToken();
    }
    return this.tokenSet.accessToken;
  }

  isAuthenticated(): boolean {
    return this.tokenSet !== null && Date.now() < this.tokenSet.expiresAt;
  }

  async logout(): Promise<void> {
    this.tokenSet = null;
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.notifyListeners(false);
  }

  onAuthStateChange(callback: AuthStateCallback): () => void {
    this.listeners.push(callback);
    return () => { this.listeners = this.listeners.filter(l => l !== callback); };
  }

  private async refreshToken(): Promise<void> {
    if (!this.tokenSet?.refreshToken) throw new Error('No refresh token');
    const tokenEndpoint = await this.getTokenEndpoint();
    const resp = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: this.config.clientId,
        refresh_token: this.tokenSet.refreshToken,
      }),
    });
    const data = await resp.json();
    this.tokenSet = {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      idToken: data.id_token,
      expiresAt: Date.now() + data.expires_in * 1000,
    };
    this.scheduleRefresh();
  }

  private scheduleRefresh(): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    if (!this.tokenSet) return;
    const delay = this.tokenSet.expiresAt - Date.now() - 60000;
    if (delay > 0) {
      this.refreshTimer = setTimeout(() => this.refreshToken(), delay);
    }
  }

  private notifyListeners(authenticated: boolean): void {
    this.listeners.forEach(cb => cb(authenticated));
  }

  private async getAuthorizationEndpoint(): Promise<string> {
    const discovery = await this.fetchDiscovery();
    return discovery.authorization_endpoint;
  }

  private async getTokenEndpoint(): Promise<string> {
    const discovery = await this.fetchDiscovery();
    return discovery.token_endpoint;
  }

  private discoveryCache: any = null;
  private async fetchDiscovery(): Promise<any> {
    if (!this.discoveryCache) {
      const resp = await fetch(this.config.discoveryUrl);
      this.discoveryCache = await resp.json();
    }
    return this.discoveryCache;
  }

  private async generatePKCE(): Promise<{ codeVerifier: string; codeChallenge: string }> {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    const codeVerifier = btoa(String.fromCharCode(...array))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(codeVerifier));
    const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    return { codeVerifier, codeChallenge };
  }
}
```

## Dart 実装

**配置先**: `regions/system/library/dart/auth/`

**pubspec.yaml**:

```yaml
name: k1s0_auth
version: 0.1.0
environment:
  sdk: ">=3.4.0 <4.0.0"
dependencies:
  dio: ^5.7.0
  flutter_secure_storage: ^9.2.0
  crypto: ^3.0.0
dev_dependencies:
  test: ^1.25.0
  mocktail: ^1.0.0
```

**主要コード**:

```dart
import 'dart:convert';
import 'dart:math';
import 'package:crypto/crypto.dart';
import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class AuthConfig {
  final String discoveryUrl;
  final String clientId;
  final String redirectUri;
  final List<String> scopes;

  AuthConfig({required this.discoveryUrl, required this.clientId, required this.redirectUri, required this.scopes});
}

class TokenSet {
  final String accessToken;
  final String refreshToken;
  final String idToken;
  final DateTime expiresAt;

  TokenSet({required this.accessToken, required this.refreshToken, required this.idToken, required this.expiresAt});
}

typedef AuthStateCallback = void Function(bool authenticated);

class AuthClient {
  final AuthConfig config;
  final FlutterSecureStorage _storage;
  final Dio _dio;
  TokenSet? _tokenSet;
  final List<AuthStateCallback> _listeners = [];
  Map<String, dynamic>? _discoveryCache;

  AuthClient(this.config)
      : _storage = const FlutterSecureStorage(),
        _dio = Dio();

  Future<String> getAuthorizationUrl() async {
    final discovery = await _fetchDiscovery();
    final codeVerifier = _generateCodeVerifier();
    final codeChallenge = _generateCodeChallenge(codeVerifier);
    final state = _generateRandomString(32);

    await _storage.write(key: 'pkce_verifier', value: codeVerifier);
    await _storage.write(key: 'oauth_state', value: state);

    final params = {
      'response_type': 'code',
      'client_id': config.clientId,
      'redirect_uri': config.redirectUri,
      'scope': config.scopes.join(' '),
      'code_challenge': codeChallenge,
      'code_challenge_method': 'S256',
      'state': state,
    };

    return '${discovery['authorization_endpoint']}?${Uri(queryParameters: params).query}';
  }

  Future<TokenSet> handleCallback(String code, String state) async {
    final savedState = await _storage.read(key: 'oauth_state');
    if (state != savedState) throw AuthError('State mismatch');

    final codeVerifier = await _storage.read(key: 'pkce_verifier');
    if (codeVerifier == null) throw AuthError('Missing PKCE verifier');

    final discovery = await _fetchDiscovery();
    final resp = await _dio.post(
      discovery['token_endpoint'] as String,
      data: {
        'grant_type': 'authorization_code',
        'client_id': config.clientId,
        'code': code,
        'redirect_uri': config.redirectUri,
        'code_verifier': codeVerifier,
      },
      options: Options(contentType: Headers.formUrlEncodedContentType),
    );

    _tokenSet = TokenSet(
      accessToken: resp.data['access_token'],
      refreshToken: resp.data['refresh_token'],
      idToken: resp.data['id_token'],
      expiresAt: DateTime.now().add(Duration(seconds: resp.data['expires_in'])),
    );

    await _persistTokens();
    _notifyListeners(true);
    return _tokenSet!;
  }

  Future<String> getAccessToken() async {
    if (_tokenSet == null) throw AuthError('Not authenticated');
    if (DateTime.now().isAfter(_tokenSet!.expiresAt.subtract(const Duration(minutes: 1)))) {
      await refreshToken();
    }
    return _tokenSet!.accessToken;
  }

  bool get isAuthenticated => _tokenSet != null && DateTime.now().isBefore(_tokenSet!.expiresAt);

  Future<void> refreshToken() async {
    if (_tokenSet?.refreshToken == null) throw AuthError('No refresh token');
    final discovery = await _fetchDiscovery();
    final resp = await _dio.post(
      discovery['token_endpoint'] as String,
      data: {
        'grant_type': 'refresh_token',
        'client_id': config.clientId,
        'refresh_token': _tokenSet!.refreshToken,
      },
      options: Options(contentType: Headers.formUrlEncodedContentType),
    );
    _tokenSet = TokenSet(
      accessToken: resp.data['access_token'],
      refreshToken: resp.data['refresh_token'],
      idToken: resp.data['id_token'],
      expiresAt: DateTime.now().add(Duration(seconds: resp.data['expires_in'])),
    );
    await _persistTokens();
  }

  Future<void> logout() async {
    _tokenSet = null;
    await _storage.deleteAll();
    _notifyListeners(false);
  }

  void Function() onAuthStateChange(AuthStateCallback callback) {
    _listeners.add(callback);
    return () => _listeners.remove(callback);
  }

  Future<void> _persistTokens() async {
    if (_tokenSet == null) return;
    await _storage.write(key: 'access_token', value: _tokenSet!.accessToken);
    await _storage.write(key: 'refresh_token', value: _tokenSet!.refreshToken);
  }

  void _notifyListeners(bool authenticated) {
    for (final cb in _listeners) { cb(authenticated); }
  }

  Future<Map<String, dynamic>> _fetchDiscovery() async {
    if (_discoveryCache != null) return _discoveryCache!;
    final resp = await _dio.get(config.discoveryUrl);
    _discoveryCache = resp.data as Map<String, dynamic>;
    return _discoveryCache!;
  }

  String _generateCodeVerifier() => _generateRandomString(43);

  String _generateCodeChallenge(String verifier) {
    final bytes = utf8.encode(verifier);
    final digest = sha256.convert(bytes);
    return base64Url.encode(digest.bytes).replaceAll('=', '');
  }

  String _generateRandomString(int length) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
    final random = Random.secure();
    return List.generate(length, (_) => chars[random.nextInt(chars.length)]).join();
  }
}

class AuthError implements Exception {
  final String message;
  AuthError(this.message);
  @override
  String toString() => 'AuthError: $message';
}
```

## C# 実装

**配置先**: `regions/system/library/csharp/auth/`

```
auth/
├── src/
│   ├── Auth.csproj
│   ├── IJwksVerifier.cs        # JWKS 検証インターフェース
│   ├── JwksVerifier.cs         # JWT 検証・JWKS キャッシュ実装
│   ├── IJwksFetcher.cs         # JWKS 取得インターフェース
│   ├── HttpJwksFetcher.cs      # HTTP ベースの JWKS 取得
│   ├── Claims.cs               # TokenClaims 型定義
│   ├── RbacChecker.cs          # RBAC 権限チェック
│   ├── DeviceFlowClient.cs     # Device Authorization Grant（オプショナル）
│   ├── Middleware/
│   │   └── JwtAuthMiddleware.cs # ASP.NET Core 認証ミドルウェア
│   └── AuthException.cs        # 公開例外型
├── tests/
│   ├── Auth.Tests.csproj
│   ├── Unit/
│   │   ├── JwksVerifierTests.cs
│   │   └── RbacCheckerTests.cs
│   └── Integration/
│       └── JwtAuthMiddlewareTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Microsoft.IdentityModel.Tokens | JWT トークン検証 |
| System.IdentityModel.Tokens.Jwt | JWT トークンハンドリング |

**名前空間**: `K1s0.System.Auth`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IJwksVerifier` | interface | JWT トークン検証の抽象 |
| `JwksVerifier` | class | JWKS ベースの JWT 検証（キャッシュ TTL 付き） |
| `IJwksFetcher` | interface | JWKS エンドポイントからの鍵取得抽象 |
| `HttpJwksFetcher` | class | HTTP ベースの JWKS 取得実装 |
| `TokenClaims` | record | JWT クレーム（Sub・Iss・Aud・RealmAccess・ResourceAccess 等） |
| `RbacChecker` | static class | RBAC 権限チェック |
| `DeviceFlowClient` | class | OAuth2 Device Authorization Grant クライアント（オプショナル） |
| `AuthException` | class | authlib の公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Auth;

public interface IJwksVerifier
{
    Task<TokenClaims> VerifyTokenAsync(
        string token,
        CancellationToken cancellationToken = default);
}

public interface IJwksFetcher
{
    Task<JsonWebKeySet> FetchKeysAsync(
        CancellationToken cancellationToken = default);
}

public static class RbacChecker
{
    public static bool CheckPermission(
        TokenClaims claims,
        string resource,
        string action);
}
```

**DI 拡張・ミドルウェア登録**:

```csharp
public static class AuthExtensions
{
    public static IServiceCollection AddK1s0JwtAuth(
        this IServiceCollection services,
        string jwksUrl,
        string issuer,
        string audience,
        TimeSpan? cacheTtl = null);

    public static IApplicationBuilder UseK1s0JwtAuth(
        this IApplicationBuilder app);
}
```

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Authlib`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// JWKS 検証器（actor で並行安全）
public actor JwksVerifier {
    public init(jwksURL: URL, cacheTTL: Duration)
    public func verify(token: String) async throws -> Claims
}

// JWT クレーム
public struct Claims: Codable, Sendable {
    public let sub: String
    public let exp: Date
    public let iat: Date
    public let roles: [String]
    public let scopes: [String]
}

// ロールベースアクセス制御
public enum RBAC: Sendable {
    case admin
    case editor
    case viewer
    case custom(String)

    public static func hasRole(_ role: RBAC, in claims: Claims) -> Bool
}
```

### エラー型
```swift
public enum AuthError: Error, Sendable {
    case tokenExpired
    case invalidToken(String)
    case jwksFetchFailed(underlying: Error)
    case keyNotFound(kid: String)
    case algorithmMismatch
    case insufficientPermissions
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上
- [認証認可設計](認証認可設計.md) — OAuth2.0・JWT・RBAC

---

## Python 実装

### パッケージ構造

```
auth/
├── pyproject.toml
├── src/
│   └── k1s0_auth/
│       ├── __init__.py        # 公開 API エクスポート
│       ├── models.py          # TokenClaims・DeviceFlowResponse・TokenResponse データクラス
│       ├── verifier.py        # JwksVerifier（JWT 検証・同期/非同期）
│       ├── jwks.py            # JwksFetcher（ABC）・HttpJwksFetcher（TTL キャッシュ付き）
│       ├── pkce.py            # PKCE コードベリファイア/チャレンジ生成（RFC 7636）
│       ├── rbac.py            # RbacChecker（ロールベースアクセス制御）
│       ├── exceptions.py      # AuthError・AuthErrorCodes
│       └── py.typed           # PEP 561 型スタブマーカー
└── tests/
    ├── test_verifier.py
    ├── test_verifier_integration.py
    ├── test_jwks.py
    ├── test_pkce.py
    ├── test_models.py
    └── test_rbac.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `TokenClaims` | dataclass | JWT クレーム（sub・iss・aud・exp・iat・scope・roles・extra） |
| `JwksVerifier` | class | JWKS を使用した JWT 検証（同期 `verify_token` / 非同期 `verify_token_async`） |
| `JwksFetcher` | ABC | JWKS キーリスト取得の抽象基底クラス |
| `HttpJwksFetcher` | class | HTTP で JWKS を取得するフェッチャー（TTL キャッシュ付き） |
| `RbacChecker` | class | ロールベースアクセス制御チェッカー（permission_map ベース） |
| `DeviceFlowResponse` | dataclass | デバイスフロー開始レスポンス |
| `TokenResponse` | dataclass | トークン取得レスポンス |
| `AuthError` | Exception | auth ライブラリのエラー基底クラス（code・message・cause） |
| `AuthErrorCodes` | class | エラーコード定数（INVALID_TOKEN・EXPIRED_TOKEN・JWKS_FETCH_ERROR 等） |

### 使用例

```python
from k1s0_auth import JwksVerifier, HttpJwksFetcher, RbacChecker

# JWKS フェッチャー（TTL 300秒キャッシュ）
fetcher = HttpJwksFetcher(jwks_uri="http://localhost:8180/realms/k1s0/protocol/openid-connect/certs")

# JWT 検証器
verifier = JwksVerifier(
    issuer="http://localhost:8180/realms/k1s0",
    audience="k1s0-api",
    fetcher=fetcher,
)

# 同期検証
claims = verifier.verify_token(token_string)
print(claims.sub, claims.roles)

# 非同期検証
claims = await verifier.verify_token_async(token_string)

# RBAC チェック
checker = RbacChecker(permission_map={
    "admin": ["user:read", "user:write"],
    "viewer": ["user:read"],
})
has_access = checker.check_permission(claims, "user", "write")
```

### 依存ライブラリ

| パッケージ | 用途 |
|-----------|------|
| `pyjwt[crypto]` >= 2.9 | JWT トークンデコード・検証（RSA 署名対応） |
| `httpx` >= 0.27 | JWKS エンドポイントへの HTTP/非同期 HTTP リクエスト |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90% 以上（`pyproject.toml` の `fail_under = 90`）
- 実行: `pytest` / `ruff check .`
