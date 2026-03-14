package buildingblocks

import (
	"context"
	"fmt"
	"strconv"
	"sync"
	"sync/atomic"
)

type stateEntry struct {
	value []byte
	etag  string
}

// Compile-time interface compliance check.
var _ StateStore = (*InMemoryStateStore)(nil)

// InMemoryStateStore is an in-memory implementation of StateStore for testing.
type InMemoryStateStore struct {
	mu      sync.RWMutex
	entries map[string]*stateEntry
	counter atomic.Uint64
	status  ComponentStatus
}

// NewInMemoryStateStore creates a new InMemoryStateStore.
func NewInMemoryStateStore() *InMemoryStateStore {
	return &InMemoryStateStore{
		entries: make(map[string]*stateEntry),
		status:  StatusUninitialized,
	}
}

func (s *InMemoryStateStore) Name() string    { return "inmemory-statestore" }
func (s *InMemoryStateStore) Version() string { return "1.0.0" }

func (s *InMemoryStateStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

func (s *InMemoryStateStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

func (s *InMemoryStateStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

func (s *InMemoryStateStore) nextETag() string {
	return strconv.FormatUint(s.counter.Add(1), 10)
}

// Get returns the state entry for key, or nil if not found.
func (s *InMemoryStateStore) Get(_ context.Context, key string) (*StateEntry, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	e, ok := s.entries[key]
	if !ok {
		return nil, nil
	}
	return &StateEntry{Key: key, Value: e.value, ETag: &ETag{Value: e.etag}}, nil
}

// Set stores a value. If req.ETag is non-nil, it enforces optimistic concurrency.
func (s *InMemoryStateStore) Set(_ context.Context, req *SetRequest) (*ETag, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	existing, ok := s.entries[req.Key]
	if req.ETag != nil {
		if !ok {
			return nil, &ETagMismatchError{Key: req.Key, Expected: req.ETag, Actual: nil}
		}
		if existing.etag != req.ETag.Value {
			return nil, &ETagMismatchError{Key: req.Key, Expected: req.ETag, Actual: &ETag{Value: existing.etag}}
		}
	}
	newETag := s.nextETag()
	s.entries[req.Key] = &stateEntry{value: req.Value, etag: newETag}
	return &ETag{Value: newETag}, nil
}

// Delete removes the entry for key. If etag is non-nil, it enforces optimistic concurrency.
func (s *InMemoryStateStore) Delete(_ context.Context, key string, etag *ETag) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	existing, ok := s.entries[key]
	if !ok {
		return nil
	}
	if etag != nil && existing.etag != etag.Value {
		return &ETagMismatchError{Key: key, Expected: etag, Actual: &ETag{Value: existing.etag}}
	}
	delete(s.entries, key)
	return nil
}

// BulkGet retrieves multiple entries by key.
func (s *InMemoryStateStore) BulkGet(ctx context.Context, keys []string) ([]*StateEntry, error) {
	results := make([]*StateEntry, 0, len(keys))
	for _, key := range keys {
		entry, err := s.Get(ctx, key)
		if err != nil {
			return nil, fmt.Errorf("bulk get key %q: %w", key, err)
		}
		results = append(results, entry)
	}
	return results, nil
}

// BulkSet stores multiple entries in sequence.
func (s *InMemoryStateStore) BulkSet(ctx context.Context, requests []*SetRequest) ([]*ETag, error) {
	etags := make([]*ETag, 0, len(requests))
	for _, req := range requests {
		etag, err := s.Set(ctx, req)
		if err != nil {
			return nil, fmt.Errorf("bulk set key %q: %w", req.Key, err)
		}
		etags = append(etags, etag)
	}
	return etags, nil
}
