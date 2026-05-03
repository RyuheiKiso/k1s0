// tests/e2e/owner/upgrade/upgrade_test.go
//
// owner suite upgrade/ — kubeadm N → N+1 minor version upgrade の実機検証。
// control-plane 各 5 分 / worker 各 3 分 / 全体 30 分以内が成功判定（ADR-TEST-005 §成功判定）。
//
// 設計正典:
//   ADR-TEST-005（Upgrade drill）
//   ADR-INFRA-001（kubeadm N + N+1 経路）

//go:build owner_e2e

package upgrade

import (
	"testing"
)

// TestKubeadmMinorUpgrade は cp-1 → cp-2 → cp-3 → w-1 → w-2 の順で
// kubeadm upgrade apply / upgrade node を実行し、
// 全 node が target version で Ready になることを検証する。
func TestKubeadmMinorUpgrade(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-005 Upgrade drill)")
}
