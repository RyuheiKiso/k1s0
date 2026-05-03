// tests/e2e/owner/ha-dr/ha_dr_test.go
//
// owner suite ha-dr/ — HA / DR 経路の実機検証。
// 4 経路: 3CP HA 切替 / etcd snapshot 復旧 / CNPG barman-cloud 復旧 / Argo CD GitOps 完全再構築.
//
// 設計正典:
//   ADR-TEST-005（DR drill 4 経路）
//   ADR-TEST-008 §1 ha-dr 配置

//go:build owner_e2e

package hadr

import (
	"testing"
)

// TestHAControlPlaneFailover は control-plane 1 の kube-apiserver を停止し、
// 残 2 control-plane で API server が継続稼働することを検証する。
func TestHAControlPlaneFailover(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-005 経路 A)")
}

// TestEtcdSnapshotRecovery は etcd snapshot から cluster 状態を復旧する経路の検証
func TestEtcdSnapshotRecovery(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-005 経路 A)")
}

// TestCNPGBarmanCloudRecovery は CNPG cluster を barman-cloud から復旧する経路の検証
func TestCNPGBarmanCloudRecovery(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-005 経路 C)")
}

// TestGitOpsFullRebuild は Argo CD で cluster を完全再構築する経路の検証
func TestGitOpsFullRebuild(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-005 経路 B)")
}
