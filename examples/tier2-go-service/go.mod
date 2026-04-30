// k1s0 examples/tier2-go-service — Golden Path tier2 Go サービス完動例
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md
//       docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
// 関連 ID: ADR-DEV-001（Paved Road）/ ADR-TIER1-002（Protobuf gRPC）
module github.com/k1s0/k1s0/examples/tier2-go-service

go 1.25.0

require (
	github.com/k1s0/k1s0/src/tier2/go v0.0.0
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
)

require (
	github.com/go-jose/go-jose/v4 v4.1.4 // indirect
	golang.org/x/net v0.48.0 // indirect
	golang.org/x/sys v0.39.0 // indirect
	golang.org/x/text v0.32.0 // indirect
	google.golang.org/genproto/googleapis/api v0.0.0-20260427160629-7cedc36a6bc4 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260420184626-e10c466a9529 // indirect
	google.golang.org/grpc v1.79.3 // indirect
	google.golang.org/protobuf v1.36.11 // indirect
)

// 同一リポジトリ内の SDK Go を path 参照（リリース後 publish したら削除）
replace github.com/k1s0/sdk-go => ../../src/sdk/go

// tier2 共通 JWT 認証 middleware を path 参照（同 monorepo 内 module）
replace github.com/k1s0/k1s0/src/tier2/go => ../../src/tier2/go
