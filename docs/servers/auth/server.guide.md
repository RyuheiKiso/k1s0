# system-auth-server 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### POST /api/v1/auth/token/validate

JWT トークンを検証し、有効であれば Claims を返却する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIs..."
}
```

**レスポンス（200 OK）**

```json
{
  "valid": true,
  "claims": {
    "sub": "user-uuid-1234",
    "iss": "https://auth.k1s0.internal.example.com/realms/k1s0",
    "aud": "k1s0-api",
    "exp": 1740000000,
    "iat": 1739996400,
    "jti": "token-uuid-5678",
    "preferred_username": "taro.yamada",
    "email": "taro.yamada@example.com",
    "scope": "openid profile email",
    "realm_access": {
      "roles": ["sys_auditor"]
    }
  }
}
```

**レスポンス（401 Unauthorized）**

```json
{
  "error": {
    "code": "SYS_AUTH_TOKEN_INVALID",
    "message": "Token validation failed",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/auth/token/introspect

RFC 7662 準拠のトークンイントロスペクション。トークンが無効でも 200 を返し、`active: false` で応答する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type_hint": "access_token"
}
```

**レスポンス（200 OK - アクティブ）**

```json
{
  "active": true,
  "sub": "user-uuid-1234",
  "client_id": "react-spa",
  "username": "taro.yamada",
  "token_type": "Bearer",
  "exp": 1740000000,
  "iat": 1739996400,
  "scope": "openid profile email",
  "realm_access": {
    "roles": ["sys_auditor"]
  }
}
```

**レスポンス（200 OK - 非アクティブ）**

```json
{
  "active": false
}
```

### GET /api/v1/users

**レスポンス（200 OK）**

```json
{
  "users": [
    {
      "id": "user-uuid-1234",
      "username": "taro.yamada",
      "email": "taro.yamada@example.com",
      "display_name": "Taro Yamada",
      "status": "active",
      "email_verified": true,
      "created_at": "2026-01-15T09:00:00Z",
      "attributes": {}
    }
  ],
  "pagination": {
    "total_count": 42,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

### GET /api/v1/users/:id

**レスポンス（200 OK）**

```json
{
  "id": "user-uuid-1234",
  "username": "taro.yamada",
  "email": "taro.yamada@example.com",
  "display_name": "Taro Yamada",
  "status": "active",
  "email_verified": true,
  "created_at": "2026-01-15T09:00:00Z",
  "attributes": {}
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_AUTH_USER_NOT_FOUND",
    "message": "The specified user was not found",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/users/:id/roles

**レスポンス（200 OK）**

```json
{
  "user_id": "user-uuid-1234",
  "realm_roles": [
    {
      "id": "role-1",
      "name": "user",
      "description": "General user"
    },
    {
      "id": "role-2",
      "name": "sys_admin",
      "description": "System admin"
    }
  ],
  "client_roles": {
    "order-service": [
      {
        "id": "role-3",
        "name": "read",
        "description": "Read access"
      }
    ]
  }
}
```

### POST /api/v1/auth/permissions/check

**リクエスト**

```json
{
  "roles": ["sys_admin"],
  "permission": "admin",
  "resource": "users"
}
```

**レスポンス（200 OK）**

```json
{
  "allowed": true,
  "reason": ""
}
```

### GET /api/v1/audit/logs

**レスポンス（200 OK）**

```json
{
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "event_type": "LOGIN_SUCCESS",
      "user_id": "user-uuid-1234",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0",
      "resource": "/api/v1/auth/token",
      "resource_id": null,
      "action": "POST",
      "result": "SUCCESS",
      "detail": {"client_id": "react-spa"},
      "trace_id": "trace-001",
      "created_at": "2026-02-20T10:30:00Z"
    }
  ],
  "pagination": {
    "total_count": 1,
    "page": 1,
    "page_size": 50,
    "has_next": false
  }
}
```

### POST /api/v1/audit/logs

**リクエスト**

```json
{
  "event_type": "LOGIN_SUCCESS",
  "user_id": "user-uuid-1234",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0",
  "resource": "/api/v1/auth/token",
  "action": "POST",
  "result": "SUCCESS",
  "detail": {"client_id": "react-spa"}
}
```

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-02-20T10:30:00Z"
}
```

### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

### GET /readyz

**レスポンス（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "keycloak": "ok"
  }
}
```

**レスポンス（503 Service Unavailable）**

```json
{
  "status": "not ready",
  "checks": {
    "database": "ok",
    "keycloak": "error"
  }
}
```

---

## Keycloak JWKS 連携フロー

auth-server は Keycloak の JWKS エンドポイントから公開鍵を取得し、JWT の署名を検証する。

### トークン検証シーケンス

```
Client                  auth-server               Keycloak
  |                         |                         |
  |  POST /token/validate   |                         |
  |  { token: "eyJ..." }   |                         |
  |------------------------>|                         |
  |                         |                         |
  |                         |  GET /realms/k1s0/      |
  |                         |  protocol/openid-connect|
  |                         |  /certs                 |
  |                         |------------------------>|
  |                         |                         |
  |                         |  JWKS { keys: [...] }   |
  |                         |<------------------------|
  |                         |                         |
  |                         | 1. kid から公開鍵を特定   |
  |                         | 2. RS256 署名を検証       |
  |                         | 3. iss/aud/exp を検証     |
  |                         |                         |
  |  { valid: true,         |                         |
  |    claims: {...} }      |                         |
  |<------------------------|                         |
