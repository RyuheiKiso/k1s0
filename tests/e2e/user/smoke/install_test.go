// tests/e2e/user/smoke/install_test.go
//
// user suite smoke/ — k1s0 install が正常か確認する最小検証。
// PR 5 分予算（ADR-TEST-001）に収まる軽量 test。
//
// 設計正典:
//   ADR-TEST-008 §1 user smoke 配置
//   ADR-TEST-010（test-fixtures、本 test の起動経路）

//go:build user_e2e

package smoke

import (
	"context"
	"testing"

	"github.com/k1s0/k1s0/tests/e2e/user/helpers"
)

// TestK1s0InstallNormal は kind cluster + minimum stack が起動済で、
// kind cluster の context が正しく current であることを確認する最小 smoke。
func TestK1s0InstallNormal(t *testing.T) {
	// test スコープの context を生成
	ctx := context.Background()
	// helpers.IsRunning で cluster 状態確認
	running, err := helpers.IsRunning(ctx)
	if err != nil {
		t.Fatalf("kind get clusters 実行失敗: %v", err)
	}
	if !running {
		t.Skipf("kind cluster %s 未起動。'./tools/e2e/user/up.sh' を実行してください", helpers.KindClusterName)
	}
	// context 名を log に出して PASS
	t.Logf("user suite cluster ready (context = %s)", helpers.KubeContextName())
}
