# k1s0-service-auth

k1s0 service_auth ライブラリ — サービス間 OAuth2 Client Credentials 認証を提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_service_auth import HttpServiceAuthClient, ServiceAuthConfig

config = ServiceAuthConfig(
    token_url="https://auth.example.com/token",
    client_id="my-service",
    client_secret="my-secret",
    scope="api.read",
)
client = HttpServiceAuthClient(config)
token = client.get_cached_token()
print(token.access_token)
```

## 開発

```bash
uv run pytest
```
