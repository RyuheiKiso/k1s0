// tests/e2e/owner/observability/trace_propagation_test.go
//
// 観測性 E2E 検証 1: OTLP trace 貫通
// tier1→2→3 を貫通する trace_id が OTel Collector → Tempo まで往復するかを検証する。
//
// 設計正典:
//   ADR-TEST-009 §1 検証 1
//   ADR-OBS-002（OTel Collector）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/04_観測性5検証.md
//
// 採用初期で real 化対象（検証 1 / 5 が先行 phase）。

//go:build owner_e2e

package observability

import (
	"testing"
)

// TestOTLPTracePropagation は trace_id を生成して tier3-bff → tier2-service →
// tier1-state を呼び、Tempo HTTP API で span tree を取得して service name 集合を assert する。
func TestOTLPTracePropagation(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-009 §1 検証 1)")
}
