// k1s0 Go fuzz テスト（Go 1.18+ 標準 fuzzing）。
//
// 目的:
//   tier1 facade の HTTP/JSON gateway / proto decoder が任意 byte 列で
//   panic / OOM / 無限ループしないことを保証する。tier1 facade は外部から
//   信頼境界外の JSON を受けるため、decoder の crash は SEV1 直結。

module github.com/k1s0/k1s0/tests/fuzz/go

go 1.22

require (
	github.com/k1s0/sdk-go v0.0.0-00010101000000-000000000000
	google.golang.org/protobuf v1.34.2
)

require (
	golang.org/x/net v0.26.0 // indirect
	golang.org/x/sys v0.21.0 // indirect
	golang.org/x/text v0.16.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240604185151-ef581f913117 // indirect
	google.golang.org/grpc v1.66.2 // indirect
)

// SDK Go を local path から参照する（tier2 / tier3 BFF と同パターン）。
replace github.com/k1s0/sdk-go => ../../../src/sdk/go
