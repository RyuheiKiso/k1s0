// k1s0 Go SDK モジュール（リリース時点 最小、生成 stub のみ）
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
//     - 物理配置は src/sdk/go/ だが module 名は OSS 公開 path に揃える
//   docs/05_実装/20_コード生成設計/10_buf_Protobuf/01_buf_Protobuf生成パイプライン.md
//     - 生成先: src/sdk/go/proto/v1/
//
// 構成:
//   ./proto/v1/k1s0/tier1/<api>/v1/   ... buf 生成 stub（pb.go + grpc.pb.go）
//   高水準 facade（k1s0.State.Save 等の動詞統一 API）はリリース時点 対象外、
//   ロードマップ #8 で追加予定。
//
// 利用側からの import:
//   import statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
//
// tier1 / tier2 / tier3 の同リポジトリ Go module からは go.mod の
// `replace github.com/k1s0/sdk-go => ../../sdk/go` で参照する（リリース時点 削除）。
module github.com/k1s0/sdk-go

go 1.25.0

require (
	google.golang.org/genproto/googleapis/api v0.0.0-20260427160629-7cedc36a6bc4
	google.golang.org/grpc v1.79.3
	google.golang.org/protobuf v1.36.11
)

require (
	golang.org/x/net v0.48.0 // indirect
	golang.org/x/sys v0.39.0 // indirect
	golang.org/x/text v0.32.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260420184626-e10c466a9529 // indirect
)