```

### JWKS キャッシュ戦略

- JWKS レスポンスはインメモリにキャッシュ
- kid が未知の場合のみ JWKS を再取得（rotate 対応）
- Keycloak の JWKS エンドポイント: `{keycloak.base_url}/realms/{keycloak.realm}/protocol/openid-connect/certs`

### トークン検証の詳細手順

1. JWT ヘッダーから `kid`（Key ID）を抽出
2. キャッシュ済み JWKS から対応する公開鍵を検索
3. 未発見の場合、Keycloak から JWKS を再取得してキャッシュを更新
4. 公開鍵で RS256 署名を検証
5. Claims を検証:
   - `iss` が `auth.jwt.issuer` と一致するか
   - `aud` が `auth.jwt.audience` と一致するか
   - `exp` が現在時刻より未来か
6. 検証成功時、Claims を返却

---

## RBAC ミドルウェア設計

### ミドルウェアスタック

保護エンドポイントには2段階のミドルウェアが適用される。

```
リクエスト
  |
  v
auth_middleware       -- Bearer トークン検証、Claims を Extensions に格納
  |
  v
rbac_middleware       -- Claims のロールが必要な権限を持つか判定
  |
  v
ハンドラー
```

### パーミッションキャッシュ

moka を使用したインメモリキャッシュで RBAC 判定結果をキャッシュする。

- TTL: `permission_cache.ttl_secs`（デフォルト 300 秒）
- キャッシュミス時の自動リフレッシュ: `permission_cache.refresh_on_miss`

---

## 依存関係図

```
                    ┌──────────────────────────────────────────────────────────────┐
                    │                       adapter 層                             │
                    │  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐  │
                    │  │ REST Handler │  │ gRPC Handler │  │ auth/rbac         │  │
                    │  │ (auth,audit, │  │ (AuthGrpc,   │  │ middleware        │  │
                    │  │  navigation) │  │  AuditGrpc)  │  │                   │  │
                    │  └──────┬───────┘  └──────┬───────┘  └─────────┬─────────┘  │
                    │         │                  │                    │            │
                    └─────────┼──────────────────┼────────────────────┼────────────┘
                              │                  │                    │
                    ┌─────────▼──────────────────▼────────────────────▼────────────┐
                    │                      usecase 層                              │
                    │  ValidateToken / GetUser / ListUsers / GetUserRoles /        │
                    │  CheckPermission / RecordAuditLog / SearchAuditLogs          │
                    └─────────┬────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/entity  │  │ domain/service │  │ domain/repository   │
    │  Claims, User,  │  │ (RBAC check   │  │ UserRepository      │
    │  AuditLog,      │  │  logic)        │  │ AuditLogRepository  │
    │  Role           │  │                │  │ (interface/trait)    │
    └────────────────┘  └────────────────┘  │                     │
                                            └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │             infrastructure 層         │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ Permission   │  │ PostgreSQL Repository  │  │
                    │  │ Cache (moka) │  │ (impl)                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Keycloak     │  │ Kafka Producer         │  │
                    │  │ Gateway      │  │ (audit events)         │  │
                    │  │ (JWKS/Admin) │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Config Loader          │  │
                    │  │ Telemetry    │  │ (config.yaml)          │  │
                    │  │ (Metrics)    │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```
