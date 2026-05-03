// tests/e2e/owner/helpers/version_planner.go
//
// upgrade/ で使う K8s バージョン計画 helper。kubeadm upgrade plan の実行 +
// upgrade 可能 version の取得 + control-plane / worker の段階適用順序を提供。
//
// 設計正典:
//   ADR-TEST-005（Upgrade drill）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
//
// リリース時点は skeleton 配置のみ。real 実装は採用初期で multipass exec 経由の kubeadm 連携.
package helpers

import (
	"context"
	"fmt"
)

// VersionPlanner は kubeadm upgrade plan の wrapper
type VersionPlanner struct {
	// CP1VM は control-plane 1 の VM 名（upgrade plan 実行 host）
	CP1VM string
}

// NewVersionPlanner は CP1 VM 名を受け取って VersionPlanner を生成
func NewVersionPlanner(cp1VM string) *VersionPlanner {
	return &VersionPlanner{CP1VM: cp1VM}
}

// AvailableVersions は kubeadm upgrade plan で取得した次期 version 一覧を返す。
// 採用初期で multipass exec ${CP1VM} -- sudo kubeadm upgrade plan の解析を実装する。
func (p *VersionPlanner) AvailableVersions(_ context.Context) ([]string, error) {
	return nil, fmt.Errorf("AvailableVersions 未実装 (CP1=%s、ADR-TEST-005 採用初期)", p.CP1VM)
}

// UpgradeControlPlane は control-plane node の kubeadm upgrade を実行する。
// 採用初期で multipass exec ${nodeName} -- sudo kubeadm upgrade {apply|node} を実装する。
func (p *VersionPlanner) UpgradeControlPlane(_ context.Context, nodeName, targetVersion string) error {
	return fmt.Errorf("UpgradeControlPlane 未実装 (node=%s, ver=%s、ADR-TEST-005 採用初期)", nodeName, targetVersion)
}
