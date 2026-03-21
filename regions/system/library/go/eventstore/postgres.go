package eventstore

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"time"

	_ "github.com/lib/pq"
)

const createEventsTableSQL = `
CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    event_id TEXT NOT NULL UNIQUE,
    stream_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (stream_id, version)
);
CREATE INDEX IF NOT EXISTS idx_events_stream_id ON events (stream_id, version);
`

// PostgresEventStore は PostgreSQL を使用したイベントストア実装。
type PostgresEventStore struct {
	db *sql.DB
}

// EventStoreConfig は PostgresEventStore の接続プール設定を保持する
type EventStoreConfig struct {
	MaxOpenConns    int
	MaxIdleConns    int
	ConnMaxLifetime time.Duration
}

// DefaultEventStoreConfig はデフォルトの接続プール設定（後方互換）
func DefaultEventStoreConfig() EventStoreConfig {
	return EventStoreConfig{
		MaxOpenConns:    10,
		MaxIdleConns:    5,
		ConnMaxLifetime: 5 * time.Minute,
	}
}

// NewPostgresEventStoreWithConfig は接続プール設定を指定して新しい PostgresEventStore を生成する。
//
// databaseURL は PostgreSQL 接続 URL (例: "postgres://user:pass@localhost:5432/dbname?sslmode=disable")
// cfg には接続プールのパラメータ（最大接続数、アイドル接続数、接続最大有効期間）を指定する。
func NewPostgresEventStoreWithConfig(databaseURL string, cfg EventStoreConfig) (*PostgresEventStore, error) {
	db, err := sql.Open("postgres", databaseURL)
	if err != nil {
		return nil, &EventStoreError{Code: "CONNECTION_ERROR", Message: err.Error()}
	}
	// 外部化された設定値を接続プールに適用する
	db.SetMaxOpenConns(cfg.MaxOpenConns)
	db.SetMaxIdleConns(cfg.MaxIdleConns)
	db.SetConnMaxLifetime(cfg.ConnMaxLifetime)
	return &PostgresEventStore{db: db}, nil
}

// NewPostgresEventStore は新しい PostgresEventStore をデフォルト設定で生成する（後方互換）。
//
// databaseURL は PostgreSQL 接続 URL (例: "postgres://user:pass@localhost:5432/dbname?sslmode=disable")
func NewPostgresEventStore(databaseURL string) (*PostgresEventStore, error) {
	// デフォルト設定を使用して後方互換性を維持する
	return NewPostgresEventStoreWithConfig(databaseURL, DefaultEventStoreConfig())
}

// NewPostgresEventStoreFromDB は既存の *sql.DB から PostgresEventStore を生成する。
func NewPostgresEventStoreFromDB(db *sql.DB) *PostgresEventStore {
	return &PostgresEventStore{db: db}
}

// Migrate はイベントテーブルを作成する。
func (s *PostgresEventStore) Migrate(ctx context.Context) error {
	_, err := s.db.ExecContext(ctx, createEventsTableSQL)
	if err != nil {
		return &EventStoreError{Code: "MIGRATION_ERROR", Message: err.Error()}
	}
	return nil
}

// Close はデータベース接続を閉じる。
func (s *PostgresEventStore) Close() error {
	return s.db.Close()
}

func (s *PostgresEventStore) Append(ctx context.Context, streamID StreamId, events []*EventEnvelope, expectedVersion *uint64) (uint64, error) {
	// REPEATABLE READ で楽観的ロックのバージョンチェックをファントムリードから保護する。
	tx, err := s.db.BeginTx(ctx, &sql.TxOptions{Isolation: sql.LevelRepeatableRead})
	if err != nil {
		return 0, &EventStoreError{Code: "TX_ERROR", Message: err.Error()}
	}
	// トランザクションをロールバックし、エラーが発生した場合はログに記録する
	// sql.ErrTxDone はコミット済みの場合に返される正常終了扱いのエラーのため除外する
	defer func() {
		if rbErr := tx.Rollback(); rbErr != nil && !errors.Is(rbErr, sql.ErrTxDone) {
			slog.Error("トランザクションのロールバックに失敗しました", slog.String("error", rbErr.Error()))
		}
	}()

	// 行レベルロックを使用して現在のバージョンを取得する。
	var currentVersion int64
	err = tx.QueryRowContext(ctx,
		"SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = $1 FOR UPDATE",
		streamID.String(),
	).Scan(&currentVersion)
	if err != nil {
		return 0, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}

	cv := uint64(currentVersion)
	if expectedVersion != nil && *expectedVersion != cv {
		return 0, NewVersionConflictError(*expectedVersion, cv)
	}

	version := cv
	for _, event := range events {
		version++
		// json.RawMessage はすでに valid JSON バイト列のため json.Marshal での二重エンコードを避ける。
		payload := []byte(event.Payload)
		metadata := []byte(event.Metadata)

		_, err = tx.ExecContext(ctx,
			`INSERT INTO events (event_id, stream_id, version, event_type, payload, metadata, recorded_at)
			 VALUES ($1, $2, $3, $4, $5, $6, $7)`,
			event.EventID,
			streamID.String(),
			int64(version),
			event.EventType,
			payload,
			metadata,
			event.RecordedAt,
		)
		if err != nil {
			return 0, &EventStoreError{Code: "INSERT_ERROR", Message: err.Error()}
		}
	}

	if err := tx.Commit(); err != nil {
		return 0, &EventStoreError{Code: "COMMIT_ERROR", Message: err.Error()}
	}

	return version, nil
}

