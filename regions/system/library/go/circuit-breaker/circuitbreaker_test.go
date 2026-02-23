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

func TestInitialState_Closed(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	assert.Equal(t, circuitbreaker.StateClosed, cb.State())
	assert.False(t, cb.IsOpen())
}

func TestClosed_ToOpen_OnFailureThreshold(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	assert.Equal(t, circuitbreaker.StateOpen, cb.State())
	assert.True(t, cb.IsOpen())
}

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

func TestCall_Open_ReturnsErrOpen(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	err := cb.Call(func() error { return nil })
	assert.ErrorIs(t, err, circuitbreaker.ErrOpen)
}

func TestCall_Success(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	err := cb.Call(func() error { return nil })
	assert.NoError(t, err)
}

func TestCall_Failure(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	testErr := errors.New("fail")
	err := cb.Call(func() error { return testErr })
	assert.ErrorIs(t, err, testErr)
}

func TestSuccess_ResetFailureCount(t *testing.T) {
	cb := circuitbreaker.New(defaultConfig())
	cb.RecordFailure()
	cb.RecordFailure()
	cb.RecordSuccess()
	// Should not be open after 2 failures + 1 success + 1 failure
	cb.RecordFailure()
	assert.Equal(t, circuitbreaker.StateClosed, cb.State())
}
