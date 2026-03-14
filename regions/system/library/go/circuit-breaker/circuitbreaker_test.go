package circuitbreaker_test

import (
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-circuit-breaker"
	"github.com/stretchr/testify/assert"
)

func defaultConfig() circuitbreaker.Config {
	return circuitbreaker.Config{
		FailureThreshold: 3,
		SuccessThreshold: 2,
		Timeout:          100 * time.Millisecond,
	}
}

// サーキットブレーカーの初期状態がClosedであることを確認する。
func TestInitialState_Closed(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	assert.Equal(t, circuitbreaker.StateClosed, cb.State())
	assert.False(t, cb.IsOpen())
}

// 失敗回数がしきい値に達したときにClosedからOpenへ遷移することを確認する。
func TestClosed_ToOpen_OnFailureThreshold(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	assert.Equal(t, circuitbreaker.StateOpen, cb.State())
	assert.True(t, cb.IsOpen())
}

// タイムアウト経過後にOpenからHalfOpenへ遷移することを確認する。
func TestOpen_ToHalfOpen_AfterTimeout(t *testing.T) {
	cfg := defaultConfig()
	cfg.Timeout = 50 * time.Millisecond
	cb := circuitbreaker.New(cfg)

	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	assert.Equal(t, circuitbreaker.StateOpen, cb.State())

	time.Sleep(60 * time.Millisecond)
	assert.Equal(t, circuitbreaker.StateHalfOpen, cb.State())
}

// HalfOpen状態で成功回数がしきい値に達したときにClosedへ遷移することを確認する。
func TestHalfOpen_ToClosed_OnSuccessThreshold(t *testing.T) {
	cfg := defaultConfig()
	cfg.Timeout = 50 * time.Millisecond
	cb := circuitbreaker.New(cfg)

	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	time.Sleep(60 * time.Millisecond)

	cb.RecordSuccess()
	assert.Equal(t, circuitbreaker.StateHalfOpen, cb.State())
	cb.RecordSuccess()
	assert.Equal(t, circuitbreaker.StateClosed, cb.State())
}

// HalfOpen状態で失敗が発生したときに再びOpenへ遷移することを確認する。
func TestHalfOpen_ToOpen_OnFailure(t *testing.T) {
	cfg := defaultConfig()
	cfg.Timeout = 50 * time.Millisecond
	cb := circuitbreaker.New(cfg)

	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	time.Sleep(60 * time.Millisecond)

	cb.RecordFailure()
	assert.Equal(t, circuitbreaker.StateOpen, cb.State())
}

// Open状態でCallを呼んだ場合にErrOpenが返ることを確認する。
func TestCall_Open_ReturnsErrOpen(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	err := cb.Call(func() error { return nil })
	assert.ErrorIs(t, err, circuitbreaker.ErrOpen)
}

// Closed状態でCallが成功した場合にエラーなしで完了することを確認する。
func TestCall_Success(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	err := cb.Call(func() error { return nil })
	assert.NoError(t, err)
}

// Callがエラーを返す関数を実行した場合にそのエラーが伝播することを確認する。
func TestCall_Failure(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	testErr := errors.New("fail")
	err := cb.Call(func() error { return testErr })
	assert.ErrorIs(t, err, testErr)
}

// 成功を記録すると失敗カウントがリセットされ、その後の失敗でOpenにならないことを確認する。
func TestSuccess_ResetFailureCount(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	cb.RecordFailure()
	cb.RecordFailure()
	cb.RecordSuccess()
	// Should not be open after 2 failures + 1 success + 1 failure
	cb.RecordFailure()
	assert.Equal(t, circuitbreaker.StateClosed, cb.State())
}
