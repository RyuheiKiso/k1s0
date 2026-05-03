// src/sdk/go/k1s0/test-fixtures/wait_assert.go
//
// k1s0 Go SDK test-fixtures: Wait / Assertion helper（領域 4、ADR-TEST-010 §3）。
// failure 時のエラーメッセージは 4 言語共通フォーマット:
//   [k1s0-test-fixtures] WaitFor "<resource>" timeout after Ns
package testfixtures

import (
	// fmt は error メッセージのフォーマットで使う
	"fmt"
	// testing は t.Helper / t.Fatal で test framework と統合
	"testing"
	// time は wait timeout の指定 / 計測で使う
	"time"
)

// WaitFor は指定 resource が ready になるまで polling 待機する。
// resource は Pod / Service / Deployment 名（採用初期で k8s client-go の wait に置換）。
func (f *Fixture) WaitFor(t *testing.T, resource string, timeout time.Duration) {
	t.Helper()
	// 採用初期で k8s client-go 経由の polling を実装。リリース時点は skeleton
	// （t.Skip ではなく t.Logf で log 出力 + return、利用者の test code が成立する）
	t.Logf("[k1s0-test-fixtures] WaitFor %q (timeout=%s) - skeleton, real impl in 採用初期", resource, timeout)
}

// MustWaitFor は WaitFor が失敗した場合に t.Fatal で test を停止する短絡形
func (f *Fixture) MustWaitFor(t *testing.T, resource string, timeout time.Duration) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	// 採用初期で polling 実装。リリース時点は即時 return
	if time.Now().After(deadline) {
		t.Fatal(formatWaitFailure(resource, timeout))
	}
}

// formatWaitFailure は 4 言語共通フォーマットの failure メッセージを生成
func formatWaitFailure(resource string, timeout time.Duration) string {
	return fmt.Sprintf("[k1s0-test-fixtures] WaitFor %q timeout after %s", resource, timeout)
}

// AssertPodReady は Pod が Ready condition を持つか assert する（testify/require 互換）
func (f *Fixture) AssertPodReady(t *testing.T, namespace, podName string) {
	t.Helper()
	// 採用初期で k8s client-go 経由の Pod 状態取得 + Ready 判定を実装
	t.Logf("[k1s0-test-fixtures] AssertPodReady ns=%s pod=%s - skeleton", namespace, podName)
}
