# 03. Go module 配置

本ファイルは `src/tier1/go/` 配下の Go module レイアウトを確定する。DS-SW-COMP-124 の規定を物理配置レベルで展開し、ADR-DIR-001（contracts 昇格）による import path の影響を明示する。

## レイアウト

```
src/tier1/go/
├── README.md
├── go.mod                          # module github.com/k1s0/k1s0/src/tier1/go
├── go.sum
├── Dockerfile.state                # t1-state Pod 用
├── Dockerfile.secret               # t1-secret Pod 用
├── Dockerfile.workflow             # t1-workflow Pod 用
├── cmd/
│   ├── state/
│   │   └── main.go                 # t1-state 起動エントリ
│   ├── secret/
│   │   └── main.go                 # t1-secret 起動エントリ
│   └── workflow/
│       └── main.go                 # t1-workflow 起動エントリ
├── internal/
│   ├── adapter/
│   │   ├── dapr/                   # Dapr Go SDK ラッパー
│   │   ├── log/                    # Log Adapter（OpenTelemetry bridge）
│   │   └── metrics/                # Metrics Emitter
│   ├── common/                     # k1s0-common 相当（横断型・util）
│   ├── policy/                     # Policy Enforcer
│   ├── proto/
│   │   └── tier1/                  # buf generate で自動生成（proto package `k1s0.tier1.v1` / `k1s0.tier1.internal.v1` が階層を決める）
│   │       ├── v1/                 # 公開 tier1 API
│   │       │   ├── state.pb.go
│   │       │   ├── state_grpc.pb.go
│   │       │   ├── secrets.pb.go
│   │       │   ├── secrets_grpc.pb.go
│   │       │   ├── workflow.pb.go
│   │       │   ├── workflow_grpc.pb.go
│   │       │   └── ...（他 API 分）
│   │       └── internal/
│   │           └── v1/             # tier1 内部 API（SDK には漏れない、buf.gen.go.yaml の internal include_types で生成）
│   │               ├── common.pb.go
│   │               ├── errors.pb.go
│   │               └── pii.pb.go
│   ├── otel/                       # k1s0-otel 相当
│   ├── state/                      # COMP-T1-STATE 固有実装
│   ├── secret/                     # COMP-T1-SECRET 固有実装
│   └── workflow/                   # COMP-T1-WORKFLOW 固有実装
├── pkg/                            # Phase 1b 以降の public API（予約）
├── tests/
│   ├── integration/                # testcontainers 利用の統合テスト
│   └── fixtures/
└── scripts/
    ├── run-local.sh
    └── gen-mocks.sh
```

## 主要変更点（DS-SW-COMP-124 からの差分）

### 1. Dockerfile は Pod ごとに分離

DS-SW-COMP-124 原文では Dockerfile の位置が特定されていなかった。本配置では `Dockerfile.<pod>` 形式で 3 つ分 `src/tier1/go/` 直下に置く。理由は以下。

- `deploy/charts/tier1/` からの参照パスが単純化
- `docker build -f src/tier1/go/Dockerfile.state` で CI ジョブが明確
- multi-stage build の第 1 stage で `go build ./cmd/state/` と書ける

### 2. proto/tier1/ 階層の導入

ADR-DIR-001 の昇格により、contracts は tier1 公開 11 API（`src/contracts/tier1/v1/`、package `k1s0.tier1.v1`）と tier1 内部（`src/contracts/internal/v1/`、package `k1s0.tier1.internal.v1`）の 2 種類に分離した。Go 側は `paths=source_relative` で proto package 階層がそのまま出力階層になるため、生成先は `internal/proto/tier1/v1/`（公開）と `internal/proto/tier1/internal/v1/`（内部）に自動分離される。

### 3. tests/ と scripts/ の位置

tier 共通の tests/ は `/tests/`（リポジトリルート）に配置するが、tier1 Go 固有の integration / fixtures は `src/tier1/go/tests/` に置く。これは Go のテスト慣習（`_test.go` が各 package 配下）と整合しつつ、パッケージ外の fixture を保持する場所として必要。

`scripts/` は `src/tier1/go/` 固有のスクリプト（ローカル起動・モック生成）を置く。横断ツールは `/tools/` に、横断運用は `/ops/` に行く。

## go.mod の推奨サンプル

