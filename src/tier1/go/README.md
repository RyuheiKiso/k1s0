# tier1 Go ファサード（3 Pod 構成）

stable Dapr Go SDK を直接叩く Go ファサード層。tier2 / tier3 / 外部 SDK 利用者が見るのは本層が露出する gRPC のみ。Dapr 型は `internal/dapr/` 配下に封じ込め、analyzer（plan フェーズ 06）と golangci-lint `forbidigo` の二重防御で漏洩を遮断する（ADR-TIER1-003）。

## Pod 構成（docs 正典）

`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md` 準拠で **3 Pod** に分離。

| Pod 名 | ディレクトリ | 担当 API | 主な依存 |
|---|---|---|---|
| **t1-state** | `cmd/state/` | StateService / PubSubService | Dapr State / PubSub building block |
| **t1-secret** | `cmd/secret/` | SecretsService / BindingService | OpenBao + Dapr Bindings |
| **t1-workflow** | `cmd/workflow/` | WorkflowService | Dapr Workflow + Temporal（振り分け） |

各 Pod は単一 `go.mod` 内でビルドされ、`go build ./cmd/<pod>` でバイナリが個別に出力される。Dockerfile も Pod ごとに分離（`Dockerfile.state` / `.secret` / `.workflow`）。

> **注**: tier1 公開 11 API のうち **log / decision / feature / telemetry / audit** は Rust 実装（`src/tier1/rust/`）。本ディレクトリは Go 担当 5 API（state / pubsub / secret / binding / workflow）+ serviceinvoke の対応箇所のみを扱う（serviceinvoke の Pod 配置は次セッションで確認）。

## 現状（リリース時点最小骨格）

- [x] 3 Pod の `cmd/{state,secret,workflow}/main.go` 配置
- [x] 共通ランタイム `internal/server/runtime/` に gRPC server bootstrap を集約
- [x] 標準 `grpc.health.v1.Health/Check` 応答（3 Pod 共通）
- [x] gRPC reflection（dev/staging で grpcurl 疎通用、production は config で無効化予定）
- [x] SIGINT / SIGTERM で graceful shutdown（25s timeout）
- [ ] 11 API ハンドラ（plan 04-03 〜 04-13、docs に応じて Pod 単位で実装）
- [ ] Dapr / Temporal / Keycloak / OpenBao client wrapper（plan 04-02）
- [ ] OTel / retry / circuit-breaker（plan 04-02）
- [ ] 設定読込（YAML + envvar、plan 04-02）

## ディレクトリ責務

```text
src/tier1/go/
├── cmd/
│   ├── state/main.go        # t1-state Pod エントリ（State + PubSub）
│   ├── secret/main.go       # t1-secret Pod エントリ（Secrets + Binding）
│   └── workflow/main.go     # t1-workflow Pod エントリ（Workflow）
├── internal/
│   ├── server/
│   │   └── runtime/         # 共通 gRPC server bootstrap（health / reflection / graceful shutdown）
│   ├── dapr/                # Dapr Go SDK ラッパ（型変換 + retry + OTel）※plan 04-02
│   ├── temporal/            # Temporal client ラッパ ※plan 04-07
│   ├── keycloak/            # Keycloak client ※plan 04-11
│   ├── openbao/             # OpenBao client ※plan 04-06
│   ├── core/                # Rust core への internal gRPC client ※plan 04-08 / 04-09
│   ├── observability/       # OTel trace / metrics / logger ※plan 04-02
│   ├── reliability/         # retry / circuit-breaker / timeout ※plan 04-02
│   ├── config/              # YAML + envvar 設定読込 ※plan 04-02
│   ├── grpc/                # buf 生成物（公開 11 API、tier1.v1）※plan 04-01 後段
│   └── proto/               # buf 生成物（tier1 内部 proto）※buf.gen.internal.yaml 出力先
└── pkg/                     # 外部 import 想定の公開 API（基本空、必要時のみ）
```

`internal/grpc/` と `internal/proto/` の使い分け:

- `internal/grpc/`: 公開 11 API（`src/contracts/tier1/`）の Go 生成物。tier1 facade が gRPC server として実装する API。
- `internal/proto/`: tier1 内部 proto（`src/contracts/internal/`）の Go 生成物。Rust core への委譲などに使う。`buf.gen.internal.yaml` で生成（ADR-TIER1-003 隔離）。

## ローカル起動 / 疎通確認

```bash
cd src/tier1/go
go build -o /tmp/t1-state ./cmd/state
go build -o /tmp/t1-secret ./cmd/secret
go build -o /tmp/t1-workflow ./cmd/workflow

# 3 Pod 同時起動はポート競合するため、開発時は flag でずらす（k8s では Pod 分離済）
/tmp/t1-state    -listen :50561 &
/tmp/t1-secret   -listen :50562 &
/tmp/t1-workflow -listen :50563 &

# 疎通確認（要: grpcurl）
for p in 50561 50562 50563; do
  grpcurl -plaintext localhost:$p grpc.health.v1.Health/Check
done
# → 各 Pod が {"status": "SERVING"} を返す
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
- [`docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md`](../../../docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md)
- [`docs/02_構想設計/02_tier1設計/`](../../../docs/02_構想設計/02_tier1設計/)
- ADR-TIER1-001（Dapr ファサード）/ ADR-TIER1-002 / ADR-TIER1-003（tier1 内部言語不可視）

## ポリシー

- **Dapr SDK 型は `internal/dapr/` 配下にしか出てこない**: handler 層は contracts 型のみを扱う。
- **薄く保つ**: ビジネスロジックは tier2 にあるべき。ファサードは「型変換 + retry + 観測」だけ。
- **OTel 伝搬必須**: SDK → tier1 → Dapr → backend の全段で trace が繋がる。
- **SLO 予算 200ms（業務）+ 80ms（Dapr）+ 20ms（OTel）= 300ms**: ファサード層の overhead を測定可能に。
- **cmd/ → internal/ のみ**: 他 Pod の internal 直接参照は禁止（依存方向の固定）。
