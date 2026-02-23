# k1s0-auth

k1s0 認証・認可 Swift ライブラリ

Keycloak 発行の JWT を検証し、RBAC によるアクセス制御を提供します。

## 使い方

```swift
import K1s0Auth

let verifier = JwksVerifier(
    jwksURL: URL(string: "https://keycloak.example.com/realms/k1s0/protocol/openid-connect/certs")!,
    issuer: "https://keycloak.example.com/realms/k1s0",
    audience: "k1s0-api"
)

let claims = try await verifier.verify(token: bearerToken)

if RBAC.hasRole(claims, role: "admin") {
    // 管理者向け処理
}
```

## 開発

```bash
swift build
swift test
```
