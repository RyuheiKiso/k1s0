// 本ファイルは Circuit Breaker（FR-T1-INVOKE-004）の単体テスト。
//
// 観点:
//   - 状態遷移: closed → open → half-open → closed/open
//   - 連続失敗閾値の境界
//   - half-open probe の上限
//   - observer 通知
//   - env 設定読込

package common

import (
	"sync"
	"testing"
	"time"
)

// TestCB_ClosedToOpenAfterThreshold は連続失敗が threshold に達すると open に遷移することを確認する。
func TestCB_ClosedToOpenAfterThreshold(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 3, HalfOpenAfter: 1 * time.Second, HalfOpenMaxProbes: 1})
	// 初期状態は closed。
	if cb.State() != CBClosed {
		t.Fatalf("initial state = %v, want closed", cb.State())
	}
	// 2 回失敗しても closed のまま。
	cb.RecordFailure()
	cb.RecordFailure()
	if cb.State() != CBClosed {
		t.Errorf("after 2 failures: state = %v, want closed", cb.State())
	}
	// 3 回目で open。
	cb.RecordFailure()
	if cb.State() != CBOpen {
		t.Errorf("after 3 failures: state = %v, want open", cb.State())
	}
	// open 中は Allow=false。
	if cb.Allow() {
		t.Error("Allow should return false when open")
	}
}

// TestCB_SuccessResetsFailureCounter は closed 中の成功で failure counter がリセットされることを確認する。
func TestCB_SuccessResetsFailureCounter(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 3, HalfOpenAfter: 1 * time.Second})
	cb.RecordFailure()
	cb.RecordFailure()
	// success で counter が 0 に戻る。
	cb.RecordSuccess()
	// もう 2 回失敗しても open にならない（counter が 0 から開始）。
	cb.RecordFailure()
	cb.RecordFailure()
	if cb.State() != CBClosed {
		t.Errorf("state = %v, want closed (success should reset counter)", cb.State())
	}
}

// TestCB_OpenToHalfOpenAfterTimeout は HalfOpenAfter 経過で半 open に遷移することを確認する。
func TestCB_OpenToHalfOpenAfterTimeout(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 1, HalfOpenAfter: 50 * time.Millisecond, HalfOpenMaxProbes: 1})
	cb.RecordFailure()
	if cb.State() != CBOpen {
		t.Fatalf("expected open after 1 failure")
	}
	// HalfOpenAfter 経過まで待つ。
	time.Sleep(80 * time.Millisecond)
	// 状態を再評価すると half-open に遷移しているはず。
	if cb.State() != CBHalfOpen {
		t.Errorf("state = %v, want half-open after timeout", cb.State())
	}
}

// TestCB_HalfOpenSuccessReturnsToClose は half-open での probe 成功で closed に戻ることを確認する。
func TestCB_HalfOpenSuccessReturnsToClose(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 1, HalfOpenAfter: 50 * time.Millisecond, HalfOpenMaxProbes: 1})
	cb.RecordFailure()
	time.Sleep(80 * time.Millisecond)
	// half-open に遷移、probe を許可する。
	if !cb.Allow() {
		t.Fatal("Allow should permit probe in half-open")
	}
	cb.RecordSuccess()
	if cb.State() != CBClosed {
		t.Errorf("state = %v, want closed after probe success", cb.State())
	}
}

// TestCB_HalfOpenFailureReturnsToOpen は half-open での probe 失敗で open に戻ることを確認する。
func TestCB_HalfOpenFailureReturnsToOpen(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 1, HalfOpenAfter: 50 * time.Millisecond, HalfOpenMaxProbes: 1})
	cb.RecordFailure()
	time.Sleep(80 * time.Millisecond)
	if !cb.Allow() {
		t.Fatal("Allow should permit probe in half-open")
	}
	cb.RecordFailure()
	if cb.State() != CBOpen {
		t.Errorf("state = %v, want open after probe failure", cb.State())
	}
}

// TestCB_HalfOpenLimitsProbes は half-open 中の probe 数が MaxProbes に制限されることを確認する。
func TestCB_HalfOpenLimitsProbes(t *testing.T) {
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 1, HalfOpenAfter: 50 * time.Millisecond, HalfOpenMaxProbes: 2})
	cb.RecordFailure()
	time.Sleep(80 * time.Millisecond)
	// 1 件目 / 2 件目は許可。
	if !cb.Allow() {
		t.Fatal("probe 1 should be allowed")
	}
	if !cb.Allow() {
		t.Fatal("probe 2 should be allowed")
	}
	// 3 件目は拒否。
	if cb.Allow() {
		t.Error("probe 3 should be rejected (exceeded MaxProbes)")
	}
}

