# authlib ライブラリ設計

> 詳細な認証設計は [認証認可設計.md](../../architecture/auth/認証認可設計.md) を参照。

## サーバー用 API（Go / Rust）

### JWKSVerifier / トークン検証

| 関数・型 | シグネチャ | 説明 |
|---------|-----------|------|
| `NewJWKSVerifier` / `JwksVerifier::new` | `(jwksURL, issuer, audience, cacheTTL) -> Verifier` | JWKS 検証器を生成 |
| `VerifyToken` / `verify_token` | `(tokenString) -> (Claims, Error)` | JWT トークンを検証 |
| `Claims` | struct | JWT クレーム（sub, iss, aud, realm_access, resource_access, tier_access 等） |
| `AuthError` | enum | `TokenExpired` / `InvalidToken` / `JwksFetchFailed` / `PermissionDenied` |

### RBAC チェック関数

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `HasRole` / `has_role` | `(claims, role) -> bool` | レルムロール保有チェック |
| `HasResourceRole` / `has_resource_role` | `(claims, resource, role) -> bool` | リソースロール保有チェック |
| `HasPermission` / `has_permission` | `(claims, resource, action) -> bool` | リソース × アクション権限チェック |
| `HasTierAccess` / `has_tier_access` | `(claims, tier) -> bool` | tier_access チェック |

### ミドルウェア（HTTP / gRPC 認証ハンドラ）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| `AuthMiddleware` / `require_role` | `(verifier) -> Middleware` | JWT 検証 + Claims 注入ミドルウェア |
| `RequireRole` / `require_role` | `(role) -> Middleware` | ロール必須ミドルウェア |
| `RequirePermission` / `require_permission` | `(resource, action) -> Middleware` | RBAC 権限必須ミドルウェア |
| `RequireTierAccess` / `require_tier_access` | `(tier) -> Middleware` | Tier アクセス必須ミドルウェア |
| `GetClaims` / `get_claims` | `(ctx) -> Claims?` | コンテキストから Claims を取得 |
| `GetClaimsFromContext` | `(ctx) -> Claims?` | Go のみ: gin.Context から Claims を取得 |

## クライアント用 API（TypeScript / Dart）

| 関数 | シグネチャ | 説明 |
|------|-----------|------|
| new AuthClient | `(options: AuthClientOptions) -> AuthClient` | OAuth2 PKCE クライアント生成（`AuthClientOptions` は `config`, `tokenStore?`, `fetch?` 等を含む） |
| login | `() -> void` | 認証フロー開始（認可サーバーへリダイレクト） |
| handleCallback | `(code, state) -> TokenSet` | 認可コードを受け取りトークンを取得 |
| logout | `() -> void` | ログアウト |
| getAccessToken | `() -> string` | アクセストークン取得（自動リフレッシュ） |
| isAuthenticated | `() -> bool` | 認証状態確認 |

## Go 実装

**配置先**: `regions/system/library/go/auth/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

## デバイスフロー API（全言語）

CLI ツール等のブラウザを持たないクライアント向けに OAuth2 Device Authorization Grant Flow を提供する。

| 型・関数 | シグネチャ | 説明 |
|---------|-----------|------|
| DeviceAuthClient | `constructor(deviceEndpoint, tokenEndpoint)` | デバイスフロークライアント生成 |
| DeviceCodeResponse | `{ deviceCode, userCode, verificationUri, expiresIn, interval }` | デバイスコードレスポンス |
| TokenResult | `{ accessToken, refreshToken, tokenType, expiresIn }` | トークン取得結果 |
| DeviceFlowError | エラー型（ExpiredToken, AccessDenied 等） | デバイスフローエラー |
| requestDeviceCode | `(clientId, scope?) -> DeviceCodeResponse` | デバイスコードを要求 |
| pollToken | `(clientId, deviceCode, interval) -> TokenResult` | トークンをポーリング取得 |
| deviceFlow | `(clientId, scope?, onUserCode) -> TokenResult` | デバイスフロー完全実行 |

## クライアント補助 API（TypeScript / Dart）

PKCE フローと TokenStore の実装詳細。

| 型・関数 | 説明 |
|---------|------|
| TokenStore | トークン永続化インターフェース（`getTokenSet`, `setTokenSet`, `clearTokenSet`, `getState`, `setState` 等） |
| MemoryTokenStore | メモリ内 TokenStore 実装（テスト・SPA 向け） |
| LocalStorageTokenStore | localStorage ベースの TokenStore 実装（TypeScript） |
| generateCodeVerifier | PKCE コードベリファイア生成 |
| generateCodeChallenge | PKCE コードチャレンジ生成（SHA-256 ハッシュ） |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](serviceauth.md) — k1s0-serviceauth ライブラリ

---
