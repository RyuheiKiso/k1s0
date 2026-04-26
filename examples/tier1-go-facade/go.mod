// k1s0 examples/tier1-go-facade — Golden Path tier1 Go ファサード最小例
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
//       docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（monorepo path-style）
// 関連 ID: ADR-TIER1-001 / ADR-TIER1-002 / ADR-TIER1-003 / ADR-DEV-001
//
// scope（リリース時点）: gRPC server + 標準 health protocol + graceful shutdown のみ
// 採用初期で拡張: proto handler 登録 / Dapr SDK adapter / OTel interceptor / integration test
module github.com/k1s0/k1s0/examples/tier1-go-facade

go 1.22

require google.golang.org/grpc v1.66.2

require (
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	google.golang.org/protobuf v1.34.1 // indirect
)
