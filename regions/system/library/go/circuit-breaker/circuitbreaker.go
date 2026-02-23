package circuitbreaker

import (
	"errors"
	"sync"
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

// CircuitBreaker はサーキットブレーカー。
type CircuitBreaker struct {
	mu             sync.Mutex
	config         Config
	state          State
	failureCount   uint32
	successCount   uint32
	lastFailureAt  time.Time
}

// New は新しい CircuitBreaker を生成する。
func New(cfg Config) *CircuitBreaker {
	return &CircuitBreaker{
		config: cfg,
		state:  StateClosed,
	}
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
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.checkStateTransition()

	switch cb.state {
	case StateClosed:
		cb.failureCount = 0
	case StateHalfOpen:
		cb.successCount++
		if cb.successCount >= cb.config.SuccessThreshold {
			cb.state = StateClosed
			cb.failureCount = 0
			cb.successCount = 0
		}
	}
}

// RecordFailure は失敗を記録する。
func (cb *CircuitBreaker) RecordFailure() {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	cb.checkStateTransition()

	switch cb.state {
	case StateClosed:
		cb.failureCount++
		cb.lastFailureAt = time.Now()
		if cb.failureCount >= cb.config.FailureThreshold {
			cb.state = StateOpen
		}
	case StateHalfOpen:
		cb.state = StateOpen
		cb.lastFailureAt = time.Now()
		cb.successCount = 0
	}
}

// Call は関数を呼び出す。Open状態の場合は ErrOpen を返す。
func (cb *CircuitBreaker) Call(fn func() error) error {
	cb.mu.Lock()
	cb.checkStateTransition()
	if cb.state == StateOpen {
		cb.mu.Unlock()
		return ErrOpen
	}
	cb.mu.Unlock()

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
	}
}
