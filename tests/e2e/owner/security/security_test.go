// tests/e2e/owner/security/security_test.go
//
// owner suite security/ — Kyverno deny / NetworkPolicy / SPIRE workload identity /
// Istio mTLS STRICT / Trivy CVE block の機械検証。
//
// 設計正典: ADR-TEST-008 §1 ディレクトリ配置 / ADR-CICD-003（Kyverno）

//go:build owner_e2e

package security

import (
	"testing"
)

// TestKyvernoBlocksNonCanonicalHelmRelease は kyverno policy
// block-non-canonical-helm-releases が allow-list 外の helm release を deny することを検証
func TestKyvernoBlocksNonCanonicalHelmRelease(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 security)")
}

// TestNetworkPolicyEnforcement は tier1 namespace 間の NetworkPolicy 強制を検証
func TestNetworkPolicyEnforcement(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 security)")
}

// TestIstioMTLSStrict は Istio Ambient PeerAuthentication mTLS STRICT 強制を検証
func TestIstioMTLSStrict(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 security)")
}

// TestSPIREWorkloadIdentity は SPIRE が tier1 / tier2 Pod に SVID を発行することを検証
func TestSPIREWorkloadIdentity(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用後の運用拡大時 (ADR-TEST-008 §1 security)")
}

// TestTrivyCVECriticalBlocked は Harbor が Trivy で Critical 検出された image の push を deny することを検証
func TestTrivyCVECriticalBlocked(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §1 security)")
}
