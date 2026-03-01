# k1s0-serviceauth ライブラリ設計

## 概要

サービス間 OAuth2 Client Credentials 認証ライブラリ。`ServiceAuthClient` トレイト（HTTP 実装: `HttpServiceAuthClient`）、`ServiceToken`（キャッシュ・自動更新）、`SpiffeId`（SPIFFE URI 検証）を提供する。Istio mTLS 環境でのワークロードアイデンティティ検証もサポートする。

**配置先**: `regions/system/library/rust/serviceauth/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `ServiceAuthClient` | トレイト | トークン取得・検証の抽象インターフェース（`get_token`, `get_cached_token`, `verify_token`, `validate_spiffe_id`）|
| `HttpServiceAuthClient` | 構造体 | OAuth2 Client Credentials フローの HTTP 実装 |
| `MockServiceAuthClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `ServiceClaims` | 構造体 | サービストークンのクレーム（`sub`・`iss`・`scope` 等） |
| `ServiceAuthConfig` | 構造体 | トークンエンドポイント・クライアント ID/シークレット・JWKS URI |
| `ServiceToken` | 構造体 | アクセストークン + 有効期限（キャッシュ・自動更新対応） |
| `SpiffeId` | 構造体 | SPIFFE URI のパース・検証（`spiffe://<trust-domain>/ns/<ns>/sa/<sa>`）。`parse(uri)`, `to_uri()`, `allows_tier_access(tier)` メソッドあり |
| `ServiceAuthError` | enum | トークン取得・検証・SPIFFE エラー型 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-serviceauth"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
jsonwebtoken = "9"
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-serviceauth = { path = "../../system/library/rust/serviceauth" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
serviceauth/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # ServiceAuthClient トレイト・HttpServiceAuthClient・ServiceClaims・MockServiceAuthClient
│   ├── config.rs       # ServiceAuthConfig（トークンエンドポイント・JWKS URI 等）
│   ├── error.rs        # ServiceAuthError
│   └── token.rs        # ServiceToken（有効期限管理）・SpiffeId（URI 検証）
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_serviceauth::{HttpServiceAuthClient, ServiceAuthClient, ServiceAuthConfig};

let config = ServiceAuthConfig::new(
    "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/token",
    "my-service",
    "my-secret",
)
.with_jwks_uri("https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs");

let client = HttpServiceAuthClient::new(config).unwrap();

// キャッシュ付きトークン取得（有効期限前に自動リフレッシュ）
let bearer = client.get_cached_token().await.unwrap();

// gRPC 発信時のヘッダー設定
let mut request = tonic::Request::new(payload);
request.metadata_mut().insert(
    "authorization",
    format!("Bearer {}", bearer.access_token).parse().unwrap(),
);

// SPIFFE ID 検証（Istio mTLS 環境）
let spiffe = client
    .validate_spiffe_id("spiffe://k1s0.internal/ns/system/sa/auth-service", "system")
    .unwrap();
```

## Go 実装

**配置先**: `regions/system/library/go/serviceauth/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/lestrrat-go/jwx/v2 v2.1.3`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type ServiceAuthClient interface {
    GetToken(ctx context.Context) (*ServiceToken, error)
    GetCachedToken(ctx context.Context) (string, error)
    ValidateSpiffeId(spiffeId string, expectedNamespace string) (*SpiffeId, error)
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/serviceauth/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface ServiceClaims {
  sub: string;
  iss: string;
  scope?: string;
  exp?: number;
}

export interface SpiffeId {
  trustDomain: string;
  namespace: string;
  serviceAccount: string;
  uri: string;
}

export function parseSpiffeId(uri: string): SpiffeId;
export function validateSpiffeId(uri: string, expectedNamespace: string): SpiffeId;

export interface ServiceToken {
  accessToken: string;
  tokenType: string;
  expiresAt: Date;
  scope?: string;
}

export function isExpired(token: ServiceToken): boolean;
export function shouldRefresh(token: ServiceToken): boolean;
export function bearerHeader(token: ServiceToken): string;

export interface ServiceAuthConfig {
  tokenEndpoint: string;
  clientId: string;
  clientSecret: string;
}

export interface ServiceAuthClient {
  getToken(): Promise<ServiceToken>;
  getCachedToken(): Promise<string>;
  validateSpiffeId(uri: string, expectedNamespace: string): SpiffeId;
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/serviceauth/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
class ServiceClaims {
  final String sub;
  final String iss;
  final String? scope;
  final int? exp;
}

class SpiffeId {
  final String trustDomain;
  final String namespace;
  final String serviceAccount;
  final String uri;
}

SpiffeId parseSpiffeId(String uri);
SpiffeId validateSpiffeId(String uri, String expectedNamespace);

class ServiceToken {
  final String accessToken;
  final String tokenType;
  final DateTime expiresAt;
  final String? scope;
}

bool isExpired(ServiceToken token);
bool shouldRefresh(ServiceToken token);
String bearerHeader(ServiceToken token);

class ServiceAuthConfig {
  final String tokenEndpoint;
  final String clientId;
  final String clientSecret;
  const ServiceAuthConfig({required tokenEndpoint, required clientId, required clientSecret});
}

abstract class ServiceAuthClient {
  Future<ServiceToken> getToken();
  Future<String> getCachedToken();
  SpiffeId validateSpiffeIdCheck(String uri, String expectedNamespace);
}

class HttpServiceAuthClient implements ServiceAuthClient {
  HttpServiceAuthClient(ServiceAuthConfig config, {http.Client? httpClient});
  Future<ServiceToken> getToken();
  Future<String> getCachedToken();
  SpiffeId validateSpiffeIdCheck(String uri, String expectedNamespace);
}

class ServiceAuthError implements Exception {
  final String message;
  final Object? cause;
}
```

**カバレッジ目標**: 90%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](authlib.md) — authlib ライブラリ
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](../messaging/kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](../messaging/outbox.md) — k1s0-outbox ライブラリ

---
