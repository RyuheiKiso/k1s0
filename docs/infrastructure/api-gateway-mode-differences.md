# Kong モード差異: 開発（DB-less）vs 本番（DB-backed）

## KONG-03 監査対応

| 環境 | Kong モード | 設定管理 |
|------|------------|---------|
| Docker Compose（開発） | DB-less (`KONG_DATABASE: "off"`) | 宣言型 YAML（`infra/kong/kong.dev.yaml`） |
| Kubernetes 本番 | DB-backed (`KONG_DATABASE: postgres`) | Admin API + deck sync |

## 差異と注意点

### ルーティング挙動の差異
- DB-less は `kong.dev.yaml` の宣言型設定を使用。Admin API への変更は再起動で失われる。
- DB-backed は PostgreSQL に設定を永続化。deck sync（`.github/workflows/kong-sync.yaml`）でGitHub管理。

### 開発環境での検証について
Docker Compose でのルーティング検証は本番 DB-backed 環境での動作を完全には再現しない。
重大なルーティング変更は staging 環境（DB-backed）でも必ず検証すること。

### 設計上の意図
開発環境では設定変更の高速サイクルを優先して DB-less を採用している。
本番環境では設定永続化・クラスタリング・Admin API の活用のために DB-backed を使用する。
