package repository

import (
	"context"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// AuditLogRepository は監査ログの永続化インターフェース。
type AuditLogRepository interface {
	// Create は監査ログエントリを作成する。
	Create(ctx context.Context, log *model.AuditLog) error

	// Search は監査ログを検索する。
	Search(ctx context.Context, params AuditLogSearchParams) ([]*model.AuditLog, int, error)
}

// AuditLogSearchParams は監査ログ検索パラメータ。
type AuditLogSearchParams struct {
	UserID    string
	EventType string
	Result    string
	From      *time.Time
	To        *time.Time
	Page      int
	PageSize  int
}
