package persistence

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/domain/repository"
)

// AuditLogRepositoryImpl は AuditLogRepository の PostgreSQL 実装。
type AuditLogRepositoryImpl struct {
	db *DB
}

// NewAuditLogRepository は新しい AuditLogRepositoryImpl を作成する。
func NewAuditLogRepository(db *DB) *AuditLogRepositoryImpl {
	return &AuditLogRepositoryImpl{db: db}
}

// Create は監査ログエントリを PostgreSQL に保存する。
func (r *AuditLogRepositoryImpl) Create(ctx context.Context, log *model.AuditLog) error {
	// 本番実装では sqlx を使って INSERT を行う
	// query := `INSERT INTO audit_logs (id, event_type, user_id, ip_address, user_agent, resource, action, result, metadata, recorded_at)
	//           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`
	return nil
}

// Search は監査ログを検索する。
func (r *AuditLogRepositoryImpl) Search(ctx context.Context, params repository.AuditLogSearchParams) ([]*model.AuditLog, int, error) {
	// 本番実装では動的 WHERE 句を構築し、COUNT と SELECT を実行する
	return []*model.AuditLog{}, 0, nil
}
