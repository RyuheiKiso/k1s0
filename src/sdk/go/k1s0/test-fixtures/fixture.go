// src/sdk/go/k1s0/test-fixtures/fixture.go
//
// k1s0 Go SDK test-fixtures: Setup / Teardown / Fixture struct。
// 利用者が import 1 行で kind 起動 + k1s0 install + SDK client init を立てられる。
//
// 設計正典:
//   ADR-TEST-010 §3 領域 1（Setup / Teardown）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md
package testfixtures

import (
	// fmt は error format 用
	"fmt"
	// os/exec は kind / helm / kubectl 子プロセス呼び出し用
	"os/exec"
	// testing は t.Helper / t.Cleanup で test framework と統合する
	"testing"
)

// Fixture は Setup の戻り値。利用者が test code から SDK client init や mock builder を取得する経路。
type Fixture struct {
	// Options は Setup に渡された設定（再利用 + debug 用）
	Options Options
	// MockBuilder は 12 service の mock data builder への entry point
	MockBuilder *MockBuilderRoot
	// kubeContext は kubectl context 名（teardown / status 確認で使う）
	kubeContext string
}

// Setup は kind cluster 起動 + k1s0 install + SDK client の前提整備を行う。
// t.Cleanup で teardown を自動登録するため、利用者は明示的に Teardown を呼ぶ必要が少ない。
//
// real 実装は採用初期で kind / helm / kubectl の子プロセス呼び出し + tools/e2e/user/up.sh
// の wrapper として実装する。リリース時点では skeleton（cluster 起動済前提で fixture struct のみ返す）。
func Setup(t *testing.T, opts Options) *Fixture {
	t.Helper()

	// Options 既定値の補完（Tenant / Namespace 未指定時は DefaultOptions の値で埋める）
	if opts.Tenant == "" {
		opts.Tenant = "tenant-a"
	}
	if opts.Namespace == "" {
		opts.Namespace = "k1s0"
	}
	if opts.KindNodes == 0 {
		opts.KindNodes = 2
	}

	// kind cluster の状態確認（リリース時点では tools/e2e/user/up.sh を別途実行済前提）
	// 採用初期で本関数内から up.sh を spawn する形に拡張する。
	fx := &Fixture{
		Options:     opts,
		MockBuilder: newMockBuilderRoot(opts.Tenant),
		kubeContext: "kind-k1s0-user-e2e",
	}

	// t.Cleanup で teardown を予約（test 終了時に自動実行）
	t.Cleanup(func() {
		// skeleton: 採用初期で tools/e2e/user/down.sh を呼ぶ
		// リリース時点では cluster 維持 + 利用者の判断で down.sh を別途実行
	})

	return fx
}

// Teardown は Setup 後に呼ぶ後片付け。t.Cleanup 登録済のため通常は不要だが、
// 明示的な呼び出しを許容する（owner suite + user suite で対称形にするため）。
func (f *Fixture) Teardown() {
	// skeleton: 採用初期で tools/e2e/user/down.sh の spawn を実装
	// リリース時点は no-op
}

// WaitForTier1FacadeReady は tier1 facade Pod が Ready 状態になるまで待機する。
// 採用初期で kubectl wait の wrapper として実装する。
func (f *Fixture) WaitForTier1FacadeReady() error {
	// 採用初期で k8s client-go 経由の Pod readiness 待機を実装
	cmd := exec.Command("kubectl", "wait", "--context", f.kubeContext,
		"-n", f.Options.Namespace,
		"--for=condition=Ready", "pod", "-l", "app=tier1-facade",
		"--timeout=300s")
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("tier1 facade Ready 待機失敗: %w", err)
	}
	return nil
}