// defaultLoadLimit は Load() メソッドのデフォルト取得件数上限。
// 無制限クエリによるメモリ枯渇やレスポンス遅延を防ぐ安全策として設定する。
const defaultLoadLimit = 10000

// Load はストリームの全イベントをデフォルト上限付きで読み込む。
// 安全策として LIMIT を付与し、無制限の結果セットによるメモリ圧迫を防止する。
// 明示的に件数を指定したい場合は LoadWithLimit を使用すること。
func (s *PostgresEventStore) Load(ctx context.Context, streamID StreamId) ([]*EventEnvelope, error) {
	return s.LoadWithLimit(ctx, streamID, defaultLoadLimit)
}

// LoadWithLimit は指定された件数上限でストリームのイベントを読み込む。
// limit パラメータにより取得件数を明示的に制御できる。
// ページネーションが必要な場合や、大量イベントの段階的読み込みに使用する。
func (s *PostgresEventStore) LoadWithLimit(ctx context.Context, streamID StreamId, limit int) ([]*EventEnvelope, error) {
	rows, err := s.db.QueryContext(ctx,
		`SELECT event_id, stream_id, version, event_type, payload, metadata, recorded_at
		 FROM events WHERE stream_id = $1 ORDER BY version ASC LIMIT $2`,
		streamID.String(),
		limit,
	)
	if err != nil {
		return nil, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	defer rows.Close()

	return scanEvents(rows)
}

func (s *PostgresEventStore) LoadFrom(ctx context.Context, streamID StreamId, fromVersion uint64) ([]*EventEnvelope, error) {
	// defaultLoadLimit で無制限クエリによるメモリ枯渇を防止する（Load() と同一安全策）。
	rows, err := s.db.QueryContext(ctx,
		`SELECT event_id, stream_id, version, event_type, payload, metadata, recorded_at
		 FROM events WHERE stream_id = $1 AND version >= $2 ORDER BY version ASC LIMIT $3`,
		streamID.String(),
		int64(fromVersion),
		defaultLoadLimit,
	)
	if err != nil {
		return nil, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	defer rows.Close()

	return scanEvents(rows)
}

func (s *PostgresEventStore) Exists(ctx context.Context, streamID StreamId) (bool, error) {
	// SELECT EXISTS を使用することで COUNT(*) より効率的にストリーム存在確認を行う。
	// PostgreSQL は EXISTS サブクエリで最初の一致行を見つけた時点でスキャンを停止する。
	var exists bool
	err := s.db.QueryRowContext(ctx,
		"SELECT EXISTS(SELECT 1 FROM events WHERE stream_id = $1)",
		streamID.String(),
	).Scan(&exists)
	if err != nil {
		return false, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	return exists, nil
}

func (s *PostgresEventStore) CurrentVersion(ctx context.Context, streamID StreamId) (uint64, error) {
	var version int64
	err := s.db.QueryRowContext(ctx,
		"SELECT COALESCE(MAX(version), 0) FROM events WHERE stream_id = $1",
		streamID.String(),
	).Scan(&version)
	if err != nil {
		return 0, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	return uint64(version), nil
}

func scanEvents(rows *sql.Rows) ([]*EventEnvelope, error) {
	var events []*EventEnvelope
	for rows.Next() {
		var (
			e          EventEnvelope
			version    int64
			payload    []byte
			metadata   []byte
			recordedAt time.Time
		)
		err := rows.Scan(
			&e.EventID,
			&e.StreamID,
			&version,
			&e.EventType,
			&payload,
			&metadata,
			&recordedAt,
		)
		if err != nil {
			return nil, &EventStoreError{
				Code:    "SCAN_ERROR",
				Message: fmt.Sprintf("failed to scan event row: %v", err),
			}
		}
		e.Version = uint64(version)
		e.Payload = json.RawMessage(payload)
		e.Metadata = json.RawMessage(metadata)
		e.RecordedAt = recordedAt
		events = append(events, &e)
	}
	if err := rows.Err(); err != nil {
		return nil, &EventStoreError{Code: "ROWS_ERROR", Message: err.Error()}
	}
	if events == nil {
		return []*EventEnvelope{}, nil
	}
	return events, nil
}
