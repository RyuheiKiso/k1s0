// k1s0 統合テスト Go module（独立、testcontainers 利用）。
module github.com/k1s0/k1s0/tests/integration/go

go 1.24.0

// sdk-go は monorepo の同梱 module を使う。OSS 公開 module 名 (github.com/k1s0/sdk-go) を
// ローカルの src/sdk/go に向けて、binary level streaming gRPC テストから直接型を利用する。
replace github.com/k1s0/sdk-go => ../../../src/sdk/go

require (
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	google.golang.org/grpc v1.80.0
)

require (
	golang.org/x/net v0.49.0 // indirect
	golang.org/x/sys v0.40.0 // indirect
	golang.org/x/text v0.33.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260120221211-b8f7ae30c516 // indirect
	google.golang.org/protobuf v1.36.11 // indirect
)
