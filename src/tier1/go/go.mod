// k1s0 tier1 Go ファサード（plan 04-01 / IMP-DIR-002 / IMP-BUILD-002）
//
// 役割:
//   stable Dapr Go SDK を直接叩く Go ファサード層。tier2/tier3 は本層が露出する gRPC のみを使う
//   （ADR-TIER1-003: tier1 内部言語は不可視）。
//
// scope（リリース時点）:
//   plan 04-01 完了条件の最小骨格として `cmd/k1s0d` の gRPC server 起動 + 標準 gRPC health
//   protocol（grpc.health.v1.Health/Check）応答までを提供。Dapr / Temporal / Keycloak / OpenBao
//   の各 client wrapper、12 API ハンドラの実装は plan 04-02 〜 04-13 で順次追加する。
//
// module path: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md 正典に準拠（monorepo path-style）
module github.com/k1s0/k1s0/src/tier1/go

go 1.22.6

toolchain go1.23.4

require (
	github.com/dapr/go-sdk v1.11.0
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	google.golang.org/grpc v1.66.2
	google.golang.org/protobuf v1.34.2
)

require (
	github.com/dapr/dapr v1.14.0 // indirect
	github.com/google/uuid v1.6.0 // indirect
	github.com/kr/pretty v0.3.1 // indirect
	go.opentelemetry.io/otel v1.27.0 // indirect
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240624140628-dc46fd24d27d // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

// docs 正典: docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
// 同一リポジトリ内で SDK Go を参照するため replace directive を使う。
// SDK が外部 registry に publish されたら本 directive は削除する（リリース時点 削除）。
replace github.com/k1s0/sdk-go => ../../sdk/go
