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

go 1.22

require google.golang.org/grpc v1.66.2

require (
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	google.golang.org/protobuf v1.34.1 // indirect
)
