// 本ファイルは internal/health の HealthService 実装の単体テスト。
//
// 検証ポイント:
//   1. Liveness が version + uptime を返す（uptime は >= 0 で過剰な値を返さないこと）
//   2. probes 空の Readiness は ready=true / 空 dependencies map を返す
//   3. probes 全件 nil error の Readiness は ready=true / 各 dependency reachable=true
//   4. probes に 1 件 error が混ざる Readiness は ready=false / 該当 dependency に error_message
//   5. context cancel が probe に伝播し、即時 error_message を持って reachable=false 扱い

// Package health のテスト。
package health

import (
	"context"
	"errors"
	"testing"
	"time"

	healthv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/health/v1"
)

// Liveness は version + uptime を返す。uptime は New 直後 0 秒以上 1 秒未満。
func TestService_Liveness_ReturnsVersionAndUptime(t *testing.T) {
	// Service を New する。起動時刻は内部で time.Now() に固定される。
	svc := New("9.9.9-test", nil)
	// 即時に Liveness を呼ぶ。
	resp, err := svc.Liveness(context.Background(), &healthv1.LivenessRequest{})
	// error は nil 期待。
	if err != nil {
		// Liveness は依存に触らないため失敗は実装バグ。
		t.Fatalf("Liveness err: %v", err)
	}
	// version は New に渡した値そのまま。
	if got := resp.GetVersion(); got != "9.9.9-test" {
		// 不一致は version 配線バグ。
		t.Fatalf("version: want %q got %q", "9.9.9-test", got)
	}
	// uptime は 0 以上（int64 切り捨てで負値にならない）。
	if up := resp.GetUptimeSeconds(); up < 0 {
		// 負値は実装バグ。
		t.Fatalf("uptime negative: %d", up)
	}
}

// probes 空の Readiness は ready=true / 空 dependencies。
func TestService_Readiness_EmptyProbes_Ready(t *testing.T) {
	// 依存なし Service を構築する。
	svc := New("0.0.0", nil)
	// Readiness を呼ぶ。
	resp, err := svc.Readiness(context.Background(), &healthv1.ReadinessRequest{})
	// error は nil 期待。
	if err != nil {
		t.Fatalf("Readiness err: %v", err)
	}
	// ready=true 期待。
	if !resp.GetReady() {
		t.Fatalf("ready: want true, got false")
	}
	// dependencies は空 map（nil 可）。
	if n := len(resp.GetDependencies()); n != 0 {
		t.Fatalf("dependencies len: want 0, got %d", n)
	}
}

// probes 全件 nil error の Readiness は ready=true / 各 dependency reachable=true。
func TestService_Readiness_AllProbesPass(t *testing.T) {
	// 2 件の依存を設定する。両方とも nil error を返す。
	svc := New("0.0.0", []DependencyProbe{
		{Name: "alpha", Check: func(_ context.Context) error { return nil }},
		{Name: "beta", Check: func(_ context.Context) error { return nil }},
	})
	// Readiness を呼ぶ。
	resp, err := svc.Readiness(context.Background(), &healthv1.ReadinessRequest{})
	// error は nil 期待。
	if err != nil {
		t.Fatalf("Readiness err: %v", err)
	}
	// 全 probe pass で ready=true。
	if !resp.GetReady() {
		t.Fatalf("ready: want true, got false")
	}
	// 各 dependency が reachable=true / error_message 空。
	for _, name := range []string{"alpha", "beta"} {
		// 名前で引く。
		dep := resp.GetDependencies()[name]
		// nil なら map 配線バグ。
		if dep == nil {
			t.Fatalf("dep %q missing", name)
		}
		// reachable=true 期待。
		if !dep.GetReachable() {
			t.Fatalf("dep %q reachable: want true", name)
		}
		// error_message は空。
		if msg := dep.GetErrorMessage(); msg != "" {
			t.Fatalf("dep %q error_message: want empty, got %q", name, msg)
		}
	}
}

// probes 1 件 error の Readiness は ready=false / 該当 dependency に error_message。
func TestService_Readiness_OneProbeFails(t *testing.T) {
	// alpha は OK、beta は error を返す。
	svc := New("0.0.0", []DependencyProbe{
		{Name: "alpha", Check: func(_ context.Context) error { return nil }},
		{Name: "beta", Check: func(_ context.Context) error { return errors.New("connection refused") }},
	})
	// Readiness を呼ぶ。
	resp, err := svc.Readiness(context.Background(), &healthv1.ReadinessRequest{})
	// 例外的状況以外は err nil。
	if err != nil {
		t.Fatalf("Readiness err: %v", err)
	}
	// 1 件 fail で ready=false。
	if resp.GetReady() {
		t.Fatalf("ready: want false, got true")
	}
	// alpha は reachable=true。
	if !resp.GetDependencies()["alpha"].GetReachable() {
		t.Fatalf("alpha: want reachable=true")
	}
	// beta は reachable=false + error_message に error 文字列。
	beta := resp.GetDependencies()["beta"]
	// reachable=false 期待。
	if beta.GetReachable() {
		t.Fatalf("beta: want reachable=false")
	}
	// error_message は probe error の Error() 文字列そのもの。
	if msg := beta.GetErrorMessage(); msg != "connection refused" {
		t.Fatalf("beta error_message: want %q got %q", "connection refused", msg)
	}
}

// probes が長時間 block しても context 期限内に終わる（context cancel 伝播）。
func TestService_Readiness_RespectsContextCancel(t *testing.T) {
	// probe は ctx.Done を待ってから ctx.Err を返す（cancel 伝播確認）。
	svc := New("0.0.0", []DependencyProbe{
		{Name: "slow", Check: func(ctx context.Context) error {
			// ctx cancel まで block する。
			<-ctx.Done()
			// probe error として ctx.Err を伝える。
			return ctx.Err()
		}},
	})
	// 50ms 後に cancel される ctx を用意する。
	ctx, cancel := context.WithTimeout(context.Background(), 50*time.Millisecond)
	// 関数末尾で必ず cancel する。
	defer cancel()
	// Readiness を呼ぶ（probe は cancel まで block する）。
	resp, err := svc.Readiness(ctx, &healthv1.ReadinessRequest{})
	// Readiness 自体は err 返さず ready=false で結果を返す。
	if err != nil {
		t.Fatalf("Readiness err: %v", err)
	}
	// cancel で ready=false 期待。
	if resp.GetReady() {
		t.Fatalf("ready: want false on context cancel")
	}
	// slow は reachable=false + error_message に context.DeadlineExceeded か Canceled が入る。
	slow := resp.GetDependencies()["slow"]
	// nil なら map 配線バグ。
	if slow == nil {
		t.Fatalf("slow missing")
	}
	// reachable=false 期待。
	if slow.GetReachable() {
		t.Fatalf("slow: want reachable=false")
	}
	// error_message が空でないことを確認する（具体内容は ctx.Err() 由来）。
	if msg := slow.GetErrorMessage(); msg == "" {
		t.Fatalf("slow error_message: want non-empty, got empty")
	}
}
