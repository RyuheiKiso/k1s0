// tests/e2e/owner/observability/prometheus_cardinality_test.go
//
// 観測性 E2E 検証 2: Prometheus cardinality regression
// tier1 / tier2 metric の cardinality が baseline の 1.2 倍を超えないことを検証する。
//
// 設計正典: ADR-TEST-009 §1 検証 2

//go:build owner_e2e

package observability

import (
	"testing"
)

// TestPrometheusCardinality は infra/observability/cardinality/baselines/<metric>.json と
// Prometheus /api/v1/series の比較で cardinality regression を検出する。
func TestPrometheusCardinality(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-009 §1 検証 2)")
}
