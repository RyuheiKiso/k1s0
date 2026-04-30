// k1s0 Go fuzz テスト（Go 1.18+ 標準 fuzzing）。
//
// 目的:
//   tier1 facade の HTTP/JSON gateway / proto decoder が任意 byte 列で
//   panic / OOM / 無限ループしないことを保証する。tier1 facade は外部から
//   信頼境界外の JSON を受けるため、decoder の crash は SEV1 直結。

module github.com/k1s0/k1s0/tests/fuzz/go

go 1.25.0

require (
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	google.golang.org/protobuf v1.36.11
)

require (
	golang.org/x/net v0.48.0 // indirect
	golang.org/x/sys v0.39.0 // indirect
	golang.org/x/text v0.32.0 // indirect
	google.golang.org/genproto/googleapis/api v0.0.0-20260427160629-7cedc36a6bc4 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20260420184626-e10c466a9529 // indirect
	google.golang.org/grpc v1.79.3 // indirect
)

// SDK Go を local path から参照する（tier2 / tier3 BFF と同パターン）。
replace github.com/k1s0/sdk-go => ../../../src/sdk/go
