package buildingblocks

import "context"

// StateEntry represents a stored state value.
type StateEntry struct {
	Key   string
	Value []byte
	ETag  string
}

// StateStore provides state management with optimistic concurrency via ETags.
type StateStore interface {
	Component
	Get(ctx context.Context, key string) (*StateEntry, error)
	Set(ctx context.Context, key string, value []byte, etag *string) (string, error)
	Delete(ctx context.Context, key string, etag *string) error
	BulkGet(ctx context.Context, keys []string) ([]StateEntry, error)
	BulkSet(ctx context.Context, entries []StateEntry) ([]string, error)
}
