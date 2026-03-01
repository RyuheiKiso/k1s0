package auditclient

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// AuditEvent は監査イベント。
type AuditEvent struct {
	ID           string                 `json:"id"`
	TenantID     string                 `json:"tenant_id"`
	ActorID      string                 `json:"actor_id"`
	Action       string                 `json:"action"`
	ResourceType string                 `json:"resource_type"`
	ResourceID   string                 `json:"resource_id"`
	Metadata     map[string]interface{} `json:"metadata"`
	Timestamp    time.Time              `json:"timestamp"`
}

// AuditErrorKind は監査エラーの種別。
type AuditErrorKind int

const (
	SerializationError AuditErrorKind = iota
	SendError
	InternalError
)

// AuditError は監査クライアントのエラー型。
type AuditError struct {
	Kind    AuditErrorKind
	Message string
	Err     error
}

func (e *AuditError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("AuditError(%d): %s: %v", e.Kind, e.Message, e.Err)
	}
	return fmt.Sprintf("AuditError(%d): %s", e.Kind, e.Message)
}

func (e *AuditError) Unwrap() error { return e.Err }

// AuditClient は監査クライアントのインターフェース。
type AuditClient interface {
	Record(ctx context.Context, event AuditEvent) error
	Flush(ctx context.Context) ([]AuditEvent, error)
}

// BufferedAuditClient はバッファ付き監査クライアント。
type BufferedAuditClient struct {
	mu  sync.Mutex
	buf []AuditEvent
}

// NewBufferedAuditClient は新しい BufferedAuditClient を生成する。
func NewBufferedAuditClient() *BufferedAuditClient {
	return &BufferedAuditClient{}
}

func (c *BufferedAuditClient) Record(_ context.Context, event AuditEvent) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.buf = append(c.buf, event)
	return nil
}

func (c *BufferedAuditClient) Flush(_ context.Context) ([]AuditEvent, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	events := make([]AuditEvent, len(c.buf))
	copy(events, c.buf)
	c.buf = nil
	return events, nil
}
