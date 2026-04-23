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
    // Phase 1a の初期開発中は local path で SDK を参照する（SDK が未 publish のため）
    github.com/k1s0/k1s0/src/sdk/go => ../../sdk/go
)
```

Phase 1a は SDK を社内 registry に publish していないため、上記 `replace` directive で local path を参照して開発を進める。Phase 1b 以降、SDK が GitHub Packages / 社内プライベートに publish されたら `replace` 行をコメントアウトではなく削除し、正規の Go module 依存（`require github.com/k1s0/k1s0/src/sdk/go v0.1.0` のみ）に切り替える。`replace` の存在検出は `tools/ci/lint-replace-directive.sh` で Phase ゲートと連携する（Phase 1b 以降に `replace` が残っていれば fail）。

## 複数 module の開発体験: go.work

k1s0 は `src/tier1/go/` / `src/tier2/go/` / `src/tier3/bff/` / `src/sdk/go/` の 4 module を併存させる。VS Code Go 拡張（gopls）はモジュール跨ぎ参照を `go.work` 経由で解決するため、リポジトリルートに `go.work` を配置する。

```go
// go.work（リポジトリルート）
go 1.22

use (
    ./src/tier1/go
    ./src/tier2/go
    ./src/tier3/bff
    ./src/sdk/go
)
```

`go.work` の運用ルール:

- **ローカル開発**: `go.work` が有効になり、`replace` directive と併せて local path 解決。IDE で module 跨ぎの "Go to Definition" が機能する
- **CI / 本番 build**: `GOWORK=off` 環境変数で `go.work` を無効化し、各 module が独立して `go build` される。`.github/workflows/*.yaml` の Go ジョブで `env: GOWORK: off` を明示。これにより CI は `go.work` の存在に依存せず、各 module の `go.mod` のみで再現性のあるビルドを行う
- **Docker build**: Dockerfile 内で `GOWORK=off` を明示（`ENV GOWORK=off`）。build context が単一 module に閉じるため `go.work` があっても読めないが、安全側に明示する
- **`go.work.sum`**: commit する（`go.sum` と同じ扱い）。`go.work` 使用時の依存の reproducibility を担保

`go.work` は開発者体験の向上のみを目的とし、CI / 本番ビルドの依存関係には一切影響しない。`replace` と `go.work` の二系統で local 参照を張ることになるが、前者は publish 前の module 依存解決、後者は IDE の multi-module navigation という役割分担で矛盾しない。

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
# build-context 要件: docker build は src/tier2/go/ をルートとして実行する前提で書いている。
# CI は `docker build -f src/tier2/go/services/stock-reconciler/Dockerfile src/tier2/go/` を使う。
# リポジトリルートから build すると go.mod の位置が合わず、`COPY go.mod` が失敗する。
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

## shared/ の位置付けと可視性強制

`src/tier2/go/shared/` は tier2 Go サービス間で共有するユーティリティ。

- `dapr/`: Dapr SDK のラッパー（ログ出力や metrics 取得のフックを共通化）
- `otel/`: OpenTelemetry 初期化ボイラープレート
- `errors/`: 共通エラー型（tier2 専用の E-T2-\* 体系）

shared は tier2 内部の利用のみ許容するが、`shared/` ディレクトリは Go の `internal/` 配下ではないため、Go コンパイラの internal 機構では外部 import を自動禁止できない。この差を認識した上で、以下の **3 層防御** で可視性を強制する。

1. **CODEOWNERS 上のレビュー境界**: `/src/tier2/go/shared/` を `@k1s0/tier2-dev` の所有とし、tier1 / tier3 / sdk の PR から参照が追加されたらブロック
2. **`go vet` カスタム lint**: `tools/ci/lint-import-boundaries.go` で `go/ast` を使い、`src/tier1/` や `src/tier3/` や `src/sdk/` 配下の `.go` が `github.com/k1s0/k1s0/src/tier2/go/shared/...` を import していれば fail。CI 必須 check
3. **依存方向テスト**: `tests/contract/import-graph_test.go` で全モジュールの import グラフを静的解析し、`CLAUDE.md` の依存方向と照合。違反があれば赤

なぜ `internal/shared/` 配下に移して Go 標準の internal 機構で強制しないか:

- tier2 の複数サービス（stock-reconciler / notification-hub 等）が `go.mod` を 1 本に統合しているため、`src/tier2/go/services/*/internal/shared/` のようなサービス配下の internal では共有できない
- `src/tier2/go/internal/shared/` に置くと、`src/tier2/go/services/*/` から見える（Go internal は親ディレクトリ木内を許容）ため一見成立するが、`src/tier2/` 配下に .NET services も同居し、.NET からの参照禁止は Go internal では制御できない（.NET ビルドは Go 無視）

以上から、仕組みで強制するのは import boundary lint とし、`shared/` のまま配置する方針を採る。lint は `go vet` と同格で CI 必須化する。

ただし `shared/` 配下は tier2 Go の内部 API と位置付け、外部への公開 API ではない。安定した API が必要な場合は SDK（`src/sdk/go/`）に昇格する。

### shared サブパッケージの安定度ラベル

`shared/` 内でも、サブパッケージごとに API 安定度が異なる。新規サブパッケージの追加 PR には以下のラベルを package doc comment の冒頭に明記する。

| ラベル | 意味 | 変更時のレビュー |
|---|---|---|
| `// Stability: Alpha` | 実装検証中。次 PR で shape が変わり得る | `@k1s0/tier2-dev` 1 名 |
| `// Stability: Beta` | 実装確定。wire / signature 変更には 1 週間の deprecation 通告 | `@k1s0/tier2-dev` 2 名 |
| `// Stability: Stable` | 破壊的変更禁止。breaking 時は SDK 昇格か新サブパッケージ | `@k1s0/tier2-dev` 2 名 + `@k1s0/arch-council` |

Phase 1a の初期サブパッケージ（`dapr/` / `otel/` / `errors/`）は Alpha 開始、Phase 1b で Beta、Phase 1c で Stable を目指す。Stable に到達した API は Phase 2 で `src/sdk/go/` への昇格を検討する。

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
