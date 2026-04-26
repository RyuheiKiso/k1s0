// 本ファイルは internal/common/timeout.go の単体テスト。
//
// 設計: plan/04_tier1_Goファサード実装/02_共通基盤.md（主作業 5: timeout 階層）
//       SLO 予算: 業務 200 + Dapr 80 + OTel 20 + 監査 50 + net/DB 150 = 500ms / API
// 関連 ID: NFR-B-PERF-* / IMP-RELIABILITY-*

package common

// 標準ライブラリのみ使用。
import (
	// context 操作。
	"context"
	// テストフレームワーク。
	"testing"
	// 時間定数照合。
	"time"
)

// TestTimeoutConstants は階層 timeout 定数が SLO 予算と整合していることを確認する。
func TestTimeoutConstants(t *testing.T) {
	// FacadeBudget = 500ms（API 全体の SLO 予算上限）。
	if FacadeBudget != 500*time.Millisecond {
		// 期待値と異なる場合は fail（SLO 予算は不変）。
		t.Errorf("FacadeBudget = %v, want 500ms", FacadeBudget)
		// if 分岐を閉じる。
	}
	// DaprCall = 200ms（Dapr sidecar 呼び出し上限）。
	if DaprCall != 200*time.Millisecond {
		// 期待値と異なる場合は fail。
		t.Errorf("DaprCall = %v, want 200ms", DaprCall)
		// if 分岐を閉じる。
	}
	// RustCoreCall = 100ms（Rust core への internal gRPC 上限）。
	if RustCoreCall != 100*time.Millisecond {
		// 期待値と異なる場合は fail。
		t.Errorf("RustCoreCall = %v, want 100ms", RustCoreCall)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}

// TestWithFacadeTimeout は子 context が deadline を持つことを確認する。
func TestWithFacadeTimeout(t *testing.T) {
	// 親 context（deadline なし）。
	ctx, cancel := WithFacadeTimeout(context.Background())
	// cancel を必ず defer する（go vet 対策、本テスト自身もこれを exercise する）。
	defer cancel()

	// 子 context は deadline を持つはず。
	deadline, ok := ctx.Deadline()
	// deadline が無いなら fail（FacadeBudget が効いてない）。
	if !ok {
		// fail。
		t.Fatal("WithFacadeTimeout returned ctx without deadline")
		// if 分岐を閉じる。
	}
	// deadline までの残り時間が FacadeBudget に近いことを確認する（許容差 50ms、テスト遅延考慮）。
	remaining := time.Until(deadline)
	// 範囲チェック: 450ms < remaining <= 500ms（FacadeBudget の 90% 以上）。
	if remaining < FacadeBudget-50*time.Millisecond || remaining > FacadeBudget {
		// fail。
		t.Errorf("remaining = %v, want close to %v", remaining, FacadeBudget)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}

// TestWithDaprTimeout は Dapr 呼び出し上限が適用されることを確認する。
func TestWithDaprTimeout(t *testing.T) {
	// 親 context は deadline なし。
	ctx, cancel := WithDaprTimeout(context.Background())
	// cancel を defer。
	defer cancel()

	// 子 context の deadline を取得する。
	deadline, ok := ctx.Deadline()
	// deadline 必須。
	if !ok {
		// fail。
		t.Fatal("WithDaprTimeout returned ctx without deadline")
		// if 分岐を閉じる。
	}
	// DaprCall（200ms）相当の残り時間があることを確認する。
	remaining := time.Until(deadline)
	// 範囲: 150ms < remaining <= 200ms。
	if remaining < DaprCall-50*time.Millisecond || remaining > DaprCall {
		// fail。
		t.Errorf("remaining = %v, want close to %v", remaining, DaprCall)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}

// TestWithRustCoreTimeout は Rust core 呼び出し上限が適用されることを確認する。
func TestWithRustCoreTimeout(t *testing.T) {
	// 親 context は deadline なし。
	ctx, cancel := WithRustCoreTimeout(context.Background())
	// cancel を defer。
	defer cancel()

	// 子 context の deadline を取得する。
	deadline, ok := ctx.Deadline()
	// deadline 必須。
	if !ok {
		// fail。
		t.Fatal("WithRustCoreTimeout returned ctx without deadline")
		// if 分岐を閉じる。
	}
	// RustCoreCall（100ms）相当の残り時間。
	remaining := time.Until(deadline)
	// 範囲: 50ms < remaining <= 100ms。
	if remaining < RustCoreCall-50*time.Millisecond || remaining > RustCoreCall {
		// fail。
		t.Errorf("remaining = %v, want close to %v", remaining, RustCoreCall)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}
