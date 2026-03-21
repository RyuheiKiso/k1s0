package circuitbreaker

import (
	"errors"
	"sync"
	"sync/atomic"
	"time"
)

// State はサーキットブレーカーの状態。
type State int

const (
	StateClosed   State = iota
	StateOpen
	StateHalfOpen
)

// ErrOpen はサーキットブレーカーがOpen状態のときのエラー。
var ErrOpen = errors.New("サーキットブレーカーがOpen状態です")

// Config はサーキットブレーカーの設定。
type Config struct {
	FailureThreshold uint32
	SuccessThreshold uint32
	Timeout          time.Duration
}

// Metrics はサーキットブレーカーのメトリクススナップショット。
type Metrics struct {
	// SuccessCount は成功回数の累計。
	SuccessCount uint64
	// FailureCount は失敗回数の累計。
	FailureCount uint64
	// State は現在の状態文字列（"Closed" / "Open" / "HalfOpen"）。
	State string
}

// metricsRecorder はサーキットブレーカーのメトリクスをアトミックに記録する。
type metricsRecorder struct {
	successCount atomic.Uint64
	failureCount atomic.Uint64
	stateCode    atomic.Int64
}

// recordSuccess は成功カウンタをインクリメントする。
func (m *metricsRecorder) recordSuccess() {
	m.successCount.Add(1)
}

// recordFailure は失敗カウンタをインクリメントする。
func (m *metricsRecorder) recordFailure() {
	m.failureCount.Add(1)
}

// setState は現在の状態コードを設定する。
func (m *metricsRecorder) setState(s State) {
	m.stateCode.Store(int64(s))
}

// snapshot は現在のメトリクスのスナップショットを返す。
func (m *metricsRecorder) snapshot() Metrics {
	code := m.stateCode.Load()
	var stateName string
	switch State(code) {
	case StateOpen:
		stateName = "Open"
	case StateHalfOpen:
		stateName = "HalfOpen"
	default:
		stateName = "Closed"
	}
	return Metrics{
		SuccessCount: m.successCount.Load(),
		FailureCount: m.failureCount.Load(),
		State:        stateName,
	}
}

// CircuitBreaker はサーキットブレーカー。
// halfOpenInFlight は HalfOpen 状態で同時に通過できるリクエストを1件に制限するためのアトミックフラグ。
type CircuitBreaker struct {
	mu                sync.Mutex
	config            Config
	state             State
	failureCount      uint32
	successCount      uint32
	lastFailureAt     time.Time
	metrics           *metricsRecorder
	halfOpenInFlight  int32
}

// New は新しい CircuitBreaker を生成する。
func New(cfg Config) *CircuitBreaker {
	cb := &CircuitBreaker{
		config:  cfg,
		state:   StateClosed,
		metrics: &metricsRecorder{},
	}
	// 初期状態を Closed として記録する
	cb.metrics.setState(StateClosed)
	return cb
}

// State は現在の状態を返す。
func (cb *CircuitBreaker) State() State {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.checkStateTransition()
	return cb.state
}

// IsOpen はOpen状態かどうかを返す。
func (cb *CircuitBreaker) IsOpen() bool {
	return cb.State() == StateOpen
}

// RecordSuccess は成功を記録する。
func (cb *CircuitBreaker) RecordSuccess() {
	cb.metrics.recordSuccess()
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.checkStateTransition()

	switch cb.state {
	case StateClosed:
		// Closed 状態では成功時に失敗カウントをリセットする
		cb.failureCount = 0
	case StateHalfOpen:
		cb.successCount++
		if cb.successCount >= cb.config.SuccessThreshold {
			cb.state = StateClosed
			cb.failureCount = 0
			cb.successCount = 0
			cb.metrics.setState(StateClosed)
		}
	}
}

// RecordFailure は失敗を記録する。
func (cb *CircuitBreaker) RecordFailure() {
	cb.metrics.recordFailure()
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.checkStateTransition()

	switch cb.state {
	case StateClosed:
		cb.failureCount++
		cb.lastFailureAt = time.Now()
		if cb.failureCount >= cb.config.FailureThreshold {
			cb.state = StateOpen
			cb.metrics.setState(StateOpen)
		}
	case StateHalfOpen:
		// HalfOpen 状態での失敗は即座に Open へ再遷移する
		cb.state = StateOpen
		cb.lastFailureAt = time.Now()
		cb.successCount = 0
		cb.metrics.setState(StateOpen)
	}
}

// Metrics は現在のメトリクススナップショットを返す。
func (cb *CircuitBreaker) Metrics() Metrics {
	return cb.metrics.snapshot()
}

// Call は関数を呼び出す。Open状態の場合は ErrOpen を返す。
// HalfOpen状態では atomic CAS により同時に1件のリクエストのみ通過させ、
// 複数 goroutine が同時にプローブ呼び出しを行うことを防ぐ。
func (cb *CircuitBreaker) Call(fn func() error) error {
	cb.mu.Lock()
	cb.checkStateTransition()
	if cb.state == StateOpen {
		cb.mu.Unlock()
		return ErrOpen
	}
	// HalfOpen状態の場合は1件のみ通過させ、それ以外は ErrOpen を返す
	if cb.state == StateHalfOpen {
		if !atomic.CompareAndSwapInt32(&cb.halfOpenInFlight, 0, 1) {
			cb.mu.Unlock()
			return ErrOpen
		}
		cb.mu.Unlock()
		defer atomic.StoreInt32(&cb.halfOpenInFlight, 0)
	} else {
		cb.mu.Unlock()
	}

	err := fn()
	if err != nil {
		cb.RecordFailure()
		return err
	}
	cb.RecordSuccess()
	return nil
}

func (cb *CircuitBreaker) checkStateTransition() {
	if cb.state == StateOpen && time.Since(cb.lastFailureAt) >= cb.config.Timeout {
		cb.state = StateHalfOpen
		cb.successCount = 0
		cb.metrics.setState(StateHalfOpen)
	}
}
