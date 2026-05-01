# `examples/tier3-bff-graphql/` — portal-bff GraphQL 最小例

tier3 BFF（Backend For Frontend）レイヤの典型的な実装パタンを示す GraphQL 例。

## 目的

- `src/tier3/bff/cmd/portal-bff` と同じ構造（cmd / internal/{graphql,rest,k1s0client}）を
  新規メンバーが真似できる
- tier1 gRPC を `github.com/k1s0/sdk` 経由で呼び出し、GraphQL API として tier3 web に公開する
  典型的な配置を示す
- N+1 query 問題を DataLoader で解く実装パタン

## scope

| 段階 | 提供範囲 |
|---|---|
| リリース時点 | 最小完動: `cmd/portal-bff/main.go` (HTTP GraphQL minimal + tier1 SDK State.Get) + `Dockerfile` + `catalog-info.yaml` + `go.mod` |
| 採用初期 | `internal/{graphql,k1s0client,auth}` 層分離 + gqlgen schema/resolver + DataLoader + Keycloak OIDC ミドルウェア |
| 採用後の運用拡大時 | persisted queries / federation / subscription |

`go run ./cmd/portal-bff -listen :8080 -tier1-target localhost:50001` で起動できる。
`POST /graphql` に `{"query": "query { stateGet(store: $s, key: $k) { data etag } }",
"variables": {"store": "valkey-default", "key": "..."}}` を送ると tier1 facade
の State.Get を呼び出す (substring 一致での簡易ルータ、採用初期で gqlgen に置換)。

## 想定構成（採用初期）

```text
tier3-bff-graphql/
├── README.md                       # 本ファイル
├── go.mod                          # github.com/k1s0/k1s0/examples/tier3-bff-graphql
├── cmd/
│   └── example-bff/
│       └── main.go
├── internal/
│   ├── graphql/                    # gqlgen generated + resolver
│   │   ├── schema.graphqls
│   │   └── resolver/
│   ├── k1s0client/                 # tier1 gRPC client wrapper
│   └── auth/                       # Keycloak OIDC middleware
├── tests/
├── Dockerfile
└── catalog-info.yaml
```

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/`（tier3 BFF レイアウト準ずる）
- ADR-DEV-001（Paved Road）
- ADR-SEC-001（Keycloak OIDC）

## 参照する tier1 API（採用初期想定）

- StateService（ポータル状態取得）
- LogService（フロント発生イベントの集約ログ）
- AuditService（ユーザー操作の監査ログ）
- FeatureService（フィーチャーフラグ評価、flagd 経由）
