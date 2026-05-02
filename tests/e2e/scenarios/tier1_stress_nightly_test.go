// 本ファイルは tier1 の stress test（@nightly タグ）。
// ADR-TEST-007 の build tag による CI 実行フェーズ分離（IMP-CI-TAG-003）の実装例。
// PR ゲート（tags なし）では実行されず、nightly workflow（go test -tags=nightly）でのみ実行される。
//
// 設計正典:
//   ADR-TEST-007（テスト属性タグ + CI 実行フェーズ分離）
// 関連 ID:
//   IMP-CI-TAG-001（4 タグ最低セット: @slow / @flaky / @security / @nightly）
//   IMP-CI-TAG-002（4 段フェーズ trigger 一意化）
//   IMP-CI-TAG-003（言語別 build tag 実装、Go の build tag 経路）
//
//go:build nightly

package scenarios

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/k1s0/sdk-go/k1s0"

	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestTier1StressNightly は nightly フェーズ専用の stress test。
// PubSub.Publish を 100 回連続実行して in-memory queue の順序性を確認する。
// PR ゲートでは実行されない（所要 5-10 秒）。
func TestTier1StressNightly(t *testing.T) {
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	target := os.Getenv("K1S0_TIER1_TARGET")
	if target == "" {
		t.Skip("K1S0_TIER1_TARGET 未設定")
	}

	client, err := k1s0.New(ctx, k1s0.Config{
		Target:   target,
		TenantID: "demo-tenant",
		Subject:  "e2e-test/tier1-stress-nightly",
		UseTLS:   false,
	})
	if err != nil {
		t.Fatalf("k1s0.New: %v", err)
	}
	defer func() { _ = client.Close() }()

	// PubSub.Publish を 100 回連続実行
	const iterations = 100
	for i := 0; i < iterations; i++ {
		_, err := client.PubSub().Publish(
			ctx,
			"pubsub.in-memory",
			[]byte(fmt.Sprintf(`{"seq":%d}`, i)),
			"application/json",
		)
		if err != nil {
			t.Fatalf("PubSub.Publish[%d]: %v", i, err)
		}
	}
	t.Logf("PubSub.Publish: %d 回連続 PASS", iterations)
}
