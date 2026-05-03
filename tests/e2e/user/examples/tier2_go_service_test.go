// tests/e2e/user/examples/tier2_go_service_test.go
//
// examples/tier2-go-service/ の動作確認。利用者が tier2 Go アプリを書いて
// k1s0 SDK 越しに tier1 を呼べる経路の sanity check。
//
// 設計正典:
//   ADR-TEST-008 §1 user examples 配置
//   examples/tier2-go-service/

//go:build user_e2e

package examples

import (
	"testing"
)

// TestTier2GoServiceRoundtrip は example を起動 → SDK で /health を叩く → 期待 response 確認
func TestTier2GoServiceRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (test-fixtures + ExampleRunner)")
}
