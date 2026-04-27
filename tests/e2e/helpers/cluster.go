// 本ファイルは E2E シナリオから利用する kind cluster 操作ヘルパの雛形。
// 採用初期 で helm / kubectl 経由の操作を実装し、シナリオ側から呼ぶ。
package helpers

import "testing"

// Cluster は E2E シナリオ実行用 kind cluster の handle。
// SetupCluster で kind cluster を起動し、Teardown で破棄する。
type Cluster struct {
	// kubeconfig へのパス（kind cluster 別 namespace の認証）
	Kubeconfig string
	// テスト中に作成された namespace（Teardown で削除）
	Namespaces []string
}

// SetupCluster は kind cluster を起動し infra/environments/dev/ を適用した
// Cluster handle を返す。実装は採用初期 で完成させる。
func SetupCluster(t *testing.T) *Cluster {
	t.Helper()
	t.Skip("TODO(release-initial): kind cluster + infra/environments/dev/ apply を実装する")
	return nil
}

// Teardown は SetupCluster で確保したリソースを破棄する。
// defer cluster.Teardown(t) で呼ぶ。
func (c *Cluster) Teardown(t *testing.T) {
	t.Helper()
}
