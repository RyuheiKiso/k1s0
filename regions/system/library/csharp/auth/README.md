# K1s0.System.Auth

JWT/JWKS ベースの認証ライブラリ (C#/.NET 10)。サーバーサイドの JWT 検証、RBAC 権限チェック、デバイスフロー認証を提供する。

## 機能

- **JWKS ベース JWT 検証**: JWKS エンドポイントから公開鍵を取得し、JWT トークンを検証
- **キャッシュ付き鍵取得**: `ReaderWriterLockSlim` による排他制御と TTL ベースのキャッシュ
- **RBAC 権限チェック**: スコープ、ロール、リソースアクセスに基づく権限判定
- **デバイスフロー**: OAuth 2.0 Device Authorization Grant のクライアント実装
- **ASP.NET Core ミドルウェア統合**: `UseK1s0JwtAuth()` による認証パイプライン設定

## インストール

プロジェクト参照:

```xml
<ProjectReference Include="..\auth\K1s0.System.Auth.csproj" />
```

## 使い方

### DI 登録

```csharp
var config = new AuthConfig
{
    JwksUrl = "https://auth.example.com/.well-known/jwks.json",
    Issuer = "https://auth.example.com",
    Audience = "k1s0-api",
    CacheTtlSeconds = 300,
};

builder.Services.AddK1s0Auth(config);
```

### JWT 検証

```csharp
public class MyService(IJwksVerifier verifier)
{
    public async Task<TokenClaims> ValidateAsync(string token)
    {
        return await verifier.VerifyTokenAsync(token);
    }
}
```

### RBAC チェック

```csharp
var allowed = RbacChecker.CheckPermission(claims, "orders", "read");
```

### ミドルウェア

```csharp
app.UseK1s0JwtAuth();
```

## テスト

```bash
dotnet test regions/system/library/csharp/auth/tests/
```

## 主要な型

| 型 | 説明 |
|---|---|
| `AuthConfig` | JWKS URL, Issuer, Audience, CacheTTL の設定 |
| `TokenClaims` | JWT クレームの record 型 |
| `IJwksFetcher` | JWKS 鍵取得インターフェース |
| `HttpJwksFetcher` | HTTP ベースの JWKS 取得実装 |
| `IJwksVerifier` | JWT 検証インターフェース |
| `JwksVerifier` | キャッシュ付き JWT 検証実装 |
| `RbacChecker` | RBAC 権限チェックユーティリティ |
| `DeviceFlowClient` | OAuth 2.0 デバイスフロークライアント |
| `AuthException` | 認証エラー例外 (Code プロパティ付き) |
