package retry

import (
	"fmt"
	"sync"
	"time"
)

// CircuitBreakerState はサーキットブレーカーの状態。
type CircuitBreakerState int

const (
	// StateClosed は正常状態。
	StateClosed CircuitBreakerState = iota
	// StateOpen はオープン状態（リクエストを拒否）。
	StateOpen
	// StateHalfOpen はハーフオープン状態（テストリクエストを許可）。
	StateHalfOpen
)

func (s CircuitBreakerState) String() string {
	switch s {
	case StateClosed:
		return "Closed"
	case StateOpen:
		return "Open"
	case StateHalfOpen:
		return "HalfOpen"
	default:
		return "Unknown"
	}
}

// CircuitBreakerConfig はサーキットブレーカーの設定。
type CircuitBreakerConfig struct {
	// FailureThreshold はオープンに遷移する失敗回数閾値。
	FailureThreshold int
	// SuccessThreshold はハーフオープンからクローズに遷移する成功回数閾値。
	SuccessThreshold int
	// Timeout はオープンからハーフオープンに遷移するまでの待機時間。
	Timeout time.Duration
}

// DefaultCircuitBreakerConfig はデフォルトのサーキットブレーカー設定を返す。
func DefaultCircuitBreakerConfig() *CircuitBreakerConfig {
	return &CircuitBreakerConfig{
		FailureThreshold: 5,
		SuccessThreshold: 2,
		Timeout:          30 * time.Second,
	}
}

// CircuitBreakerOpenError はサーキットブレーカーがオープン状態のときのエラー。
type CircuitBreakerOpenError struct{}

func (e *CircuitBreakerOpenError) Error() string {
	return fmt.Sprintf("サーキットブレーカーがオープン状態です")
}

// CircuitBreaker はサーキットブレーカーパターンの実装。
type CircuitBreaker struct {
	config       *CircuitBreakerConfig
	mu           sync.Mutex
	state        CircuitBreakerState
	failureCount int
	successCount int
	openedAt     time.Time
}

// NewCircuitBreaker は新しいサーキットブレーカーを生成する。
func NewCircuitBreaker(config *CircuitBreakerConfig) *CircuitBreaker {
	return &CircuitBreaker{
		config: config,
		state:  StateClosed,
	}
}

// IsOpen はサーキットブレーカーがオープン状態かどうかを返す。
// オープン状態でタイムアウトを過ぎた場合、ハーフオープンに遷移して false を返す。
func (cb *CircuitBreaker) IsOpen() bool {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	if cb.state == StateOpen {
		if time.Since(cb.openedAt) >= cb.config.Timeout {
			cb.state = StateHalfOpen
			cb.successCount = 0
			return false
		}
		return true
	}
	return false
}

// RecordSuccess は成功を記録する。
func (cb *CircuitBreaker) RecordSuccess() {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	switch cb.state {
	case StateHalfOpen:
		cb.successCount++
		if cb.successCount >= cb.config.SuccessThreshold {
			cb.state = StateClosed
			cb.failureCount = 0
			cb.successCount = 0
		}
	case StateClosed:
		cb.failureCount = 0
	}
}

// RecordFailure は失敗を記録する。
func (cb *CircuitBreaker) RecordFailure() {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	cb.failureCount++
	if cb.failureCount >= cb.config.FailureThreshold {
		cb.state = StateOpen
		cb.openedAt = time.Now()
		cb.failureCount = 0
	}
}

// GetState は現在の状態を返す。
func (cb *CircuitBreaker) GetState() CircuitBreakerState {
	cb.mu.Lock()
	defer cb.mu.Unlock()
	return cb.state
}
