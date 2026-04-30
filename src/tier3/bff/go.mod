// tier3 BFF (Backend For Frontend) Go module。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/01_tier3全体配置.md
//
// scope:
//   portal-bff / admin-bff の 2 アプリが本 module を共有する。
//   tier3 web / native は本 BFF の HTTP / gRPC / GraphQL を呼び、内部 Go パッケージは参照しない。
//
// 依存方向:
//   src/sdk/go/ を経由して tier1 / tier2 にアクセスする。
//   tier1 / tier2 / contracts の internal package 直接参照は禁止。
module github.com/k1s0/k1s0/src/tier3/bff

go 1.25.0

require (
	github.com/go-jose/go-jose/v4 v4.1.4
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
)

require (
	golang.org/x/net v0.48.0 // indirect
	golang.org/x/sys v0.39.0 // indirect
	golang.org/x/text v0.32.0 // indirect
	google.golang.org/genproto/googleapis/api v0.0.0-20260427160629-7cedc36a6bc4 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260420184626-e10c466a9529 // indirect
	google.golang.org/grpc v1.79.3 // indirect
	google.golang.org/protobuf v1.36.11 // indirect
)

// docs 正典: docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/01_tier3全体配置.md
// 同一リポジトリ内で SDK Go を参照するため replace directive を使う。
// SDK が外部 registry に publish されたら本 directive は削除する（リリース時点 削除）。
replace github.com/k1s0/sdk-go => ../../sdk/go
