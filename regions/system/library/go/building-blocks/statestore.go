package buildingblocks

import "context"

// ETag represents an optimistic concurrency token.
type ETag struct {
	Value string `json:"value"`
}

// StateEntry represents a stored state value.
type StateEntry struct {
	Key   string `json:"key"`
	Value []byte `json:"value"`
	ETag  *ETag  `json:"etag,omitempty"`
}

// SetRequest represents a request to set state.
type SetRequest struct {
	Key   string `json:"key"`
	Value []byte `json:"value"`
	ETag  *ETag  `json:"etag,omitempty"`
}

// StateStore provides state management with optimistic concurrency via ETags.
type StateStore interface {
	Component
	Get(ctx context.Context, key string) (*StateEntry, error)
	Set(ctx context.Context, req *SetRequest) (*ETag, error)
	Delete(ctx context.Context, key string, etag *ETag) error
	BulkGet(ctx context.Context, keys []string) ([]*StateEntry, error)
	BulkSet(ctx context.Context, requests []*SetRequest) ([]*ETag, error)
}
