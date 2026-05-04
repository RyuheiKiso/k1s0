// k1s0 examples/tier3-bff-graphql — Golden Path tier3 BFF GraphQL 最小例
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md
//       docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_bff配置.md
// 関連 ID: ADR-TIER3-001 / ADR-DEV-001
module github.com/k1s0/k1s0/examples/tier3-bff-graphql

// 依存先 src/sdk/go が go 1.25.0 を要求するため、本 module も同等以上に揃える
// （CI の Go 1.22 toolchain は GOTOOLCHAIN=auto で 1.25 を自動 download する）。
go 1.25.0

require github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000

require (
	golang.org/x/net v0.48.0 // indirect
	golang.org/x/sys v0.39.0 // indirect
	golang.org/x/text v0.32.0 // indirect
	google.golang.org/genproto/googleapis/api v0.0.0-20260427160629-7cedc36a6bc4 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260420184626-e10c466a9529 // indirect
	google.golang.org/grpc v1.79.3 // indirect
	google.golang.org/protobuf v1.36.11 // indirect
)

replace github.com/k1s0/sdk-go => ../../src/sdk/go
