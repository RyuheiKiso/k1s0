# 03. Go services 配置

本ファイルは `src/tier2/go/` 配下の Go サービス配置を確定する。tier1 Go とは独立した go.mod を持つ設計、サービスごとの cmd/ + internal/ 構造、pod 単位のビルド独立性を規定する。

## レイアウト

```
src/tier2/go/
├── README.md
├── go.mod                          # module github.com/k1s0/k1s0/src/tier2/go
├── go.sum
├── services/
│   ├── stock-reconciler/
│   │   ├── cmd/
│   │   │   └── main.go
│   │   ├── internal/
│   │   │   ├── config/
│   │   │   ├── domain/             # Domain 層
│   │   │   ├── application/        # Application 層（UseCases）
│   │   │   ├── infrastructure/     # Infrastructure 層
│   │   │   └── api/                # Api 層（HTTP / gRPC ハンドラ）
│   │   ├── tests/
│   │   └── Dockerfile
│   └── notification-hub/
│       ├── cmd/
│       │   └── main.go
│       ├── internal/
│       └── Dockerfile
├── shared/                         # tier2 Go 内の共通 lib
│   ├── dapr/
│   ├── otel/
│   └── errors/
└── tools/                          # tier2 Go 固有のスクリプト
    └── gen-mocks.sh
```

## tier1 Go との分離

`src/tier1/go/go.mod` と `src/tier2/go/go.mod` は独立。理由は以下。

- 依存管理の独立性: tier1 Go は Dapr SDK への依存が強く、tier2 Go はビジネスロジック寄りの依存（chi / gorm など）が中心。バージョン競合を避けるため
- ビルドの並列性: 別 go.mod にすることで `go build` のキャッシュが独立、CI ジョブの並列度が上がる
- import path の明示: `github.com/k1s0/k1s0/src/tier2/go/...` が tier2 専用であることが import 文で自明になる

## go.mod の推奨サンプル

```go
module github.com/k1s0/k1s0/src/tier2/go

go 1.22

require (
    github.com/k1s0/k1s0/src/sdk/go v0.1.0      // tier1 公開 API クライアント
    github.com/dapr/go-sdk v1.11.0               // Dapr Building Blocks（一部直接利用）
    github.com/go-chi/chi/v5 v5.1.0              // HTTP router
    gorm.io/gorm v1.25.12                        // ORM
    gorm.io/driver/postgres v1.5.11
    github.com/confluentinc/confluent-kafka-go/v2 v2.6.0  // Kafka
    go.opentelemetry.io/otel v1.30.0
    github.com/stretchr/testify v1.10.0
)

replace (
    // Phase 1a の初期開発中は local path で SDK を参照（後で削除）
    // github.com/k1s0/k1s0/src/sdk/go => ../../sdk/go
)
```

Phase 1b 以降、SDK が GitHub Packages / 社内プライベートに publish されたら `replace` を削除し、正規の NuGet / Go module 依存に切り替える。

## サービス単位の独立ビルド

各サービスの `cmd/main.go` は独立のエントリポイント。`go build ./services/<service>/cmd/` で個別ビルド可能。

```go
// services/stock-reconciler/cmd/main.go
//
// stock-reconciler サービスのエントリポイント
package main

// 外部依存のインポート
import (
    "context"
    "os/signal"
    "syscall"

    "github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/api"
    "github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/application"
    "github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
    "github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/infrastructure"
    "github.com/k1s0/k1s0/src/tier2/go/shared/otel"
)

func main() {
    // 設定をロード
    cfg := config.Load()

    // OpenTelemetry 初期化
    shutdown := otel.Init(cfg.Otel)
    defer shutdown()

    // Infrastructure 層の初期化
    db := infrastructure.NewDatabase(cfg.Database)
    sdkClient := infrastructure.NewK1s0Client(cfg.Sdk)

    // Application 層の初期化
    useCase := application.NewReconcilerUseCase(db, sdkClient)

    // Api 層の起動
    server := api.NewServer(useCase, cfg.Api)

    // graceful shutdown
    ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
    defer stop()

    server.Run(ctx)
}
```

## Dockerfile

各サービスの Dockerfile は multi-stage。

```dockerfile
# services/stock-reconciler/Dockerfile
FROM golang:1.22 AS builder
WORKDIR /workspace
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -trimpath \
    -o bin/stock-reconciler \
    ./services/stock-reconciler/cmd/

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /workspace/bin/stock-reconciler /usr/local/bin/stock-reconciler
USER nonroot
EXPOSE 8080 9090
ENTRYPOINT ["/usr/local/bin/stock-reconciler"]
```

## shared/ の位置付け

`src/tier2/go/shared/` は tier2 Go サービス間で共有するユーティリティ。

- `dapr/`: Dapr SDK のラッパー（ログ出力や metrics 取得のフックを共通化）
- `otel/`: OpenTelemetry 初期化ボイラープレート
- `errors/`: 共通エラー型（tier2 専用の E-T2-\* 体系）

shared は tier2 内部の利用のみ許容。tier1 / tier3 から直接参照することはできない（Go internal 機構で強制するため `shared/` を `internal/shared/` にリネームすべきか検討したが、tier2 全サービスから共通参照する必要があるため `shared/` の名前で公開する）。

ただし `shared/` 配下は tier2 Go の内部 API と位置付け、外部への公開 API ではない。安定した API が必要な場合は SDK（`src/sdk/go/`）に昇格する。

## 依存方向

- tier2 Go は `src/sdk/go/` を介して tier1 にアクセス
- tier1 Go の internal を直接参照することは禁止
- tier2 内部の他サービスの internal を参照することは禁止（Go の internal 機構で自動強制）

## CODEOWNERS

```
/src/tier2/go/                                  @k1s0/tier2-dev
```

## スパースチェックアウト cone

- `tier2-dev` cone に `src/tier2/go/` を含む

## 対応 IMP-DIR ID

- IMP-DIR-T2-043（go services 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003
- DS-SW-COMP-019
- FR-\* / DX-CICD-\*
