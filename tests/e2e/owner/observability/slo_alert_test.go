// tests/e2e/owner/observability/slo_alert_test.go
//
// 観測性 E2E 検証 4: SLO burn rate alert 発火 + runbook_url 必須
// 意図的な SLO 違反（k6 で latency 超過注入）で fast burn alert が 5 分窓内に発火し、
// alert label に runbook_url が含まれることを検証する。ADR-OPS-001 の必須要件継続検証。
//
// 設計正典:
//   ADR-TEST-009 §1 検証 4
//   ADR-OPS-001（Runbook 標準化、runbook_url 必須要件）

//go:build owner_e2e

package observability

import (
	"testing"
)

// TestSLOAlertBurnRateWithRunbook は k6 で SLO 違反流量を 6 分間注入し、
// Alertmanager の active alert に K1s0FastBurn が現れ、labels.runbook_url が
// non-empty + URL 形式であることを assert する。
func TestSLOAlertBurnRateWithRunbook(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-009 §1 検証 4)")
}
