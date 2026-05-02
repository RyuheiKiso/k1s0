// 本ファイルはテナント新規オンボーディング E2E シナリオ。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 (Test Pyramid + testcontainers) / ADR-TEST-002 (E2E 自動化)
//
// 前提:
//   1. tools/local-stack/up.sh --role tier1-go-dev で kind cluster + 本番再現スタック起動済
//   2. tier1 公開 API への gRPC 接続経路が用意されている（kubectl port-forward 等）
//      env var K1S0_TIER1_TARGET=localhost:50001 のように endpoint を指定する
//
// 検証段階:
//   - リリース時点（本実装）: dapr-system Running Pod 確認 + tier1 State.Save → Get → Delete サイクル
//   - 採用初期: tier1 公開 API の他動詞（Workflow.Start / Secrets.Get / PubSub.Publish 等）追加
//   - 採用後の運用拡大時: tier2 BFF + tier3 Web を経由する完全 chain 検証
package scenarios

import (
	"bytes"
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	// k1s0 SDK 高水準 facade — Client.State() 等で動詞統一 API を呼ぶ
	"github.com/k1s0/sdk-go/k1s0"

	// E2E 共通 helper（kind cluster 接続 / namespace 管理）
	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestTenantOnboarding は tier1 → tier2 → tier3 を貫通する最小フローを検証する。
// 並列実行不可（cluster 状態を共有するため、t.Parallel() を呼ばない）。
func TestTenantOnboarding(t *testing.T) {
	// kind cluster への接続（未起動なら test 全体が Skip）
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	// 全体タイムアウト 90 秒
	ctx, cancel := context.WithTimeout(context.Background(), 90*time.Second)
	defer cancel()

	// サブテスト 1: dapr-system namespace に Running Pod が存在することの確認
	t.Run("dapr-system_running", func(t *testing.T) {
		// 60 秒以内に少なくとも 1 つの Running Pod を期待する
		running, err := cluster.WaitForRunningPodInNamespace(ctx, "dapr-system", 60*time.Second)
		if err != nil {
			t.Fatalf("namespace dapr-system で Running Pod を待機中に失敗: %v", err)
		}
		// 観測した Running Pod 数を test ログに残す
		t.Logf("namespace dapr-system: %d Running Pod 確認", running)
	})

	// サブテスト 2: tier1 State の Save → Get → Delete サイクル
	t.Run("tier1_state_save_get_delete", func(t *testing.T) {
		runStateRoundTrip(t, ctx)
	})
}

// runStateRoundTrip は tier1 State 公開 API への gRPC 疎通を検証する。
// K1S0_TIER1_TARGET env var 未設定なら test を Skip（kubectl port-forward 等で endpoint
// を露出する手順は採用初期で Runbook 化する）。
func runStateRoundTrip(t *testing.T, ctx context.Context) {
	// tier1 公開 gRPC endpoint を env var から取得
	target := os.Getenv("K1S0_TIER1_TARGET")
	if target == "" {
		// endpoint 未指定なら本サブテストを Skip（kind 接続のみ検証する経路）
		t.Skip("K1S0_TIER1_TARGET 未設定: kubectl port-forward svc/tier1-state -n tier1-state 50001:50001 等で endpoint を露出してから K1S0_TIER1_TARGET=localhost:50001 で再実行")
	}

	// k1s0 SDK Client を初期化（dev 用に平文 gRPC、テスト用 tenant_id を付与）
	client, err := k1s0.New(ctx, k1s0.Config{
		// gRPC 接続先（例: localhost:50001）
		Target: target,
		// テスト専用 tenant_id（本番 namespace を汚染しない E2E 専用 prefix）
		TenantID: "e2e-tenant-onboarding",
		// 主体識別子（監査ログで test 起源を識別する）
		Subject: "e2e-test/tenant-onboarding",
		// dev 用に平文（kind 内部通信、TLS は採用初期で SPIRE / cert-manager 経由）
		UseTLS: false,
	})
	// 接続失敗は致命的（Skip ではなく Fatal — endpoint 指定があったのに接続できないのは bug）
	if err != nil {
		t.Fatalf("k1s0.New(target=%s): %v", target, err)
	}
	// test 終了時に gRPC connection を解放
	defer func() { _ = client.Close() }()

	// State store 名（local-stack の Dapr Component 名と一致させる、state.in-memory が dev 既定）
	const store = "state.in-memory"
	// E2E 専用 key（既存データと衝突しないよう test 名 + 時刻ベース）
	key := fmt.Sprintf("e2e-onboarding-%d", time.Now().UnixNano())
	// 検証用 payload（小さい固定値）
	want := []byte("hello-from-e2e")

	// Step 1: State.Save — 値を書き込む
	etag, err := client.State().Save(ctx, store, key, want)
	if err != nil {
		t.Fatalf("State.Save(store=%s, key=%s): %v", store, key, err)
	}
	// Save 成功時の etag をログに残す（後続 Get / Delete の冪等性確認用）
	t.Logf("State.Save: etag=%s", etag)

	// Step 2: State.Get — 直前に書いた値を取得
	got, gotEtag, found, err := client.State().Get(ctx, store, key)
	if err != nil {
		t.Fatalf("State.Get(store=%s, key=%s): %v", store, key, err)
	}
	// 「Save した直後の Get で found=true」が成立しないと state store が壊れている
	if !found {
		t.Fatalf("State.Get: found=false（直前に Save した key が見えない）")
	}
	// 取得値が Save 時の値と一致することを assert
	if !bytes.Equal(got, want) {
		t.Fatalf("State.Get: data mismatch: got=%q want=%q", got, want)
	}
	// etag は実装依存だが空でないこと
	if gotEtag == "" {
		t.Fatalf("State.Get: etag が空（CAS が成立しない）")
	}
	// Get 成功をログに残す
	t.Logf("State.Get: data=%q etag=%s", got, gotEtag)

	// Step 3: State.Delete — 後始末（無条件削除、空 etag）
	if err := client.State().Delete(ctx, store, key, ""); err != nil {
		t.Fatalf("State.Delete(store=%s, key=%s): %v", store, key, err)
	}

	// Step 4: 削除確認 — Get で found=false が返ること
	_, _, foundAfterDelete, err := client.State().Get(ctx, store, key)
	if err != nil {
		t.Fatalf("State.Get(after delete): %v", err)
	}
	// 削除直後に found=true なら delete が効いていない（一貫性 bug）
	if foundAfterDelete {
		t.Fatalf("State.Get(after delete): found=true（delete が効いていない）")
	}
	// 一連のサイクル成功をログに残す
	t.Logf("State Save → Get → Delete cycle 成功")
}
