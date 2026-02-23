# k1s0-serviceauth

k1s0 サービス間認証 Swift ライブラリ

OAuth2 Client Credentials フローによるサービス間認証と SPIFFE ID 検証を提供します。

## 使い方

```swift
import K1s0ServiceAuth

let config = ServiceAuthConfig(
    tokenEndpoint: "https://keycloak.example.com/token",
    clientId: "my-service",
    clientSecret: "secret"
)
let client = URLSessionServiceAuthClient(config: config)
let token = try await client.getCachedToken()
```

## 開発

```bash
swift build
swift test
```
