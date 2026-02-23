package auditclient

import (
	"context"
	"sync"
	"time"
)

// AuditEvent は監査イベント。
type AuditEvent struct {
	ID           string    `json:"id"`
	TenantID     string    `json:"tenant_id"`
	ActorID      string    `json:"actor_id"`
	Action       string    `json:"action"`
	ResourceType string    `json:"resource_type"`
	ResourceID   string    `json:"resource_id"`
	Timestamp    time.Time `json:"timestamp"`
}

// AuditClient は監査クライアントのインターフェース。
type AuditClient interface {
	Record(ctx context.Context, event AuditEvent) error
	Flush(ctx context.Context) ([]AuditEvent, error)
}

// BufferedClient はバッファ付き監査クライアント。
type BufferedClient struct {
	mu  sync.Mutex
	buf []AuditEvent
}

// NewBufferedClient は新しい BufferedClient を生成する。
func NewBufferedClient() *BufferedClient {
	return &BufferedClient{}
}

func (c *BufferedClient) Record(_ context.Context, event AuditEvent) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.buf = append(c.buf, event)
	return nil
}

func (c *BufferedClient) Flush(_ context.Context) ([]AuditEvent, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	events := make([]AuditEvent, len(c.buf))
	copy(events, c.buf)
	c.buf = nil
	return events, nil
}
