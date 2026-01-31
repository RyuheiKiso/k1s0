package consensus

import (
	"testing"
	"time"
)

func TestLockGuard_IsExpired(t *testing.T) {
	tests := []struct {
		name      string
		expiresAt time.Time
		want      bool
	}{
		{"not expired", time.Now().Add(10 * time.Minute), false},
		{"expired", time.Now().Add(-1 * time.Minute), true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			guard := &LockGuard{ExpiresAt: tt.expiresAt}
			if got := guard.IsExpired(); got != tt.want {
				t.Errorf("IsExpired() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestLockGuard_Close(t *testing.T) {
	t.Run("calls release func once", func(t *testing.T) {
		called := 0
		guard := &LockGuard{
			releaseFunc: func() error {
				called++
				return nil
			},
		}

		if err := guard.Close(); err != nil {
			t.Errorf("Close() error: %v", err)
		}
		if called != 1 {
			t.Errorf("release func called %d times, want 1", called)
		}

		// Second call should be no-op.
		if err := guard.Close(); err != nil {
			t.Errorf("second Close() error: %v", err)
		}
		if called != 1 {
			t.Errorf("release func called %d times after second Close, want 1", called)
		}
	})

	t.Run("nil release func", func(t *testing.T) {
		guard := &LockGuard{}
		if err := guard.Close(); err != nil {
			t.Errorf("Close() with nil releaseFunc error: %v", err)
		}
	})
}

func TestLockConfig_Validate(t *testing.T) {
	cfg := LockConfig{}
	cfg.Validate()

	if cfg.DefaultTTL != 30*time.Second {
		t.Errorf("DefaultTTL = %v, want 30s", cfg.DefaultTTL)
	}
	if cfg.RetryInterval != 100*time.Millisecond {
		t.Errorf("RetryInterval = %v, want 100ms", cfg.RetryInterval)
	}
	if cfg.TableName != "k1s0_distributed_locks" {
		t.Errorf("TableName = %q, want k1s0_distributed_locks", cfg.TableName)
	}
	if cfg.KeyPrefix != "k1s0:lock:" {
		t.Errorf("KeyPrefix = %q, want k1s0:lock:", cfg.KeyPrefix)
	}
}
