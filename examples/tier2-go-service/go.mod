// k1s0 examples/tier2-go-service — Golden Path tier2 Go サービス完動例
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md
//       docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
// 関連 ID: ADR-DEV-001（Paved Road）/ ADR-TIER1-002（Protobuf gRPC）
module github.com/k1s0/k1s0/examples/tier2-go-service

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

// 同一リポジトリ内の SDK Go を path 参照（リリース後 publish したら削除）
replace github.com/k1s0/sdk-go => ../../src/sdk/go
