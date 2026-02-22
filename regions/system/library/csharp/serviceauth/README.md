# K1s0.System.ServiceAuth

サービス間 OAuth2 Client Credentials 認証ライブラリ (C#)。

## 機能

- **ServiceAuthClient**: OAuth2 Client Credentials フローによるトークン取得
- **トークンキャッシュ**: SemaphoreSlim によるスレッドセーフなキャッシュ (有効期限 60 秒前に自動更新)
- **SpiffeId**: SPIFFE URI (`spiffe://<trust-domain>/ns/<ns>/sa/<sa>`) のパース・検証
- **DI 拡張**: `AddK1s0ServiceAuth` で簡単にサービス登録

## 使用方法

```csharp
using K1s0.System.ServiceAuth;

// DI 登録
services.AddK1s0ServiceAuth(new ServiceAuthConfig(
    TokenUrl: "https://auth.example.com/realms/k1s0/protocol/openid-connect/token",
    ClientId: "my-service",
    ClientSecret: "my-secret",
    Scopes: ["openid"]));

// トークン取得 (キャッシュ付き)
var token = await serviceAuthClient.GetCachedTokenAsync();
httpClient.DefaultRequestHeaders.Authorization =
    new AuthenticationHeaderValue(token.TokenType, token.AccessToken);

// SPIFFE ID 検証
var spiffe = await serviceAuthClient.ValidateSpiffeIdAsync(
    "spiffe://k1s0.internal/ns/system/sa/auth-service",
    expectedNamespace: "system");
```

## テスト

```bash
dotnet test tests/
```
