package consensus

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// DbLeaderElector implements LeaderElector using PostgreSQL.
//
// It uses a table with INSERT ON CONFLICT for atomic lease acquisition,
// UPDATE for renewal, and DELETE for release. A background heartbeat
// goroutine can be started to automatically renew leases.
//
// Required table schema (created automatically if not present):
//
//	CREATE TABLE IF NOT EXISTS k1s0_leader_election (
//	    lease_key   TEXT PRIMARY KEY,
//	    holder_id   TEXT NOT NULL,
//	    fence_token BIGINT NOT NULL DEFAULT 1,
//	    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//	    expires_at  TIMESTAMPTZ NOT NULL
//	);
type DbLeaderElector struct {
	pool   *pgxpool.Pool
	config LeaderConfig

	mu        sync.Mutex
	stopFuncs map[string]context.CancelFunc
}

// NewDbLeaderElector creates a new PostgreSQL-based leader elector.
func NewDbLeaderElector(pool *pgxpool.Pool, config LeaderConfig) *DbLeaderElector {
	config.Validate()
	return &DbLeaderElector{
		pool:      pool,
		config:    config,
		stopFuncs: make(map[string]context.CancelFunc),
	}
}

// EnsureTable creates the leader election table if it does not exist.
func (e *DbLeaderElector) EnsureTable(ctx context.Context) error {
	sql := fmt.Sprintf(`CREATE TABLE IF NOT EXISTS %s (
		lease_key   TEXT PRIMARY KEY,
		holder_id   TEXT NOT NULL,
		fence_token BIGINT NOT NULL DEFAULT 1,
		acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
		expires_at  TIMESTAMPTZ NOT NULL
	)`, e.config.TableName)
	_, err := e.pool.Exec(ctx, sql)
	if err != nil {
		return fmt.Errorf("consensus: failed to create leader election table: %w", err)
	}
	return nil
}

// TryAcquire attempts to acquire leadership for the given lease key.
func (e *DbLeaderElector) TryAcquire(ctx context.Context, leaseKey string) (*LeaderLease, error) {
	now := time.Now()
	expiresAt := now.Add(e.config.LeaseDuration)

	// INSERT a new lease, or if an existing lease is expired, take over
	// with an incremented fence token.
	sql := fmt.Sprintf(`
		INSERT INTO %s (lease_key, holder_id, fence_token, acquired_at, expires_at)
		VALUES ($1, $2, 1, $3, $4)
		ON CONFLICT (lease_key) DO UPDATE
		SET holder_id   = EXCLUDED.holder_id,
		    fence_token = %s.fence_token + 1,
		    acquired_at = EXCLUDED.acquired_at,
		    expires_at  = EXCLUDED.expires_at
		WHERE %s.expires_at < $3 OR %s.holder_id = $2
		RETURNING fence_token, acquired_at, expires_at
	`, e.config.TableName, e.config.TableName, e.config.TableName, e.config.TableName)

	var lease LeaderLease
	lease.Key = leaseKey
	lease.HolderID = e.config.NodeID

	err := e.pool.QueryRow(ctx, sql, leaseKey, e.config.NodeID, now, expiresAt).
		Scan(&lease.FenceToken, &lease.AcquiredAt, &lease.ExpiresAt)
	if err != nil {
		if err == pgx.ErrNoRows {
			// Another node holds a valid lease.
			return nil, fmt.Errorf("consensus: lease held by another node: %w", ErrNotLeader)
		}
		return nil, fmt.Errorf("consensus: failed to acquire lease: %w", err)
	}

	metricsLeaderElections.WithLabelValues(leaseKey, "acquired").Inc()
	return &lease, nil
}

// Renew extends the lease duration for an existing lease.
func (e *DbLeaderElector) Renew(ctx context.Context, lease *LeaderLease) (bool, error) {
	if lease == nil {
		return false, fmt.Errorf("consensus: nil lease: %w", ErrLeaseExpired)
	}

	newExpiry := time.Now().Add(e.config.LeaseDuration)

	sql := fmt.Sprintf(`
		UPDATE %s
		SET expires_at = $1
		WHERE lease_key = $2
		  AND holder_id = $3
		  AND fence_token = $4
		  AND expires_at > NOW()
	`, e.config.TableName)

	tag, err := e.pool.Exec(ctx, sql, newExpiry, lease.Key, lease.HolderID, lease.FenceToken)
	if err != nil {
		return false, fmt.Errorf("consensus: failed to renew lease: %w", err)
	}

	if tag.RowsAffected() == 0 {
		metricsLeaderElections.WithLabelValues(lease.Key, "lost").Inc()
		return false, nil
	}

	lease.ExpiresAt = newExpiry
	metricsLeaderRenewals.WithLabelValues(lease.Key).Inc()
	return true, nil
}

