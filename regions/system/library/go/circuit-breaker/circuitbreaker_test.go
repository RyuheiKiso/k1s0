package circuitbreaker_test

import (
	"errors"
	"sync"
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

// HalfOpen状態で複数goroutineが同時に Call を呼んだ場合、1件のみ通過し残りは ErrOpen を返すことを確認する。
func TestCall_HalfOpen_OnlyOneConcurrentRequest(t *testing.T) {
	cfg := defaultConfig()
	cfg.Timeout = 50 * time.Millisecond
	cb := circuitbreaker.New(cfg)

	// Open 状態に遷移させる
	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}
	// タイムアウト待機で HalfOpen へ
	time.Sleep(60 * time.Millisecond)
	assert.Equal(t, circuitbreaker.StateHalfOpen, cb.State())

	// 1件目の fn を実行中に2件目の Call が来るよう blocker で同期する。
	// blocker を受信するまで1件目の fn がブロックし、その間に2件目を試みる。
	blocker := make(chan struct{})
	firstStarted := make(chan struct{})

	var result1, result2 error
	var wg sync.WaitGroup

	// 1件目: fn の中でブロックして halfOpenInFlight=1 の状態を維持する
	wg.Add(1)
	go func() {
		defer wg.Done()
		result1 = cb.Call(func() error {
			close(firstStarted) // fn が始まったことを通知
			<-blocker           // 2件目の結果取得後に解放される
			return nil
		})
	}()

	// 1件目の fn が開始するまで待つ
	<-firstStarted

	// 2件目: 1件目実行中に試みる → ErrOpen になるはず
	result2 = cb.Call(func() error { return nil })

	// 1件目の fn を解放
	close(blocker)
	wg.Wait()

	// 1件目は成功、2件目は ErrOpen
	assert.NoError(t, result1, "1件目の Call は成功すること")
	assert.ErrorIs(t, result2, circuitbreaker.ErrOpen, "HalfOpen状態で2件目は ErrOpen を返すこと")
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
