# tier1 Go ファサード（k1s0d）

stable Dapr Go SDK を直接叩く Go ファサード層。tier2 / tier3 / 外部 SDK 利用者が見るのは本層が露出する gRPC のみ。Dapr 型は `internal/dapr/` 配下に封じ込め、analyzer（plan フェーズ 06）と golangci-lint `forbidigo` の二重防御で漏洩を遮断する（ADR-TIER1-003）。

## 現状（リリース時点最小骨格）

- [x] `cmd/k1s0d`: gRPC server を `:50051` で listen、標準 `grpc.health.v1.Health` に応答、graceful shutdown 対応
- [ ] 11 API ハンドラ（plan 04-03 〜 04-13）
- [ ] Dapr / Temporal / Keycloak / OpenBao client wrapper（plan 04-02）
- [ ] OTel / retry / circuit-breaker（plan 04-02）
- [ ] 設定読込（YAML + envvar、plan 04-02）

## ディレクトリ責務

```text
src/tier1/go/
├── cmd/
│   └── k1s0d/main.go        # gRPC server エントリポイント（:50051）
├── internal/
│   ├── server/              # gRPC server 実装（11 API ハンドラ）※plan 04-03 〜 04-13
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

## ローカル起動

```bash
cd src/tier1/go
go build ./...
./k1s0d &
# 別シェルで疎通確認（要: grpcurl）
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
# → {"status": "SERVING"}
```

## ビルド / 検証

```bash
go build ./...
go test ./...
golangci-lint run ./...
```

## 関連設計

- [`plan/04_tier1_Goファサード実装/`](../../../plan/04_tier1_Goファサード実装/)
- [`docs/02_構想設計/02_tier1設計/`](../../../docs/02_構想設計/02_tier1設計/)
- ADR-TIER1-001（Dapr ファサード）/ ADR-TIER1-002 / ADR-TIER1-003（tier1 内部言語不可視）

## ポリシー

- **Dapr SDK 型は `internal/dapr/` 配下にしか出てこない**: handler 層（`internal/server/`）は contracts 型のみを扱う。
- **薄く保つ**: ビジネスロジックは tier2 にあるべき。ファサードは「型変換 + retry + 観測」だけ。
- **OTel 伝搬必須**: SDK → tier1 → Dapr → backend の全段で trace が繋がる。
- **SLO 予算 200ms（業務）+ 80ms（Dapr）+ 20ms（OTel）= 300ms**: ファサード層の overhead を測定可能に。