// TestCB_StateObserverCalledOnTransition は observer が遷移時に呼ばれることを確認する。
func TestCB_StateObserverCalledOnTransition(t *testing.T) {
	var mu sync.Mutex
	transitions := []CBState{}
	cb := NewCircuitBreaker("test", CBConfig{FailureThreshold: 1, HalfOpenAfter: 50 * time.Millisecond})
	cb.SetObserver(func(_ string, s CBState) {
		mu.Lock()
		transitions = append(transitions, s)
		mu.Unlock()
	})
	cb.RecordFailure() // closed → open
	time.Sleep(80 * time.Millisecond)
	cb.State()         // open → half-open
	cb.Allow()         // 許可
	cb.RecordSuccess() // half-open → closed
	mu.Lock()
	defer mu.Unlock()
	if len(transitions) != 3 {
		t.Fatalf("expected 3 transitions, got %d: %v", len(transitions), transitions)
	}
	want := []CBState{CBOpen, CBHalfOpen, CBClosed}
	for i, w := range want {
		if transitions[i] != w {
			t.Errorf("transitions[%d] = %v, want %v", i, transitions[i], w)
		}
	}
}

// TestCBState_String は CBState の表示文字列を確認する。
func TestCBState_String(t *testing.T) {
	cases := map[CBState]string{
		CBClosed:   "closed",
		CBOpen:     "open",
		CBHalfOpen: "half-open",
		CBState(99): "unknown",
	}
	for s, want := range cases {
		if got := s.String(); got != want {
			t.Errorf("State(%d).String() = %q, want %q", s, got, want)
		}
	}
}

// TestCBRegistry_GetReusesInstance は同 name で同 instance を返すことを確認する。
func TestCBRegistry_GetReusesInstance(t *testing.T) {
	r := NewCircuitBreakerRegistry(DefaultCBConfig(), nil)
	a := r.Get("svc-a")
	b := r.Get("svc-a")
	if a != b {
		t.Error("Get should return same instance for same name")
	}
	c := r.Get("svc-b")
	if a == c {
		t.Error("Get should return different instances for different names")
	}
}

// TestLoadCBConfigFromEnv は env からの読込を確認する。
func TestLoadCBConfigFromEnv(t *testing.T) {
	env := map[string]string{
		"TIER1_CB_FAILURE_THRESHOLD": "10",
		"TIER1_CB_HALF_OPEN_AFTER":   "1m",
		"TIER1_CB_HALF_OPEN_PROBES":  "3",
	}
	cfg := LoadCBConfigFromEnv(func(k string) string { return env[k] }, nil)
	if cfg.FailureThreshold != 10 {
		t.Errorf("FailureThreshold = %d, want 10", cfg.FailureThreshold)
	}
	if cfg.HalfOpenAfter != time.Minute {
		t.Errorf("HalfOpenAfter = %v, want 1m", cfg.HalfOpenAfter)
	}
	if cfg.HalfOpenMaxProbes != 3 {
		t.Errorf("HalfOpenMaxProbes = %d, want 3", cfg.HalfOpenMaxProbes)
	}
}

// TestLoadCBConfigFromEnv_FallsBackOnInvalid は不正値で既定値を使うことを確認する。
func TestLoadCBConfigFromEnv_FallsBackOnInvalid(t *testing.T) {
	env := map[string]string{
		"TIER1_CB_FAILURE_THRESHOLD": "not-a-number",
		"TIER1_CB_HALF_OPEN_AFTER":   "garbage",
		"TIER1_CB_HALF_OPEN_PROBES":  "-5",
	}
	cfg := LoadCBConfigFromEnv(func(k string) string { return env[k] }, nil)
	def := DefaultCBConfig()
	if cfg.FailureThreshold != def.FailureThreshold {
		t.Errorf("FailureThreshold = %d, want default %d", cfg.FailureThreshold, def.FailureThreshold)
	}
	if cfg.HalfOpenAfter != def.HalfOpenAfter {
		t.Errorf("HalfOpenAfter = %v, want default %v", cfg.HalfOpenAfter, def.HalfOpenAfter)
	}
	if cfg.HalfOpenMaxProbes != def.HalfOpenMaxProbes {
		t.Errorf("HalfOpenMaxProbes = %d, want default %d", cfg.HalfOpenMaxProbes, def.HalfOpenMaxProbes)
	}
}
