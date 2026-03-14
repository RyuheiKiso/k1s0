package buildingblocks

import (
	"context"
	"fmt"
	"sync"
)

// Compile-time interface compliance check.
var _ SecretStore = (*InMemorySecretStore)(nil)

// InMemorySecretStore is an in-memory implementation of SecretStore for testing.
type InMemorySecretStore struct {
	mu      sync.RWMutex
	secrets map[string]*Secret
	status  ComponentStatus
}

// NewInMemorySecretStore creates a new InMemorySecretStore.
func NewInMemorySecretStore() *InMemorySecretStore {
	return &InMemorySecretStore{
		secrets: make(map[string]*Secret),
		status:  StatusUninitialized,
	}
}

func (s *InMemorySecretStore) Name() string    { return "inmemory-secretstore" }
func (s *InMemorySecretStore) Version() string { return "1.0.0" }

func (s *InMemorySecretStore) Init(_ context.Context, _ Metadata) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusReady
	return nil
}

func (s *InMemorySecretStore) Close(_ context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = StatusClosed
	return nil
}

func (s *InMemorySecretStore) Status(_ context.Context) ComponentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

// Get retrieves a secret by key.
func (s *InMemorySecretStore) Get(_ context.Context, key string) (*Secret, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	secret, ok := s.secrets[key]
	if !ok {
		return nil, NewComponentError("inmemory-secretstore", "Get", fmt.Sprintf("secret %q not found", key), nil)
	}
	return secret, nil
}

// BulkGet retrieves multiple secrets by key.
func (s *InMemorySecretStore) BulkGet(ctx context.Context, keys []string) ([]*Secret, error) {
	results := make([]*Secret, 0, len(keys))
	for _, key := range keys {
		secret, err := s.Get(ctx, key)
		if err != nil {
			return nil, err
		}
		results = append(results, secret)
	}
	return results, nil
}

// Put stores a secret. This is a test helper not part of the SecretStore interface.
func (s *InMemorySecretStore) Put(key, value string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.secrets[key] = &Secret{Key: key, Value: value}
}
