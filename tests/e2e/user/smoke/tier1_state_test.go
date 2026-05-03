// tests/e2e/user/smoke/tier1_state_test.go
//
// State.Set → State.Get round-trip の最小検証。利用者が SDK で
// k1s0 を呼べる経路の sanity check。real 実装は採用初期で test-fixtures 経由。
//
// 設計正典: ADR-TEST-008 §1 / ADR-TEST-010 §3 領域 2

//go:build user_e2e

package smoke

import (
	"testing"
)

// TestTier1StateRoundtrip は SDK Go client で State.Set → State.Get round-trip を検証する。
// real 実装は採用初期で src/sdk/go/k1s0/test-fixtures.Setup + .NewSDKClient 経由で行う。
func TestTier1StateRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (test-fixtures Setup + State.Set/Get)")
}
