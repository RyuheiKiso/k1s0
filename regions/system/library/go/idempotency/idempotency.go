package idempotency

import (
	"context"
	"fmt"
	"time"
)

// IdempotencyStatus はべき等レコードのステータス。
type IdempotencyStatus int

const (
	StatusPending IdempotencyStatus = iota
	StatusCompleted
	StatusFailed
)

func (s IdempotencyStatus) String() string {
	switch s {
	case StatusPending:
		return "Pending"
	case StatusCompleted:
		return "Completed"
	case StatusFailed:
		return "Failed"
	default:
		return "Unknown"
	}
}

// IdempotencyRecord はべき等レコード。
type IdempotencyRecord struct {
	Key            string
	Status         IdempotencyStatus
	RequestHash    *string
	ResponseBody   *string
	ResponseStatus *int
	CreatedAt      time.Time
	ExpiresAt      *time.Time
	CompletedAt    *time.Time
}

// NewIdempotencyRecord は新しいべき等レコードを作成する。
func NewIdempotencyRecord(key string, ttl *time.Duration) *IdempotencyRecord {
	now := time.Now()
	r := &IdempotencyRecord{
		Key:       key,
		Status:    StatusPending,
		CreatedAt: now,
	}
	if ttl != nil {
		exp := now.Add(*ttl)
		r.ExpiresAt = &exp
	}
	return r
}

// IsExpired はレコードが期限切れかどうかを返す。
func (r *IdempotencyRecord) IsExpired() bool {
	if r.ExpiresAt == nil {
		return false
	}
	return time.Now().After(*r.ExpiresAt)
}

// IdempotencyStore はべき等ストアのインターフェース。
type IdempotencyStore interface {
	// Get はレコードを取得する（期限切れの場合は nil を返す）。
	Get(ctx context.Context, key string) (*IdempotencyRecord, error)
	// Insert は新規レコードを挿入する（重複キーはエラー）。
	Insert(ctx context.Context, record *IdempotencyRecord) error
	// Update はレコードのステータスと結果を更新する。
	Update(ctx context.Context, key string, status IdempotencyStatus, responseBody *string, responseStatus *int) error
	// Delete はレコードを削除する。
	Delete(ctx context.Context, key string) (bool, error)
}

// IdempotencyError はべき等操作のエラー。
type IdempotencyError struct {
	Code    string
	Message string
}

func (e *IdempotencyError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewDuplicateError は重複キーエラーを生成する。
func NewDuplicateError(key string) *IdempotencyError {
	return &IdempotencyError{Code: "DUPLICATE", Message: fmt.Sprintf("重複リクエストです: key=%s", key)}
}

// NewNotFoundError はキーが見つからないエラーを生成する。
func NewNotFoundError(key string) *IdempotencyError {
	return &IdempotencyError{Code: "NOT_FOUND", Message: fmt.Sprintf("キーが見つかりません: %s", key)}
}
