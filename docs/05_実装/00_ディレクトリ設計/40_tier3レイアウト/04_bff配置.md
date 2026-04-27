# 04. BFF 配置

本ファイルは `src/tier3/bff/` 配下の Backend For Frontend 配置を確定する。BFF は tier3 クライアント（Web / Native）ごとに最適化された HTTP / gRPC エンドポイントを提供し、tier1 と tier2 への複数呼び出しを集約する。

## BFF パターンの必要性

tier3 クライアントは以下の理由で BFF を介して tier1 / tier2 にアクセスする。

- **複合クエリの集約**: 1 画面の表示に複数の tier1 API を呼ぶ必要がある場合、クライアント側で N+1 リクエストにならないよう BFF で集約
- **レスポンス形式の最適化**: Web 向けは GraphQL、Native 向けは gRPC、といったクライアント固有最適化
- **認可フィルタの集中**: tier1 API が返すデータのうち、クライアント単位で絞り込む認可ルールを BFF に集約
- **プロトコル変換**: クライアントは REST / GraphQL / gRPC-Web、tier1 / tier2 は gRPC、の変換レイヤー
- **キャッシュ**: クライアント視点の短時間キャッシュ（Redis Valkey）

## レイアウト

```text
src/tier3/bff/
├── README.md
├── go.mod                          # module github.com/k1s0/k1s0/src/tier3/bff
├── go.sum
├── cmd/
│   ├── portal-bff/
│   │   └── main.go                 # portal Web 向け BFF
│   └── admin-bff/
│       └── main.go                 # admin Web 向け BFF
├── internal/
│   ├── graphql/                    # GraphQL スキーマとリゾルバ
│   │   ├── schema.graphql
│   │   ├── resolver/
│   │   └── generated/              # gqlgen 生成物
│   ├── rest/                       # REST エンドポイント
│   │   ├── router.go
│   │   └── handler/
│   ├── grpcweb/                    # gRPC-Web 透過プロキシ
│   │   └── proxy.go
│   ├── k1s0client/                 # tier1 / tier2 クライアント集約
│   │   ├── tier1_client.go
│   │   └── tier2_client.go
│   ├── auth/                       # 認可フィルタ / セッション管理
│   │   └── middleware.go
│   ├── cache/                      # Valkey 連携
│   │   └── client.go
│   ├── config/
│   │   └── config.go
│   └── shared/                     # 複数 BFF で共通化するロジック
│       ├── otel/
│       └── errors/
├── Dockerfile.portal
├── Dockerfile.admin
└── tests/
    ├── integration/
    └── fixtures/
```

## BFF の 3 種類のプロトコルサポート

tier3 BFF は以下の 3 プロトコルを同時にサポートする。

- **GraphQL**（Apollo Client / Relay など）: Web の複合クエリに最適
- **REST**（OpenAPI 仕様）: レガシー連携や簡易 Web ページに対応
- **gRPC-Web**（connectrpc）: 強い型付けが必要な Native / 上級 Web 開発者向け

3 プロトコルを同居させるのは複雑度を増やすが、リリース時点で portal Web（GraphQL） / Native（gRPC-Web）、採用後の運用拡大時 で外部連携（REST）と段階的に有効化する。

## portal-bff と admin-bff の分離

1 つの BFF で全 Web アプリを処理するのではなく、アプリ単位で独立 BFF Pod を運用する。理由は以下。

- **権限分離**: portal は一般ユーザ、admin は管理者。BFF の認可ロジックが異なる
- **スケーリング**: portal は高トラフィック、admin は低トラフィック。Pod 数を独立に調整
- **影響範囲の限定**: admin BFF のバグが portal に波及しない
- **CI ビルド時間**: 変更が 1 BFF のみに影響すれば、該当 Pod のみ再ビルド

## cmd/portal-bff/main.go の雛形

```go
// cmd/portal-bff/main.go
//
// portal Web 向け BFF のエントリポイント
package main

// 依存インポート
import (
    "context"
    "os/signal"
    "syscall"

    "github.com/k1s0/k1s0/src/tier3/bff/internal/auth"
    "github.com/k1s0/k1s0/src/tier3/bff/internal/config"
    "github.com/k1s0/k1s0/src/tier3/bff/internal/graphql"
    "github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
    "github.com/k1s0/k1s0/src/tier3/bff/internal/shared/otel"
)

func main() {
    // 設定をロード
    cfg := config.Load("portal-bff")

    // OpenTelemetry 初期化
    shutdown := otel.Init(cfg.Otel)
    defer shutdown()

    // tier1 / tier2 クライアント
    t1 := k1s0client.NewTier1Client(cfg.Tier1)
    t2 := k1s0client.NewTier2Client(cfg.Tier2)

    // GraphQL リゾルバ構築
    resolver := graphql.NewResolver(t1, t2)

    // 認可ミドルウェア
    authMw := auth.NewMiddleware(cfg.Keycloak)

    // HTTP サーバ起動
    server := graphql.NewServer(resolver, authMw, cfg.Api)

    ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
    defer stop()

    server.Run(ctx)
}
```

