# k1s0-auth

k1s0 auth ライブラリ — JWT 検証・RBAC・OAuth2 PKCE/デバイスフローを提供します。

## インストール

```toml
# uv workspace メンバーとして追加済み
```

## 使い方

```python
from k1s0_auth import JwksVerifier, HttpJwksFetcher, RbacChecker

fetcher = HttpJwksFetcher(jwks_uri="https://auth.example.com/.well-known/jwks.json")
verifier = JwksVerifier(issuer="https://auth.example.com", audience="my-api", fetcher=fetcher)

claims = await verifier.verify_token_async(token)
print(claims.sub)
```

## 開発

```bash
uv run pytest
```
