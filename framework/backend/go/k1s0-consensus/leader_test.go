package consensus

import (
	"testing"
	"time"
)

func TestLeaderLease_IsExpired(t *testing.T) {
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
			lease := &LeaderLease{ExpiresAt: tt.expiresAt}
			if got := lease.IsExpired(); got != tt.want {
				t.Errorf("IsExpired() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestLeaderEventType_String(t *testing.T) {
	tests := []struct {
		eventType LeaderEventType
		want      string
	}{
		{LeaderElected, "elected"},
		{LeaderLost, "lost"},
		{LeaderChanged, "changed"},
		{LeaderEventType(99), "unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.want, func(t *testing.T) {
			if got := tt.eventType.String(); got != tt.want {
				t.Errorf("String() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestLeaderConfig_Validate(t *testing.T) {
	t.Run("defaults applied", func(t *testing.T) {
		cfg := LeaderConfig{}
		cfg.Validate()

		if cfg.LeaseDuration != 15*time.Second {
			t.Errorf("LeaseDuration = %v, want 15s", cfg.LeaseDuration)
		}
		if cfg.RenewInterval != 5*time.Second {
			t.Errorf("RenewInterval = %v, want 5s", cfg.RenewInterval)
		}
		if cfg.TableName != "k1s0_leader_election" {
			t.Errorf("TableName = %q, want k1s0_leader_election", cfg.TableName)
		}
	})

	t.Run("renew interval capped", func(t *testing.T) {
		cfg := LeaderConfig{
			LeaseDuration: 9 * time.Second,
			RenewInterval: 10 * time.Second, // Greater than lease.
		}
		cfg.Validate()

		if cfg.RenewInterval != 3*time.Second {
			t.Errorf("RenewInterval = %v, want 3s (lease/3)", cfg.RenewInterval)
		}
	})
}
