// 本ファイルは internal/common/retry.go の Do[T] / 判定関数 / backoff の単体テスト。
//
// 設計正典: 共通規約 §「冪等性と再試行」 / FR-T1-INVOKE-003
// 関連 ID: IMP-RELIABILITY-* / FR-T1-INVOKE-003

package common

import (
	// retry の context 監視テスト。
	"context"
	// retry を投げる擬似エラー。
	"errors"
	// テストフレームワーク。
	"testing"
	// 試行間隔測定。
	"time"

	// gRPC code/status を組み立てて retry 判定をテストする。
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// TestDefaultIsRetryable は共通規約の retryable 集合（Unavailable / ResourceExhausted / DeadlineExceeded）
// と非 retryable 集合の境界が正しいことを確認する。
func TestDefaultIsRetryable(t *testing.T) {
	// テストケース: gRPC code → 期待 retryable 判定。
	cases := []struct {
		code codes.Code
		want bool
	}{
		// 共通規約で retryable とされる 3 種。
		{codes.Unavailable, true},
		{codes.ResourceExhausted, true},
		{codes.DeadlineExceeded, true},
		// 共通規約で非 retryable とされる主要種。
		{codes.InvalidArgument, false},
		{codes.PermissionDenied, false},
		{codes.NotFound, false},
		{codes.Unauthenticated, false},
		{codes.AlreadyExists, false},
		{codes.Internal, false},
	}
	// 1 ケースずつ判定する。
	for _, c := range cases {
		// gRPC status から error を作る。
		err := status.Error(c.code, "test")
		// retryable 判定を実行して期待と比較する。
		if got := DefaultIsRetryable(err); got != c.want {
			t.Errorf("code=%v: DefaultIsRetryable=%v, want %v", c.code, got, c.want)
		}
	}
	// nil（成功）は retry 不要。
	if DefaultIsRetryable(nil) {
		t.Error("DefaultIsRetryable(nil) = true, want false")
	}
	// plain error（gRPC status を持たない）は retryable 不能。
	if DefaultIsRetryable(errors.New("plain")) {
		t.Error("DefaultIsRetryable(plain error) = true, want false")
	}
}

// TestMutationRetryable は副作用ありエラー判定で DeadlineExceeded を retry しないことを確認する。
//
// 共通規約: 「DeadlineExceeded はサーバ到達前のキャンセルを示す」が、副作用ありの操作では
// 重複実行リスク回避のため retry しない設計。
func TestMutationRetryable(t *testing.T) {
	// DeadlineExceeded は MutationRetryable で false を返す。
	if MutationRetryable(status.Error(codes.DeadlineExceeded, "deadline")) {
		t.Error("MutationRetryable(DeadlineExceeded) = true, want false (mutation should not retry on DeadlineExceeded)")
	}
	// Unavailable / ResourceExhausted は MutationRetryable で true を返す。
	if !MutationRetryable(status.Error(codes.Unavailable, "u")) {
		t.Error("MutationRetryable(Unavailable) = false, want true")
	}
	if !MutationRetryable(status.Error(codes.ResourceExhausted, "r")) {
		t.Error("MutationRetryable(ResourceExhausted) = false, want true")
	}
}

// TestComputeBackoff_Exponential は試行回数に対する待機時間が指数増加し MaxDelay でキャップされることを確認する。
//
// jitter 付き乱数なので jitter 範囲を考慮した境界比較を行う。
func TestComputeBackoff_Exponential(t *testing.T) {
	// jitter なし戦略を使って exact な値を検証する。
	cfg := RetryConfig{
		MaxAttempts:  4,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     400 * time.Millisecond,
		JitterRatio:  0,
	}
	// attempt=1: 100ms（initial）。
	if got := computeBackoff(1, cfg); got != 100*time.Millisecond {
		t.Errorf("attempt=1: got %v, want 100ms", got)
	}
	// attempt=2: 200ms（initial * 2）。
	if got := computeBackoff(2, cfg); got != 200*time.Millisecond {
		t.Errorf("attempt=2: got %v, want 200ms", got)
	}
	// attempt=3: min(400ms, MaxDelay)。
	if got := computeBackoff(3, cfg); got != 400*time.Millisecond {
		t.Errorf("attempt=3: got %v, want 400ms", got)
	}
	// attempt=4: 800ms → MaxDelay でキャップされて 400ms。
	if got := computeBackoff(4, cfg); got != 400*time.Millisecond {
		t.Errorf("attempt=4: got %v, want 400ms (capped by MaxDelay)", got)
	}
}

// TestComputeBackoff_Jitter は jitter 比率の範囲内に揺らぎが収まることを確認する。
func TestComputeBackoff_Jitter(t *testing.T) {
	// jitter ±50% で一定回数試行して全部範囲内に収まることを確認する。
	cfg := RetryConfig{
		MaxAttempts:  3,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     400 * time.Millisecond,
		JitterRatio:  0.5,
	}
	// nominal=100ms、jitter ±50% で 50ms〜150ms に収まるはず。
	min, max := 50*time.Millisecond, 150*time.Millisecond
	// 100 回試行して常に範囲内に収まることを確認する。
	for i := 0; i < 100; i++ {
		got := computeBackoff(1, cfg)
		if got < min || got > max {
			t.Errorf("iter %d: got %v, want in [%v, %v]", i, got, min, max)
		}
	}
}

// TestDo_SuccessFirstTry は成功時 1 回のみ試行することを確認する。
func TestDo_SuccessFirstTry(t *testing.T) {
	// 試行回数カウンタ。
	calls := 0
	// fn は常に成功する。
	fn := func() (int, error) {
		calls++
		return 42, nil
	}
	// Do で実行する。
	out, err := Do(context.Background(), DefaultRetry(), fn)
	// 成功を確認する。
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}
	if out != 42 {
		t.Errorf("out = %d, want 42", out)
	}
	if calls != 1 {
		t.Errorf("calls = %d, want 1 (no retry on success)", calls)
	}
}

