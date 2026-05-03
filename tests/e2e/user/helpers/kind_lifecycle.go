// tests/e2e/user/helpers/kind_lifecycle.go
//
// user suite で kind cluster の起動 / 状態確認を Go test から叩く薄い helper。
// 通常は test-fixtures (src/sdk/<lang>/test-fixtures、ADR-TEST-010) を使うことが推奨だが、
// helpers/kind_lifecycle.go は test-fixtures の Setup / Teardown が下地で呼ぶ薄い primitive。
//
// 設計正典:
//   ADR-TEST-008（user suite 環境契約）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/02_ディレクトリ構造.md
package helpers

import (
	"context"
	"fmt"
	"os/exec"
	"strings"
)

// KindClusterName は user suite の kind cluster 名（tools/e2e/user/up.sh と同じ）
const KindClusterName = "k1s0-user-e2e"

// IsRunning は kind cluster k1s0-user-e2e が起動しているか確認する
func IsRunning(_ context.Context) (bool, error) {
	// kind get clusters で一覧取得
	cmd := exec.Command("kind", "get", "clusters")
	out, err := cmd.Output()
	if err != nil {
		return false, fmt.Errorf("kind get clusters 失敗: %w", err)
	}
	// 改行区切りで一致確認
	clusters := strings.Split(strings.TrimSpace(string(out)), "\n")
	for _, c := range clusters {
		if c == KindClusterName {
			return true, nil
		}
	}
	return false, nil
}

// KubeContextName は kind cluster の kubectl context 名
func KubeContextName() string {
	return "kind-" + KindClusterName
}
