package consensus

import (
	"context"
	"time"
)

// LeaderEventType describes the type of leader change event.
type LeaderEventType int

const (
	// LeaderElected indicates a new leader was elected.
	LeaderElected LeaderEventType = iota

	// LeaderLost indicates the current node lost leadership.
	LeaderLost

	// LeaderChanged indicates leadership transferred to a different node.
	LeaderChanged
)

// String returns a human-readable representation of the event type.
func (t LeaderEventType) String() string {
	switch t {
	case LeaderElected:
		return "elected"
	case LeaderLost:
		return "lost"
	case LeaderChanged:
		return "changed"
	default:
		return "unknown"
	}
}

// LeaderLease represents an active leader lease held by a node.
type LeaderLease struct {
	// Key is the lease key that identifies the leadership scope.
	Key string

	// HolderID is the unique identifier of the node holding the lease.
	HolderID string

	// FenceToken is a monotonically increasing token for fencing.
	FenceToken uint64

	// AcquiredAt is the time the lease was acquired.
	AcquiredAt time.Time

	// ExpiresAt is the time the lease expires if not renewed.
	ExpiresAt time.Time
}

// IsExpired returns true if the lease has expired.
func (l *LeaderLease) IsExpired() bool {
	return time.Now().After(l.ExpiresAt)
}

// LeaderEvent describes a leadership change.
type LeaderEvent struct {
	// Type is the kind of leader event.
	Type LeaderEventType

	// LeaseKey is the key for which leadership changed.
	LeaseKey string

	// LeaderID is the current leader's node ID. Empty if no leader.
	LeaderID string

	// FenceToken is the current fence token.
	FenceToken uint64

	// Timestamp is when the event occurred.
	Timestamp time.Time
}

// LeaderElector defines the interface for leader election.
type LeaderElector interface {
	// TryAcquire attempts to acquire leadership for the given lease key.
	// Returns a LeaderLease if successful, or an error if leadership
	// could not be acquired (e.g., another node holds the lease).
	TryAcquire(ctx context.Context, leaseKey string) (*LeaderLease, error)

	// Renew extends the lease duration for an existing lease.
	// Returns true if the renewal was successful, false if the lease
	// was lost (e.g., expired before renewal).
	Renew(ctx context.Context, lease *LeaderLease) (bool, error)

	// Release voluntarily relinquishes leadership.
	Release(ctx context.Context, lease *LeaderLease) error

	// CurrentLeader returns the node ID of the current leader for the
	// given lease key. Returns an empty string if there is no leader.
	CurrentLeader(ctx context.Context, leaseKey string) (string, error)

	// Watch returns a channel that emits LeaderEvent values whenever
	// leadership changes for the given lease key. The channel is closed
	// when the context is canceled.
	Watch(ctx context.Context, leaseKey string) (<-chan LeaderEvent, error)
}
