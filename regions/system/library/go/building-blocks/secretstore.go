package buildingblocks

import "context"

// Secret represents a retrieved secret.
type Secret struct {
	Key      string            `json:"key"`
	Value    string            `json:"value"`
	Metadata map[string]string `json:"metadata,omitempty"`
}

// SecretStore provides unified secret retrieval.
type SecretStore interface {
	Component
	Get(ctx context.Context, key string) (*Secret, error)
	BulkGet(ctx context.Context, keys []string) ([]*Secret, error)
	Close(ctx context.Context) error
}