// TestDo_NonRetryableErrorImmediate は non-retryable エラー時に即返却することを確認する。
func TestDo_NonRetryableErrorImmediate(t *testing.T) {
	calls := 0
	// PermissionDenied は retry 対象外。
	fn := func() (int, error) {
		calls++
		return 0, status.Error(codes.PermissionDenied, "denied")
	}
	out, err := Do(context.Background(), DefaultRetry(), fn)
	// エラーが返ること。
	if err == nil {
		t.Fatal("err = nil, want PermissionDenied")
	}
	// 試行回数が 1 のみであること（retry されない）。
	if calls != 1 {
		t.Errorf("calls = %d, want 1 (non-retryable should not retry)", calls)
	}
	// PermissionDenied がそのまま伝播されること。
	if st, ok := status.FromError(err); !ok || st.Code() != codes.PermissionDenied {
		t.Errorf("err code = %v, want PermissionDenied", st.Code())
	}
	// out のゼロ値も確認する。
	if out != 0 {
		t.Errorf("out = %d, want 0", out)
	}
}

// TestDo_RetryableEventuallySucceeds は retry で最終的に成功することを確認する。
func TestDo_RetryableEventuallySucceeds(t *testing.T) {
	calls := 0
	// 1 回目 / 2 回目: Unavailable、3 回目: 成功。
	fn := func() (string, error) {
		calls++
		if calls < 3 {
			return "", status.Error(codes.Unavailable, "transient")
		}
		return "ok", nil
	}
	cfg := RetryConfig{
		MaxAttempts:  3,
		InitialDelay: 1 * time.Millisecond,
		MaxDelay:     2 * time.Millisecond,
		JitterRatio:  0,
	}
	out, err := Do(context.Background(), cfg, fn)
	if err != nil {
		t.Fatalf("err = %v, want nil", err)
	}
	if out != "ok" {
		t.Errorf("out = %q, want \"ok\"", out)
	}
	if calls != 3 {
		t.Errorf("calls = %d, want 3", calls)
	}
}

// TestDo_AllAttemptsExhausted は全試行が retryable エラーで失敗した場合、最後のエラーを返すことを確認する。
func TestDo_AllAttemptsExhausted(t *testing.T) {
	calls := 0
	fn := func() (int, error) {
		calls++
		return 0, status.Error(codes.Unavailable, "still down")
	}
	cfg := RetryConfig{
		MaxAttempts:  3,
		InitialDelay: 1 * time.Millisecond,
		MaxDelay:     2 * time.Millisecond,
		JitterRatio:  0,
	}
	_, err := Do(context.Background(), cfg, fn)
	if err == nil {
		t.Fatal("err = nil, want Unavailable after exhaustion")
	}
	if calls != 3 {
		t.Errorf("calls = %d, want 3 (MaxAttempts)", calls)
	}
}

// TestDo_ContextCancelInterruptsRetry は context cancel で待機中の retry が打ち切られることを確認する。
func TestDo_ContextCancelInterruptsRetry(t *testing.T) {
	calls := 0
	fn := func() (int, error) {
		calls++
		return 0, status.Error(codes.Unavailable, "down")
	}
	cfg := RetryConfig{
		MaxAttempts:  10,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     500 * time.Millisecond,
		JitterRatio:  0,
	}
	// 50ms で cancel する context を用意する（最初の retry 待機 100ms より短い）。
	ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	defer cancel()
	start := time.Now()
	_, err := Do(ctx, cfg, fn)
	elapsed := time.Since(start)
	// エラーが返ること。
	if err == nil {
		t.Fatal("err = nil, want error after context cancel")
	}
	// 最初の試行直後に context cancel で打ち切られたはずなので、
	// retry 全体のフェーズ（100+200+400+...）よりはるかに早く終わるはず。
	if elapsed > 200*time.Millisecond {
		t.Errorf("elapsed = %v, want <200ms (context cancel should interrupt retry wait)", elapsed)
	}
	// 試行回数は 1 〜 2（最初の試行 + retry 待機中に cancel が入る）。
	if calls < 1 || calls > 2 {
		t.Errorf("calls = %d, want 1 or 2 (context cancel interrupted)", calls)
	}
}

// TestDo_PreservesIsRetryable は cfg.IsRetryable が DefaultIsRetryable と異なる場合、それが使われることを確認する。
func TestDo_PreservesIsRetryable(t *testing.T) {
	calls := 0
	fn := func() (int, error) {
		calls++
		// DeadlineExceeded を返す。
		return 0, status.Error(codes.DeadlineExceeded, "deadline")
	}
	// 副作用あり戦略（MutationRetryable）を渡す → DeadlineExceeded は retry しない。
	cfg := DefaultRetry()
	cfg.IsRetryable = MutationRetryable
	cfg.InitialDelay = 1 * time.Millisecond
	_, err := Do(context.Background(), cfg, fn)
	if err == nil {
		t.Fatal("err = nil, want DeadlineExceeded immediate")
	}
	if calls != 1 {
		t.Errorf("calls = %d, want 1 (DeadlineExceeded with MutationRetryable should not retry)", calls)
	}
}
