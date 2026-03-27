package idempotency

import (
	"context"
	"fmt"
	"time"
)

// IdempotencyStatus はべき等キーの処理状態。
type IdempotencyStatus string

const (
	StatusPending   IdempotencyStatus = "pending"
	StatusCompleted IdempotencyStatus = "completed"
	StatusFailed    IdempotencyStatus = "failed"
)

func (s IdempotencyStatus) String() string {
	return string(s)
}

// IdempotencyRecord はべき等レコード。
type IdempotencyRecord struct {
	Key        string            `json:"key"`
	Status     IdempotencyStatus `json:"status"`
	Response   []byte            `json:"response,omitempty"`
	StatusCode int               `json:"status_code,omitempty"`
	Error      string            `json:"error,omitempty"`
	CreatedAt  time.Time         `json:"created_at"`
	ExpiresAt  time.Time         `json:"expires_at"`
}

// NewIdempotencyRecord は新規レコードを生成する。
func NewIdempotencyRecord(key string, ttl *time.Duration) *IdempotencyRecord {
	now := time.Now().UTC()
	record := &IdempotencyRecord{
		Key:       key,
		Status:    StatusPending,
		CreatedAt: now,
	}
	if ttl != nil {
		record.ExpiresAt = now.Add(*ttl)
	}
	return record
}

// IsExpired はレコードが期限切れかを返す。
func (r *IdempotencyRecord) IsExpired() bool {
	if r.ExpiresAt.IsZero() {
		return false
	}
	return time.Now().After(r.ExpiresAt)
}

// IdempotencyStore はべき等ストアのインターフェース。
type IdempotencyStore interface {
	Get(ctx context.Context, key string) (*IdempotencyRecord, error)
	Set(ctx context.Context, key string, record *IdempotencyRecord) error
	MarkCompleted(ctx context.Context, key string, response []byte, statusCode int) error
	MarkFailed(ctx context.Context, key string, err error) error
}

// IdempotencyError はべき等処理エラー。
type IdempotencyError struct {
	Code    string
	Message string
}

func (e *IdempotencyError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewDuplicateError は重複キーエラーを返す。
func NewDuplicateError(key string) *IdempotencyError {
	return &IdempotencyError{
		Code:    "DUPLICATE",
		Message: fmt.Sprintf("duplicate idempotency key: %s", key),
	}
}

// NewNotFoundError はキー未検出エラーを返す。
func NewNotFoundError(key string) *IdempotencyError {
	return &IdempotencyError{
		Code:    "NOT_FOUND",
		Message: fmt.Sprintf("idempotency key not found: %s", key),
	}
}

// NewExpiredError は期限切れエラーを返す。
func NewExpiredError(key string) *IdempotencyError {
	return &IdempotencyError{
		Code:    "EXPIRED",
		Message: fmt.Sprintf("idempotency key expired: %s", key),
	}
}

// ---------------------------------------------------------------------------
// L-3 監査対応: Go 命名規約準拠の短縮型エイリアス（stutter 命名解消）
// 新しいコードでは idempotency.Status / Record / Store / Error を使用すること。
// ---------------------------------------------------------------------------

// Status は IdempotencyStatus の短縮エイリアス。
type Status = IdempotencyStatus

// Record は IdempotencyRecord の短縮エイリアス。
type Record = IdempotencyRecord

// Store は IdempotencyStore の短縮エイリアス。
type Store = IdempotencyStore

// Err は IdempotencyError の短縮エイリアス。
// 注意: builtin error インターフェースとの混同を避けるため Err を使用する。
type Err = IdempotencyError
