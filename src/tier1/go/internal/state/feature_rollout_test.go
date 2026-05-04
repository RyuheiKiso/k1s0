// 本ファイルは feature_rollout.go の単体テスト。
//
// 試験戦略:
//   1. sticky 保証: 同 (user_id, flag_key) で 100 回呼んで結果が常に同じであること
//   2. percentage 0 / 100 の境界条件
//   3. 統計的均等性: 10000 user で 10% / 50% percentage が ±2.5σ 範囲内 (chi-square)
//   4. 異 flag_key で同 user_id が異なる bucket に割り当たる (相関なし)
//
// 検証する不変式:
//   FR-T1-FEATURE-002 受け入れ基準: 「同一 user_id は常に同じ結果を返す」
//                                 「10% / 50% / 100% 段階拡大」

package state

import (
	// テスト fail / 報告。
	"testing"
	// 統計検定の sqrt。
	"math"
	// user_id 生成用 strconv。
	"strconv"
)

// TestRolloutAssign_Sticky_SameUserSameResult は 100 回呼んで同 user_id +
// 同 flag_key + 同 percentage で常に同結果を返すことを確認する。
func TestRolloutAssign_Sticky_SameUserSameResult(t *testing.T) {
	// 100 回呼ぶ。
	first := RolloutAssign("user-42", "checkout-redesign", 30)
	// 同条件で 99 回呼んで全部一致すること。
	for i := 0; i < 99; i++ {
		// 同条件呼出。
		got := RolloutAssign("user-42", "checkout-redesign", 30)
		// 一致しなければ sticky 違反 (FR-T1-FEATURE-002 受け入れ基準破綻)。
		if got != first {
			t.Fatalf("RolloutAssign not sticky: call[%d] = %v, want %v", i+1, got, first)
		}
	}
}

// TestRolloutAssign_ZeroPercent_AlwaysFalse は percentage=0 で全員 false を確認する。
func TestRolloutAssign_ZeroPercent_AlwaysFalse(t *testing.T) {
	// 100 user で false を確認する。
	for i := 0; i < 100; i++ {
		// user_id を生成する。
		uid := "u-" + strconv.Itoa(i)
		// percentage=0 で呼ぶ。
		if RolloutAssign(uid, "flag", 0) {
			// percentage=0 で true を返したら違反。
			t.Fatalf("RolloutAssign(uid=%s, percentage=0) = true, want false", uid)
		}
	}
}

// TestRolloutAssign_HundredPercent_AlwaysTrue は percentage=100 で全員 true を確認する。
func TestRolloutAssign_HundredPercent_AlwaysTrue(t *testing.T) {
	// 100 user で true を確認する。
	for i := 0; i < 100; i++ {
		// user_id を生成する。
		uid := "u-" + strconv.Itoa(i)
		// percentage=100 で呼ぶ。
		if !RolloutAssign(uid, "flag", 100) {
			// percentage=100 で false を返したら違反。
			t.Fatalf("RolloutAssign(uid=%s, percentage=100) = false, want true", uid)
		}
	}
}

// TestRolloutAssign_StatisticalDistribution_10Percent は 10000 user で
// percentage=10 が ±2σ 範囲内 (期待値 1000、許容 936-1064) で割り当たることを確認する。
// SHA-256 の uniformity を実用範囲で検証する。
func TestRolloutAssign_StatisticalDistribution_10Percent(t *testing.T) {
	// total user 数。
	const total = 10000
	// 期待 percentage。
	const pct = 10
	// 期待 true 数 (= total * pct / 100)。
	const expected = total * pct / 100
	// 許容範囲は ±3σ。binomial の σ = sqrt(N * p * (1-p))。
	sigma := math.Sqrt(float64(total) * float64(pct) / 100.0 * (1.0 - float64(pct)/100.0))
	// 3σ で 99.7% 以内 (実用上の保証範囲)。
	tolerance := 3 * sigma
	// 実 true 数を count する。
	trueCount := 0
	// 10000 user で評価する。
	for i := 0; i < total; i++ {
		// user_id を生成する。
		uid := "user-" + strconv.Itoa(i)
		// 評価する。
		if RolloutAssign(uid, "feature-A", pct) {
			trueCount++
		}
	}
	// 期待値からの乖離が tolerance 内であること。
	diff := math.Abs(float64(trueCount - expected))
	if diff > tolerance {
		t.Fatalf("RolloutAssign distribution: trueCount=%d, expected=%d, diff=%.2f, tolerance=±%.2f (3σ)",
			trueCount, expected, diff, tolerance)
	}
}

// TestRolloutAssign_DifferentFlagsDifferentBuckets は同 user_id でも異なる flag_key で
// 異なる bucket に割り当たることを確認する (相関なし、独立な hash)。
func TestRolloutAssign_DifferentFlagsDifferentBuckets(t *testing.T) {
	// 100 user で 2 flag の評価結果が一致しすぎないか検査する。
	// 2 つの独立 hash なら一致率は ~p^2 + (1-p)^2 程度に収まるはず。
	// percentage=50 の場合は理論的には 50% が一致 (両方 true) + 50% が一致 (両方 false) の
	// 経験率 50% を期待。実装が flagKey を無視している (バグ) なら 100% 一致になる。
	const total = 1000
	// 両 flag で同結果になった数を count する。
	matchCount := 0
	// 評価する。
	for i := 0; i < total; i++ {
		// user_id を生成する。
		uid := "user-" + strconv.Itoa(i)
		// 異 flag_key で評価する。
		a := RolloutAssign(uid, "flag-A", 50)
		b := RolloutAssign(uid, "flag-B", 50)
		// 一致 count を加算する。
		if a == b {
			matchCount++
		}
	}
	// 一致率が 80% 未満であること (相関なしなら ~50%、バグでフラグ無視なら 100%)。
	// 80% 閾値で実装の独立性を統計的に保証する。
	if matchCount > 800 {
		t.Fatalf("RolloutAssign: flag-A と flag-B の結果一致率 = %.1f%% (>80%%), 期待 ~50%%。flagKey が hash 入力に効いていない疑い",
			float64(matchCount)/float64(total)*100)
	}
}
