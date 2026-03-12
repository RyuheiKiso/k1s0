package buildingblocks

import "context"

// SecretValue represents a retrieved secret.
type SecretValue struct {
	Key      string
	Value    string
	Metadata map[string]string
}

// SecretStore provides unified secret retrieval.
type SecretStore interface {
	Component
	GetSecret(ctx context.Context, key string) (*SecretValue, error)
	BulkGet(ctx context.Context, keys []string) (map[string]*SecretValue, error)
}