```go
module github.com/k1s0/k1s0/src/tier1/go

go 1.22

require (
    github.com/dapr/go-sdk v1.11.0
    github.com/dapr/kit v0.13.0
    google.golang.org/grpc v1.66.0
    google.golang.org/protobuf v1.34.2
    go.opentelemetry.io/otel v1.30.0
    go.opentelemetry.io/otel/trace v1.30.0
    github.com/prometheus/client_golang v1.20.4
    github.com/spf13/viper v1.19.0
)

require (
    // indirect deps は go mod tidy で自動管理
)
```

バージョンは Phase 1a 着手時点で最新安定版に合わせる。

## cmd/<pod>/main.go の構造

DS-SW-COMP-125 で規定された通り、各 Pod の `main.go` は最小限の起動コードのみ含む。典型的な構造は以下。

```go
// cmd/state/main.go
//
// t1-state Pod のエントリポイント
package main

// 設定ロード / OTel 初期化 / Policy 初期化 / Dapr Adapter 初期化 / API Router 起動 / graceful shutdown
import (
    "context"
    "os/signal"
    "syscall"

    "github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
    "github.com/k1s0/k1s0/src/tier1/go/internal/common"
    "github.com/k1s0/k1s0/src/tier1/go/internal/otel"
    "github.com/k1s0/k1s0/src/tier1/go/internal/policy"
    "github.com/k1s0/k1s0/src/tier1/go/internal/state"
)

func main() {
    // 設定をロード
    cfg := common.LoadConfig()

    // OpenTelemetry 初期化
    shutdownOTel := otel.Init(cfg.OTel)
    defer shutdownOTel()

    // Policy Enforcer 初期化
    policyEnforcer := policy.NewEnforcer(cfg.Policy)

    // Dapr Adapter 初期化
    daprClient := dapr.NewClient(cfg.Dapr)

    // state Pod のロジック起動
    svc := state.NewService(daprClient, policyEnforcer)

    // graceful shutdown 対応
    ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
    defer stop()

    svc.Run(ctx)
}
```

`main.go` は 100 行以内（DS-SW-COMP-125）。ロジックは `internal/state/` 等に委譲。

## internal/proto/v1/ の生成フロー

buf generate 実行時の流れ。

1. `src/contracts/` ディレクトリで `buf generate` を実行
2. `buf.gen.yaml` の Go plugin が `src/contracts/tier1/v1/*.proto` と `src/contracts/internal/v1/*.proto` を入力として取り込む
3. 出力先 `../tier1/go/internal/proto/` に `.pb.go` / `_grpc.pb.go` を生成
4. 生成物は git commit（DS-SW-COMP-122）

CI で `buf generate` 実行後に `git diff --exit-code internal/proto/` を実行し、drift 検出を自動化する。

Go module 分離戦略（IMP-BUILD-GM-026 により go.work は不採用）の詳細は [../30_tier2レイアウト/03_go_services配置.md](../30_tier2レイアウト/03_go_services配置.md) の「Go module 戦略」を参照。

## 依存方向の強制

`tools/ci/go-dep-check/` で以下を検証する。

- `internal/` 配下の .go ファイルは `github.com/k1s0/k1s0/src/tier2/` や `src/tier3/` を import してはいけない
- `cmd/` 配下の .go ファイルは `internal/` のみを import（他 Pod の internal 参照禁止、Go の internal 機構で自動強制）
- `pkg/` 配下（Phase 1b 以降）の公開 API は破壊的変更時に ADR 起票

## Container image

DS-SW-COMP-128 に従い、multi-stage build。

```dockerfile
# Dockerfile.state
FROM golang:1.22 AS builder
WORKDIR /workspace
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -trimpath -o bin/t1-state ./cmd/state/

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /workspace/bin/t1-state /usr/local/bin/t1-state
USER nonroot
EXPOSE 50001 9090
ENTRYPOINT ["/usr/local/bin/t1-state"]
```

## テスト戦略

- **unit test**: 各 package 配下の `_test.go`（`go test -race ./...`）
- **integration test**: `tests/integration/` で testcontainers を使い Dapr + Redis + Kafka を起動
- **coverage**: `go test -cover ./...`、目標 80%（Phase 1c で達成）

## スパースチェックアウト cone

- `tier1-go-dev` cone に `src/tier1/go/` を含む
- `tier1-rust-dev` cone には含まない

## 対応 IMP-DIR ID

- IMP-DIR-T1-023（src/tier1/go/ 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-001 / ADR-DIR-001
- DS-SW-COMP-124 / DS-SW-COMP-125 / DS-SW-COMP-126 / DS-SW-COMP-127 / DS-SW-COMP-128
- FR-STATE / FR-SECRET / FR-WORKFLOW 系 / DX-CICD-\*
