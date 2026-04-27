# tier3 BFF (Backend For Frontend)

tier3 クライアント（Web / Native）向けの BFF を 2 アプリ提供する。1 つの go.mod を共有し、`cmd/<app>/` 配下に独立したエントリポイントを置く。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md`
- 全体: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/01_tier3全体配置.md`

## レイアウト

```text
src/tier3/bff/
├── README.md
├── go.mod                          # tier3 BFF 共通 module
├── go.sum
├── cmd/
│   ├── portal-bff/main.go          # portal Web 向け BFF
│   └── admin-bff/main.go           # admin Web 向け BFF
├── internal/
│   ├── graphql/                    # GraphQL スキーマと resolver
│   ├── rest/                       # REST エンドポイント
│   ├── grpcweb/                    # gRPC-Web 透過プロキシ（リリース時点 placeholder）
│   ├── k1s0client/                 # tier1 / tier2 クライアント集約
│   ├── auth/                       # 認可ミドルウェア
│   ├── cache/                      # Valkey 連携（リリース時点 placeholder）
│   ├── config/                     # 環境変数 → Config
│   └── shared/{otel, errors}/      # OTel / エラー型
├── Dockerfile.portal
└── Dockerfile.admin
```

## 2 アプリ分離の理由

| 軸 | portal-bff | admin-bff |
|---|---|---|
| 想定ユーザ | エンドユーザ（一般） | 管理者 |
| トラフィック | 高 | 低 |
| 認可ポリシー | self-only / tenant scope | role:admin の elevation |
| HPA | min=2 / max=10 | min=1 / max=3 |

権限 / スケール特性が異なるため Pod を分離する（`docs/.../04_bff配置.md` の表参照）。

## エンドポイント

### portal-bff（POST /graphql）

GraphQL クエリを受ける。リリース時点 では最小 query のみ対応:

```graphql
type Query {
  currentUser: User
  stateGet(store: String!, key: String!): StateValue
}
```

リリース時点 で `gqlgen` ベースの resolver に置換する。

### admin-bff（GET /api/admin/tenants 等の REST）

管理者向けの管理 API。リリース時点 では構造のみ。

### 共通

- `GET /healthz` / `GET /readyz`
- `POST /api/state/get` (REST 版 State.Get、簡易呼出)

## 環境変数

| 変数 | 既定値 | 必須 | 説明 |
|---|---|---|---|
| `HTTP_ADDR` | `:8080` | - | listen address |
| `K1S0_TARGET` | tier1-state... | - | k1s0 facade gRPC target |
| `K1S0_TENANT_ID` | （無し） | ✓ | tier1 ガード必須 |
| `K1S0_SUBJECT` | `tier3/<app-name>` | - | 監査 identity（app 別に上書き） |

## ビルド

```bash
# tier3/bff ルートから。
go build ./cmd/portal-bff/
go build ./cmd/admin-bff/

# テスト。
go test ./...
```

## Dockerfile

- `Dockerfile.portal`: portal-bff 用（build context は `src/tier3/bff/`）
- `Dockerfile.admin`: admin-bff 用（同上）

```bash
docker build -f Dockerfile.portal -t ghcr.io/k1s0/t3-portal-bff:dev .
docker build -f Dockerfile.admin -t ghcr.io/k1s0/t3-admin-bff:dev .
```
