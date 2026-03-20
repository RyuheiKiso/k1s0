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

// NewPostgresEventStore は新しい PostgresEventStore を生成する。
//
// databaseURL は PostgreSQL 接続 URL (例: "postgres://user:pass@localhost:5432/dbname?sslmode=disable")
func NewPostgresEventStore(databaseURL string) (*PostgresEventStore, error) {
	db, err := sql.Open("postgres", databaseURL)
	if err != nil {
		return nil, &EventStoreError{Code: "CONNECTION_ERROR", Message: err.Error()}
	}
	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(5 * time.Minute)
	return &PostgresEventStore{db: db}, nil
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
	tx, err := s.db.BeginTx(ctx, nil)
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

	// Get current version with row-level lock
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
		payload, err := json.Marshal(json.RawMessage(event.Payload))
		if err != nil {
			return 0, &EventStoreError{Code: "SERIALIZATION_ERROR", Message: err.Error()}
		}
		metadata, err := json.Marshal(json.RawMessage(event.Metadata))
		if err != nil {
			return 0, &EventStoreError{Code: "SERIALIZATION_ERROR", Message: err.Error()}
		}

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
	rows, err := s.db.QueryContext(ctx,
		`SELECT event_id, stream_id, version, event_type, payload, metadata, recorded_at
		 FROM events WHERE stream_id = $1 AND version >= $2 ORDER BY version ASC`,
		streamID.String(),
		int64(fromVersion),
	)
	if err != nil {
		return nil, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	defer rows.Close()

	return scanEvents(rows)
}

func (s *PostgresEventStore) Exists(ctx context.Context, streamID StreamId) (bool, error) {
	var count int64
	err := s.db.QueryRowContext(ctx,
		"SELECT COUNT(*) FROM events WHERE stream_id = $1 LIMIT 1",
		streamID.String(),
	).Scan(&count)
	if err != nil {
		return false, &EventStoreError{Code: "QUERY_ERROR", Message: err.Error()}
	}
	return count > 0, nil
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
