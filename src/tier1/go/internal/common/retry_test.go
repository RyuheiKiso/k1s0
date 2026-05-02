// 本ファイルは internal/common/retry.go の単体テスト。
//
// 設計: plan/04_tier1_Goファサード実装/02_共通基盤.md（主作業 3: retry 戦略）
// 関連 ID: IMP-RELIABILITY-* / NFR-B-PERF-*

package common

// 標準ライブラリのみ使用。
import (
	// テストフレームワーク。
	"testing"
	// 時間定数照合。
	"time"
)

// TestDefaultRetry はデフォルト retry 戦略の値が plan 04-02 主作業 3 と整合することを確認する。
func TestDefaultRetry(t *testing.T) {
	// デフォルト戦略を取得する。
	cfg := DefaultRetry()

	// MaxAttempts: 初回 + 2 retry = 計 3 試行。
	if cfg.MaxAttempts != 3 {
		// 期待値と異なる場合は fail。
		t.Errorf("MaxAttempts = %d, want 3", cfg.MaxAttempts)
		// if 分岐を閉じる。
	}
	// InitialDelay: 初回 retry 前の待機 100ms。
	if cfg.InitialDelay != 100*time.Millisecond {
		// fail。
		t.Errorf("InitialDelay = %v, want 100ms", cfg.InitialDelay)
		// if 分岐を閉じる。
	}
	// MaxDelay: 上限 400ms（指数増加 100→200→400 のキャップ）。
	if cfg.MaxDelay != 400*time.Millisecond {
		// fail。
		t.Errorf("MaxDelay = %v, want 400ms", cfg.MaxDelay)
		// if 分岐を閉じる。
	}
	// JitterRatio: ±20%（共通規約 §「冪等性と再試行」"jitter ±20%" に整合）。
	if cfg.JitterRatio != 0.2 {
		// fail。
		t.Errorf("JitterRatio = %v, want 0.2", cfg.JitterRatio)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}

// TestRetryConfig_SLOBudget はデフォルト retry 戦略の累積遅延が SLO 予算（500ms）を超えないことを確認する。
//
// シナリオ: 初回失敗 → 100ms 待機 → 1 回目 retry 失敗 → 200ms 待機 → 2 回目 retry 成功。
// 累積遅延: 100 + 200 = 300ms < FacadeBudget(500ms)。
func TestRetryConfig_SLOBudget(t *testing.T) {
	// デフォルト戦略を取得する。
	cfg := DefaultRetry()

	// 最悪ケースの累積遅延（initial + initial*2 = 100 + 200 = 300ms）。
	worstCaseDelay := cfg.InitialDelay + cfg.InitialDelay*2

	// FacadeBudget は 500ms（timeout.go の SLO 予算）。
	if worstCaseDelay >= FacadeBudget {
		// 累積遅延が予算超過なら fail。
		t.Errorf("worstCaseDelay = %v >= FacadeBudget = %v, retry exceeds SLO", worstCaseDelay, FacadeBudget)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}
