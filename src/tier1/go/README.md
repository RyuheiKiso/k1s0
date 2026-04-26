# tier1 Go ファサード（3 Pod 構成）

stable Dapr Go SDK を直接叩く Go ファサード層。tier2 / tier3 / 外部 SDK 利用者が見るのは本層が露出する gRPC のみ。Dapr 型は `internal/adapter/dapr/` 配下に封じ込め、analyzer（plan フェーズ 06）と golangci-lint `forbidigo` の二重防御で漏洩を遮断する（ADR-TIER1-003）。

**module path**: `github.com/k1s0/k1s0/src/tier1/go`（monorepo path-style、`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md` 正典）

**listen port**: `:50001`（gRPC、docs 正典 EXPOSE 50001 / Dapr `dapr.io/app-port=50001`）

## Pod 構成（docs 正典）

`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md` および `docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md`（DS-SW-COMP-023 / 129–134）準拠。tier1 全体は **6 Pod（Go 3 + Rust 3）で 12 API** を担当する。本ディレクトリは Go 3 Pod 部分（**7 API**）の実装。

| Pod 名 | ディレクトリ | 担当 API | 主な依存 |
|---|---|---|---|
| **t1-state**（5 API 統合 Router） | `cmd/state/` | **ServiceInvoke / State / PubSub / Binding / Feature** | Dapr State / PubSub / Bindings + Feature Configuration backend |
| **t1-secret** | `cmd/secret/` | **Secrets** | Dapr Secrets + OpenBao |
| **t1-workflow** | `cmd/workflow/` | **Workflow** | Dapr Workflow + Temporal（振り分け） |

各 Pod は単一 `go.mod` 内でビルドされ、`go build ./cmd/<pod>` でバイナリが個別に出力される。Dockerfile も Pod ごとに分離（`Dockerfile.state` / `.secret` / `.workflow`、`src/tier1/go/` 直下、plan 04 後段で配置、EXPOSE 50001 + 9090）。

### tier1 全体（参考）

| Pod 名 | 言語 | 担当 API |
|---|---|---|
| t1-state | Go | ServiceInvoke / State / PubSub / Binding / Feature |
| t1-secret | Go | Secrets |
| t1-workflow | Go | Workflow |
| t1-decision | Rust | Decision / Log / Telemetry |
| t1-audit | Rust | Audit |
| t1-pii | Rust | PII |

> Health API は docs 正典で明記なし（標準 `grpc.health.v1.Health` 準拠が default 推定、k1s0 独自 HealthService の Pod 配置は次セッション確認事項）。

## 現状（リリース時点最小骨格）

- [x] 3 Pod の `cmd/{state,secret,workflow}/main.go` 配置
- [x] 共通ランタイム `internal/common/` に gRPC server bootstrap を集約（docs 正典: `internal/{common,adapter,policy,...}/` 責務分割）
- [x] 標準 `grpc.health.v1.Health/Check` 応答（3 Pod 共通）
- [x] gRPC reflection（dev/staging で grpcurl 疎通用、production は config で無効化予定）
- [x] SIGINT / SIGTERM で graceful shutdown（25s timeout）
- [x] listen port `:50001`（docs 正典 EXPOSE 50001）
- [x] 設定読込スケルトン（plan 04-02 / `internal/common/config.go`、envvar `K1S0_LISTEN_ADDR` のみ対応）
- [x] OTel 初期化スケルトン（plan 04-02 / `internal/otel/otel.go`、Init/Shutdown シグネチャのみ）
- [x] retry 戦略デフォルト定義（plan 04-02 / `internal/common/retry.go`、3 回 / 100-200-400ms / ±50% jitter）
- [x] timeout 階層定数 + helper（plan 04-02 / `internal/common/timeout.go`、500ms/200ms/100ms 階層）
- [x] Pod 別 Dockerfile（`Dockerfile.state` / `.secret` / `.workflow` 直下、multi-stage / distroless / nonroot / EXPOSE 50001+9090）
- [x] `.dockerignore`（テスト / README / .git / IDE 一時ファイル除外）
- [ ] 12 API ハンドラ（plan 04-03 〜 04-13、Pod 単位で実装）
- [ ] Dapr / Temporal / Keycloak / OpenBao client wrapper（plan 04-02 / `internal/adapter/`）
- [ ] OTel SDK 接続実装（plan 04-02 主作業 1〜4、TracerProvider / MeterProvider / W3C Propagator）
- [ ] retry / backoff の実コード実装（`Do[T any](ctx, cfg, fn)` ジェネリック helper）
- [ ] circuit-breaker（sony/gobreaker、配置先確定後）
- [ ] gRPC interceptor（plan 04-02 主作業 8、配置先 docs 明記なし）
- [ ] YAML 設定 + defaults + OpenBao 注入（plan 04-02 主作業 6）

## ディレクトリ責務

