// tests/e2e/user/smoke/tier1_audit_test.go
//
// Audit.Record → Audit.VerifyChain round-trip の最小検証。
//
// 設計正典: ADR-TEST-008 §1

//go:build user_e2e

package smoke

import (
	"testing"
)

// TestTier1AuditRoundtrip は SDK Go client で Audit.Record × N → VerifyChain を検証
func TestTier1AuditRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (Audit.Record + VerifyChain)")
}
