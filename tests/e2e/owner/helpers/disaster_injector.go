// tests/e2e/owner/helpers/disaster_injector.go
//
// ha-dr/ で使う障害注入 helper。control-plane kill / etcd snapshot 復旧 /
// CNPG barman-cloud 復旧 / Argo CD GitOps 完全再構築の 4 経路を提供する。
//
// 設計正典:
//   ADR-TEST-005（Upgrade drill + DR drill）
//   ADR-TEST-008（owner suite ha-dr 部位）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
//
// リリース時点は skeleton 配置のみ。real 実装は採用初期 (ADR-TEST-005 段階展開).
package helpers

import (
	"context"
	"fmt"
)

// DisasterInjector は ha-dr 試験で cluster に障害を注入する helper
type DisasterInjector struct {
	// K8sClient は k8s API 操作用（Pod delete / ConfigMap 改ざん 等）
	K8sClient *K8sClient
}

// NewDisasterInjector は K8sClient を受け取って DisasterInjector を生成
func NewDisasterInjector(client *K8sClient) *DisasterInjector {
	return &DisasterInjector{K8sClient: client}
}

// KillControlPlane は指定 control-plane node の kube-apiserver Pod を強制削除する。
// HA fail-over の検証で使う。リリース時点は skeleton (multipass exec で kubelet stop が必要).
func (d *DisasterInjector) KillControlPlane(_ context.Context, nodeName string) error {
	// 採用初期で multipass exec ${nodeName} -- sudo systemctl stop kubelet を実装
	return fmt.Errorf("KillControlPlane 未実装 (node=%s、ADR-TEST-005 採用初期)", nodeName)
}

// RestoreEtcdSnapshot は etcd snapshot から復旧する。
// 検証 ha-dr/etcd_snapshot_recovery_test.go で使う。
func (d *DisasterInjector) RestoreEtcdSnapshot(_ context.Context, snapshotPath string) error {
	// 採用初期で etcdctl snapshot restore + kube-apiserver restart を実装
	return fmt.Errorf("RestoreEtcdSnapshot 未実装 (path=%s、ADR-TEST-005 採用初期)", snapshotPath)
}

// RestoreCNPGFromBarman は barman-cloud から CNPG cluster を復旧する。
// 検証 ha-dr/cnpg_barman_recovery_test.go で使う。
func (d *DisasterInjector) RestoreCNPGFromBarman(_ context.Context, clusterName string) error {
	// 採用初期で CNPG Cluster CRD の bootstrap.recovery.source 経由復旧を実装
	return fmt.Errorf("RestoreCNPGFromBarman 未実装 (cluster=%s、ADR-TEST-005 採用初期)", clusterName)
}