```text
src/tier1/go/
├── cmd/
│   ├── state/main.go        # t1-state Pod エントリ（5 API: ServiceInvoke + State + PubSub + Binding + Feature）
│   ├── secret/main.go       # t1-secret Pod エントリ（Secrets）
│   └── workflow/main.go     # t1-workflow Pod エントリ（Workflow）
├── internal/
│   ├── common/              # 共通 gRPC bootstrap + 設定 + reliability (DS-SW-COMP-108、k1s0-common 共通ライブラリ)
│   │   ├── runtime.go         # gRPC server bootstrap（health / reflection / graceful shutdown）
│   │   ├── config.go          # 設定 loader（envvar / YAML、tenant_id 抽出は次セッション）
│   │   ├── retry.go           # retry 戦略 RetryConfig + DefaultRetry（実 helper は次セッション）
│   │   └── timeout.go         # timeout 階層定数 + WithFacadeTimeout / WithDaprTimeout / WithRustCoreTimeout
│   ├── otel/                # OpenTelemetry 初期化 (DS-SW-COMP-109、k1s0-otel 共通ライブラリ)
│   │   └── otel.go            # tracer / meter / logger / propagator 集約（実装は次セッション）
│   ├── adapter/             # 外部システムのアダプタ（Dapr / Temporal / Keycloak / OpenBao） ※plan 04-02 以降
│   ├── policy/              # tier1 横断ポリシー（JWT / Tenant / OPA / Rate Limit / 冪等性、DS-SW-COMP-110） ※plan 04-17 〜 04-21
│   ├── proto/               # buf 生成物（公開 + 内部、`buf.gen.internal.yaml` 出力先）
│   ├── state/               # t1-state Pod 専用ロジック（plan 04-04 / 04-05 / 04-10 / 04-11 / 04-12）
│   ├── secret/              # t1-secret Pod 専用ロジック（plan 04-06）
│   └── workflow/            # t1-workflow Pod 専用ロジック（plan 04-07 / 04-14）
└── pkg/                     # 外部 import 想定の公開 API（基本空、必要時のみ）
```

`internal/proto/` の使い分け:

- 公開 12 API（`src/contracts/tier1/`）の Go 生成物 + tier1 内部 proto（`src/contracts/internal/`）の Go 生成物が **同じ `internal/proto/` 配下** に分離して出力される（docs 正典: `internal/proto/tier1/v1/`（公開）+ `internal/proto/tier1/internal/v1/`（内部））。
- ADR-TIER1-003 の隔離は SDK 配下に出さない事で実現（公開 SDK は `src/sdk/go/generated/` のみ）。

## ローカル起動 / 疎通確認

```bash
cd src/tier1/go
go build -o /tmp/t1-state ./cmd/state
go build -o /tmp/t1-secret ./cmd/secret
go build -o /tmp/t1-workflow ./cmd/workflow

# 3 Pod 同時起動はポート競合するため、開発時は flag でずらす（k8s では Pod 分離済 / 全 Pod :50001）
/tmp/t1-state    -listen :50001 &
/tmp/t1-secret   -listen :50002 &
/tmp/t1-workflow -listen :50003 &

# 疎通確認（要: grpcurl）
for p in 50001 50002 50003; do
  grpcurl -plaintext localhost:$p grpc.health.v1.Health/Check
done
# → 各 Pod が {"status": "SERVING"} を返す
```

## Docker build

各 Pod の image は `Dockerfile.<pod>` でビルドする（multi-stage、distroless/static-debian12:nonroot、CGO_ENABLED=0、-trimpath、EXPOSE 50001+9090）。

```bash
# build context は src/tier1/go/、各 Dockerfile を -f で指定
docker build -f Dockerfile.state    -t k1s0/t1-state:dev    src/tier1/go/
docker build -f Dockerfile.secret   -t k1s0/t1-secret:dev   src/tier1/go/
docker build -f Dockerfile.workflow -t k1s0/t1-workflow:dev src/tier1/go/

# 動作確認（host network 不可、port-forward で確認）
docker run --rm -p 50001:50001 k1s0/t1-state:dev
# 別シェル: grpcurl -plaintext localhost:50001 grpc.health.v1.Health/Check
```

## ビルド / 検証

```bash
go build ./...
go test ./...
go vet ./...
golangci-lint run ./...
```

## 関連設計

- [`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md`](../../../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md)（**正典**）
- [`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md`](../../../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md)（module + EXPOSE + Dockerfile 正典）
- [`docs/02_構想設計/02_tier1設計/`](../../../docs/02_構想設計/02_tier1設計/)
- ADR-TIER1-001（Dapr ファサード）/ ADR-TIER1-002 / ADR-TIER1-003（tier1 内部言語不可視）

## ポリシー

- **Dapr SDK 型は `internal/adapter/dapr/` 配下にしか出てこない**: handler 層は contracts 型のみを扱う。
- **薄く保つ**: ビジネスロジックは tier2 にあるべき。ファサードは「型変換 + retry + 観測」だけ。
- **OTel 伝搬必須**: SDK → tier1 → Dapr → backend の全段で trace が繋がる。
- **SLO 予算 200ms（業務）+ 80ms（Dapr）+ 20ms（OTel）= 300ms**: ファサード層の overhead を測定可能に。
- **cmd/ → internal/ のみ**: 他 Pod の internal 直接参照は禁止（依存方向の固定、Pod 専用 internal は他 Pod から不可視）。
- **go.work 不採用**: IMP-BUILD-GM-026。SDK は `replace` directive で local path 参照（運用蓄積後 publish）。
