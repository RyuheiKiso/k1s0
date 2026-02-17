package repository

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-config/internal/domain/repository"
	"github.com/k1s0-platform/system-server-go-config/internal/infra/persistence"
)

// ConfigPostgresRepository は ConfigRepository の PostgreSQL 実装。
type ConfigPostgresRepository struct {
	db *persistence.DB
}

// NewConfigPostgresRepository は新しい ConfigPostgresRepository を作成する。
func NewConfigPostgresRepository(db *persistence.DB) *ConfigPostgresRepository {
	return &ConfigPostgresRepository{db: db}
}

// GetByKey は namespace と key で設定エントリを取得する。
func (r *ConfigPostgresRepository) GetByKey(ctx context.Context, namespace, key string) (*model.ConfigEntry, error) {
	query := `SELECT id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at
	           FROM config_entries
	           WHERE namespace = $1 AND key = $2`

	var entry model.ConfigEntry
	err := r.db.Conn().QueryRowContext(ctx, query, namespace, key).Scan(
		&entry.ID, &entry.Namespace, &entry.Key, &entry.ValueJSON,
		&entry.Version, &entry.Description, &entry.CreatedBy, &entry.UpdatedBy,
		&entry.CreatedAt, &entry.UpdatedAt,
	)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("config entry not found: %s/%s", namespace, key)
		}
		return nil, fmt.Errorf("failed to get config entry: %w", err)
	}

	return &entry, nil
}

// ListByNamespace は namespace 内の設定エントリを一覧取得する。
func (r *ConfigPostgresRepository) ListByNamespace(ctx context.Context, params repository.ConfigListParams) ([]*model.ConfigEntry, int, error) {
	var conditions []string
	var args []interface{}
	bindIdx := 1

	conditions = append(conditions, fmt.Sprintf("namespace = $%d", bindIdx))
	args = append(args, params.Namespace)
	bindIdx++

	if params.Search != "" {
		conditions = append(conditions, fmt.Sprintf("key ILIKE $%d", bindIdx))
		args = append(args, "%"+params.Search+"%")
		bindIdx++
	}

	whereClause := " WHERE "
	for i, c := range conditions {
		if i > 0 {
			whereClause += " AND "
		}
		whereClause += c
	}

	// count クエリ
	countQuery := fmt.Sprintf("SELECT COUNT(*) FROM config_entries%s", whereClause)
	var totalCount int
	err := r.db.Conn().QueryRowContext(ctx, countQuery, args...).Scan(&totalCount)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to count config entries: %w", err)
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
		"SELECT id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at FROM config_entries%s ORDER BY key ASC LIMIT $%d OFFSET $%d",
		whereClause, bindIdx, bindIdx+1,
	)
	dataArgs := append(args, pageSize, offset)

	rows, err := r.db.Conn().QueryContext(ctx, dataQuery, dataArgs...)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to query config entries: %w", err)
	}
	defer rows.Close()

	var entries []*model.ConfigEntry
	for rows.Next() {
		var entry model.ConfigEntry
		err := rows.Scan(
			&entry.ID, &entry.Namespace, &entry.Key, &entry.ValueJSON,
			&entry.Version, &entry.Description, &entry.CreatedBy, &entry.UpdatedBy,
			&entry.CreatedAt, &entry.UpdatedAt,
		)
		if err != nil {
			return nil, 0, fmt.Errorf("failed to scan config entry row: %w", err)
		}
		entries = append(entries, &entry)
	}

	if err := rows.Err(); err != nil {
		return nil, 0, fmt.Errorf("error iterating config entry rows: %w", err)
	}

	return entries, totalCount, nil
}

// GetByServiceName はサービス名に対応する設定エントリを取得する。
func (r *ConfigPostgresRepository) GetByServiceName(ctx context.Context, serviceName string) ([]*model.ConfigEntry, error) {
	query := `SELECT ce.id, ce.namespace, ce.key, ce.value_json, ce.version, ce.description, ce.created_by, ce.updated_by, ce.created_at, ce.updated_at
	           FROM config_entries ce
	           INNER JOIN service_config_mappings scm ON ce.id = scm.config_entry_id
	           WHERE scm.service_name = $1
	           ORDER BY ce.namespace, ce.key`

	rows, err := r.db.Conn().QueryContext(ctx, query, serviceName)
	if err != nil {
		return nil, fmt.Errorf("failed to query service configs: %w", err)
	}
	defer rows.Close()

	var entries []*model.ConfigEntry
	for rows.Next() {
		var entry model.ConfigEntry
		err := rows.Scan(
			&entry.ID, &entry.Namespace, &entry.Key, &entry.ValueJSON,
			&entry.Version, &entry.Description, &entry.CreatedBy, &entry.UpdatedBy,
			&entry.CreatedAt, &entry.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan service config row: %w", err)
		}
		entries = append(entries, &entry)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating service config rows: %w", err)
	}

	return entries, nil
}

// Create は設定エントリを PostgreSQL に保存する。
func (r *ConfigPostgresRepository) Create(ctx context.Context, entry *model.ConfigEntry) error {
	query := `INSERT INTO config_entries (id, namespace, key, value_json, version, description, created_by, updated_by, created_at, updated_at)
	           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`

	_, err := r.db.Conn().ExecContext(ctx, query,
		entry.ID, entry.Namespace, entry.Key, entry.ValueJSON,
		entry.Version, entry.Description, entry.CreatedBy, entry.UpdatedBy,
		entry.CreatedAt, entry.UpdatedAt,
	)
	if err != nil {
		return fmt.Errorf("failed to insert config entry: %w", err)
	}

	return nil
}

// Update は設定エントリを更新する。バージョンが一致しない場合はエラーを返す。
func (r *ConfigPostgresRepository) Update(ctx context.Context, entry *model.ConfigEntry, expectedVersion int) error {
	query := `UPDATE config_entries
	           SET value_json = $1, version = $2, description = $3, updated_by = $4, updated_at = $5
	           WHERE namespace = $6 AND key = $7 AND version = $8`

	result, err := r.db.Conn().ExecContext(ctx, query,
		entry.ValueJSON, entry.Version, entry.Description, entry.UpdatedBy, entry.UpdatedAt,
		entry.Namespace, entry.Key, expectedVersion,
	)
	if err != nil {
		return fmt.Errorf("failed to update config entry: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("config entry not found or version conflict")
	}

	return nil
}

// Delete は設定エントリを削除する。
func (r *ConfigPostgresRepository) Delete(ctx context.Context, namespace, key string) error {
	query := `DELETE FROM config_entries WHERE namespace = $1 AND key = $2`

	result, err := r.db.Conn().ExecContext(ctx, query, namespace, key)
	if err != nil {
		return fmt.Errorf("failed to delete config entry: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}
	if rows == 0 {
		return fmt.Errorf("config entry not found: %s/%s", namespace, key)
	}

	return nil
}
