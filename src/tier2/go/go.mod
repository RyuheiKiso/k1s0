// tier2 Go 全サービス共通の Go module。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/06_依存管理.md
//   docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
//
// scope:
//   tier2 Go 全サービス（services/stock-reconciler/ / services/notification-hub/）が
//   本 module を共有する（IMP-DIR-T2-043）。tier1 Go の go.mod とは独立に運用する。
//
// 依存方向:
//   tier2 Go は src/sdk/go/ を経由して tier1 にアクセスする。
//   tier1 / tier3 / contracts の直接 import は禁止（自作 import-boundary lint で強制）。
//
// SDK 参照:
//   リリース時点 では SDK が外部 registry に publish されていないため、replace directive で
//   local path（../../sdk/go）を参照する。SDK が publish されたら本 directive を削除する。
//   削除確認は tools/ci/lint-replace-directive.sh のリリースゲートで行う。
module github.com/k1s0/k1s0/src/tier2/go

go 1.22

require (
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	google.golang.org/grpc v1.66.2
)

require (
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	google.golang.org/protobuf v1.34.2 // indirect
)

// docs 正典: docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
// 同一リポジトリ内で SDK Go を参照するため replace directive を使う。
// SDK が外部 registry に publish されたら本 directive は削除する（リリース時点 削除）。
replace github.com/k1s0/sdk-go => ../../sdk/go
