// k1s0 examples/tier3-bff-graphql — Golden Path tier3 BFF GraphQL 最小例
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md
//       docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_bff配置.md
// 関連 ID: ADR-TIER3-001 / ADR-DEV-001
module github.com/k1s0/k1s0/examples/tier3-bff-graphql

go 1.22

require github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000

require (
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	google.golang.org/grpc v1.66.2 // indirect
	google.golang.org/protobuf v1.34.2 // indirect
)

replace github.com/k1s0/sdk-go => ../../src/sdk/go
