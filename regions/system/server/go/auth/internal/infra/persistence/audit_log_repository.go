package persistence

import (
	"context"
	"encoding/json"
	"fmt"

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
	metadataJSON, err := json.Marshal(log.Metadata)
	if err != nil {
		return fmt.Errorf("failed to marshal metadata: %w", err)
	}

	query := `INSERT INTO audit_logs (id, event_type, user_id, ip_address, user_agent, resource, action, result, metadata, recorded_at)
	           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`

	_, err = r.db.conn.ExecContext(ctx, query,
		log.ID, log.EventType, log.UserID, log.IPAddress, log.UserAgent,
		log.Resource, log.Action, log.Result, metadataJSON, log.RecordedAt,
	)
	if err != nil {
		return fmt.Errorf("failed to insert audit log: %w", err)
	}

	return nil
}

// Search は監査ログを検索する。
func (r *AuditLogRepositoryImpl) Search(ctx context.Context, params repository.AuditLogSearchParams) ([]*model.AuditLog, int, error) {
	var conditions []string
	var args []interface{}
	bindIdx := 1

	if params.UserID != "" {
		conditions = append(conditions, fmt.Sprintf("user_id = $%d", bindIdx))
		args = append(args, params.UserID)
		bindIdx++
	}
	if params.EventType != "" {
		conditions = append(conditions, fmt.Sprintf("event_type = $%d", bindIdx))
		args = append(args, params.EventType)
		bindIdx++
	}
	if params.Result != "" {
		conditions = append(conditions, fmt.Sprintf("result = $%d", bindIdx))
		args = append(args, params.Result)
		bindIdx++
	}
	if params.From != nil {
		conditions = append(conditions, fmt.Sprintf("recorded_at >= $%d", bindIdx))
		args = append(args, *params.From)
		bindIdx++
	}
	if params.To != nil {
		conditions = append(conditions, fmt.Sprintf("recorded_at <= $%d", bindIdx))
		args = append(args, *params.To)
		bindIdx++
	}

	whereClause := ""
	if len(conditions) > 0 {
		whereClause = " WHERE "
		for i, c := range conditions {
			if i > 0 {
				whereClause += " AND "
			}
			whereClause += c
		}
	}

	// count クエリ
	countQuery := fmt.Sprintf("SELECT COUNT(*) FROM audit_logs%s", whereClause)
	var totalCount int
	err := r.db.conn.QueryRowContext(ctx, countQuery, args...).Scan(&totalCount)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to count audit logs: %w", err)
	}

	// ページネーション計算
	page := params.Page
	if page < 1 {
		page = 1
	}
	pageSize := params.PageSize
	if pageSize < 1 {
		pageSize = 20
	}
	offset := (page - 1) * pageSize

	// data クエリ
	dataQuery := fmt.Sprintf(
		"SELECT id, event_type, user_id, ip_address, user_agent, resource, action, result, metadata, recorded_at FROM audit_logs%s ORDER BY recorded_at DESC LIMIT $%d OFFSET $%d",
		whereClause, bindIdx, bindIdx+1,
	)
	dataArgs := append(args, pageSize, offset)

	rows, err := r.db.conn.QueryContext(ctx, dataQuery, dataArgs...)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to query audit logs: %w", err)
	}
	defer rows.Close()

	var logs []*model.AuditLog
	for rows.Next() {
		var log model.AuditLog
		var metadataJSON []byte
		err := rows.Scan(
			&log.ID, &log.EventType, &log.UserID, &log.IPAddress, &log.UserAgent,
			&log.Resource, &log.Action, &log.Result, &metadataJSON, &log.RecordedAt,
		)
		if err != nil {
			return nil, 0, fmt.Errorf("failed to scan audit log row: %w", err)
		}
		if metadataJSON != nil {
			if err := json.Unmarshal(metadataJSON, &log.Metadata); err != nil {
				log.Metadata = map[string]string{}
			}
		}
		logs = append(logs, &log)
	}

	if err := rows.Err(); err != nil {
		return nil, 0, fmt.Errorf("error iterating audit log rows: %w", err)
	}

	return logs, totalCount, nil
}
