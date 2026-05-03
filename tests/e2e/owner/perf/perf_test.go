// tests/e2e/owner/perf/perf_test.go
//
// owner suite perf/ — k6 spawn を Go test がラップした性能試験。
// API latency / throughput / p99 SLO 違反検出を確認する。
//
// 設計正典:
//   ADR-TEST-008 §1 perf 配置
//   DS-DEVX-TEST-007（k6）

//go:build owner_e2e

package perf

import (
	"testing"
)

// TestPerfTier1StateLatency は k6 で tier1-state の State.Get/Set 連続呼び出しを
// 5 分間流し、p99 latency が SLO（200ms）以下であることを検証する。
func TestPerfTier1StateLatency(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 perf)")
}

// TestPerfTier1Throughput は tier1-state の同時並列接続 1000 で sustained throughput を測定
func TestPerfTier1Throughput(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 perf)")
}