// Release voluntarily relinquishes leadership.
func (e *DbLeaderElector) Release(ctx context.Context, lease *LeaderLease) error {
	if lease == nil {
		return fmt.Errorf("consensus: nil lease: %w", ErrLeaseExpired)
	}

	sql := fmt.Sprintf(`
		DELETE FROM %s
		WHERE lease_key = $1
		  AND holder_id = $2
		  AND fence_token = $3
	`, e.config.TableName)

	tag, err := e.pool.Exec(ctx, sql, lease.Key, lease.HolderID, lease.FenceToken)
	if err != nil {
		return fmt.Errorf("consensus: failed to release lease: %w", err)
	}

	if tag.RowsAffected() == 0 {
		return fmt.Errorf("consensus: lease not held or already expired: %w", ErrLockNotHeld)
	}

	metricsLeaderElections.WithLabelValues(lease.Key, "released").Inc()

	// Stop any background heartbeat for this lease.
	e.mu.Lock()
	if cancel, ok := e.stopFuncs[lease.Key]; ok {
		cancel()
		delete(e.stopFuncs, lease.Key)
	}
	e.mu.Unlock()

	return nil
}

// CurrentLeader returns the node ID of the current leader.
func (e *DbLeaderElector) CurrentLeader(ctx context.Context, leaseKey string) (string, error) {
	sql := fmt.Sprintf(`
		SELECT holder_id FROM %s
		WHERE lease_key = $1 AND expires_at > NOW()
	`, e.config.TableName)

	var holderID string
	err := e.pool.QueryRow(ctx, sql, leaseKey).Scan(&holderID)
	if err != nil {
		if err == pgx.ErrNoRows {
			return "", nil
		}
		return "", fmt.Errorf("consensus: failed to query current leader: %w", err)
	}
	return holderID, nil
}

// Watch returns a channel that emits LeaderEvent values on leadership changes.
// The channel is closed when the context is canceled.
func (e *DbLeaderElector) Watch(ctx context.Context, leaseKey string) (<-chan LeaderEvent, error) {
	ch := make(chan LeaderEvent, 8)

	go func() {
		defer close(ch)
		var lastLeader string

		ticker := time.NewTicker(e.config.WatchPollInterval)
		defer ticker.Stop()

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				leader, err := e.CurrentLeader(ctx, leaseKey)
				if err != nil {
					continue
				}

				if leader != lastLeader {
					event := LeaderEvent{
						LeaseKey:  leaseKey,
						LeaderID:  leader,
						Timestamp: time.Now(),
					}

					if lastLeader == "" && leader != "" {
						event.Type = LeaderElected
					} else if lastLeader != "" && leader == "" {
						event.Type = LeaderLost
					} else {
						event.Type = LeaderChanged
					}

					lastLeader = leader

					select {
					case ch <- event:
					case <-ctx.Done():
						return
					}
				}
			}
		}
	}()

	return ch, nil
}

// StartHeartbeat starts a background goroutine that periodically renews
// the given lease. It stops when the context is canceled or the lease
// cannot be renewed. Returns immediately.
func (e *DbLeaderElector) StartHeartbeat(ctx context.Context, lease *LeaderLease) {
	hbCtx, cancel := context.WithCancel(ctx)

	e.mu.Lock()
	e.stopFuncs[lease.Key] = cancel
	e.mu.Unlock()

	go func() {
		ticker := time.NewTicker(e.config.RenewInterval)
		defer ticker.Stop()

		for {
			select {
			case <-hbCtx.Done():
				return
			case <-ticker.C:
				renewed, err := e.Renew(hbCtx, lease)
				if err != nil || !renewed {
					cancel()
					return
				}
			}
		}
	}()
}
