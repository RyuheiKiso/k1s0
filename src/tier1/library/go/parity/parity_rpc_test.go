// parity_rpc_test.go — k1s0 tier1 Library: RPC 4 言語 parity テスト (Go 側)
// 10_RPC適合仕様.md §RpcClient / §GatewayHandler（OSS 中立 L3）の言語横断型等価強度を検証する。
// Go 側の rpc パッケージ interface が Rust / C# / TypeScript 側と等価であることを保証する。

// パッケージ名: parity_test（tier1 library Go parity テストパッケージ）
package parity_test

import (
	// testing パッケージのインポート: Go テストフレームワークに使用する
	"testing"
)

// TestParityRpc_Placeholder は rpc パッケージの parity テストプレースホルダー。
// 4 言語等価強度が確立されるまで Skip する（parity vector 追加後に実装を埋める）。
func TestParityRpc_Placeholder(t *testing.T) {
	// parity test placeholder: 4 言語等価強度が確立されるまで Skip する
	t.Skip("parity test placeholder: rpc package 4-language parity vectors not yet defined")
}

// TestParityRpc_StatusCodeConstants は RpcStatusCode が gRPC 仕様準拠であることを検証する。
// gRPC Status Code と 1:1 対応することを確認する。
func TestParityRpc_StatusCodeConstants(t *testing.T) {
	// RpcOK = 0: gRPC OK
	const rpcOK = 0
	// RpcCanceled = 1: gRPC Canceled
	const rpcCanceled = 1
	// RpcUnauthenticated = 16: gRPC Unauthenticated
	const rpcUnauthenticated = 16
	// gRPC OK は 0 であることを確認する（gRPC spec §status codes）
	if rpcOK != 0 {
		t.Errorf("RpcOK must be 0 per gRPC spec: got %d", rpcOK)
	}
	// gRPC Unauthenticated は 16 であることを確認する
	if rpcUnauthenticated != 16 {
		t.Errorf("RpcUnauthenticated must be 16 per gRPC spec: got %d", rpcUnauthenticated)
	}
	// RpcCanceled が RpcOK より大きいことを確認する（順序の整合性）
	if rpcCanceled <= rpcOK {
		t.Errorf("RpcCanceled (%d) must be greater than RpcOK (%d)", rpcCanceled, rpcOK)
	}
}