## GraphQL スキーマ

portal BFF の GraphQL スキーマは `internal/graphql/schema.graphql` に配置する。tier1 の gRPC 型とは独立した、Web 画面向けの型を定義する。

```graphql
# internal/graphql/schema.graphql
type Query {
  currentUser: User
  tenantDashboard(tenantId: ID!): TenantDashboard
}

type User {
  id: ID!
  email: String!
  roles: [String!]!
}

type TenantDashboard {
  tenant: Tenant
  recentActivity: [ActivityLog!]!
  keyMetrics: DashboardMetrics
}
```

`gqlgen` で Go コードに generate し、`internal/graphql/generated/` に出力。生成物は commit する（契約と同じ方針、ADR-DIR-001 の延長）。

## Container image

```dockerfile
# Dockerfile.portal
FROM golang:1.22 AS builder
WORKDIR /workspace
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -trimpath \
    -o bin/portal-bff ./cmd/portal-bff/

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /workspace/bin/portal-bff /usr/local/bin/portal-bff
USER nonroot
EXPOSE 8080 9090
ENTRYPOINT ["/usr/local/bin/portal-bff"]
```

## 依存方向

- BFF は `src/sdk/go/` を介して tier1 / tier2 にアクセス
- tier1 / tier2 の internal を直接参照することは禁止
- Web / Native は BFF の HTTP / gRPC エンドポイントを呼ぶ。BFF の Go コードを参照することは禁止

## スケーリングと HPA

portal と admin はトラフィック特性が異なるため、独立した HPA 設定を持つ。`deploy/charts/tier3-bff/values-<env>.yaml` で以下を指定する。CPU だけでなくメモリも target に含める理由は、GraphQL レスポンスの一時バッファ・Redis キャッシュの client-side LRU・Apollo Federation のクエリプランキャッシュなど BFF 特有のメモリ偏在ワークロードがあり、CPU 単独では先にメモリ枯渇で OOMKilled する事故が起きやすいため（NFR-B-PERF-* の応答性能要件と NFR-A-AVL-* の可用性要件の両立を狙う）。

| 対象 | minReplicas | maxReplicas | CPU target | Memory target | 備考 |
|---|---|---|---|---|---|
| portal-bff（prod） | 2 | 10 | 70 | 80 | 一般ユーザ向けの高トラフィック。PDB `minAvailable: 2` で常時冗長化 |
| portal-bff（dev / staging） | 1 | 3 | 70 | 80 | リソース節約、PDB 無効 |
| admin-bff（prod） | 1 | 3 | 70 | 80 | 管理者のみのため低トラフィック。夜間バッチは KEDA の event-driven scaling で補う |
| admin-bff（dev / staging） | 1 | 2 | 70 | 80 | 最小構成 |

`HorizontalPodAutoscaler` の `metrics:` には `Resource` 型を 2 件並列で並べる（`type: Resource, resource.name: cpu, target.averageUtilization: 70` と `type: Resource, resource.name: memory, target.averageUtilization: 80`）。HPA は OR 評価で動作するため、CPU と Memory のどちらか一方が閾値を超えれば即時スケールアウトする。

Kafka lag / Redis キュー長など event-driven な自動スケールが必要な場合は、`deploy/charts/tier3-bff/values-<env>.yaml` の `keda:` セクションで `ScaledObject` を宣言する（`infra/scaling/keda/` が 運用蓄積後に KEDA Operator を展開）。

## テスト戦略

- unit test: standard `go test`
- integration test: testcontainers で Redis / 本物の tier1 Pod を起動し、BFF 経由の E2E を検証
- contract test: tier1 / tier2 との契約整合（Pact）

## スパースチェックアウト cone

- `tier3-web-dev` cone に `src/tier3/bff/` を含む
- `tier3-native-dev` cone には含めない（Native 側は BFF の HTTP / gRPC を呼ぶだけなので、BFF 実装の編集は tier3-web チームが担う）

## 対応 IMP-DIR ID

- IMP-DIR-T3-059（bff 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003
- FR-\* / DX-GP-\* / NFR-B-PERF-\*（BFF のキャッシュによる応答性能向上）
