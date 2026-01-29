package outbox

import "context"

// Store defines the persistence interface for outbox entries.
type Store interface {
	// Insert saves a new outbox entry.
	Insert(ctx context.Context, entry *Entry) error

	// FetchPending retrieves up to limit pending entries ordered by creation time.
	FetchPending(ctx context.Context, limit int) ([]*Entry, error)

	// MarkPublished marks an entry as successfully published.
	MarkPublished(ctx context.Context, id string) error

	// MarkFailed marks an entry as failed with an error message.
	MarkFailed(ctx context.Context, id string, errMsg string) error
}
