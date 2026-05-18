// buf-plugin-k1s0 — k1s0 tier1 Buf custom lint plugin モジュール定義
// tier1/CLAUDE.md §必須 annotation「全 RPC method に conformance_class / auth_class / slo_class /
// quota_class / observability.signal_class の annotation が必須」を Buf plugin で enforce する。
// buf.build/go/protoplugin は外部リポジトリのため go.sum で固定する（go mod tidy で解決済み）。
module k1s0.dev/tools/buf-plugin-k1s0

// Go バージョン: 1.23 を使用する
go 1.23.0

// google.golang.org/protobuf: Protocol Buffers Go API に使用する（plugin.go の protoreflect に使用する）
require google.golang.org/protobuf v1.36.4
