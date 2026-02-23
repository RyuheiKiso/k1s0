package eventstore

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/k1s0-platform/system-library-go-eventstore/internal/uuid"
)

// StreamId はイベントストリームの識別子。
type StreamId struct {
	value string
}

// NewStreamId は新しい StreamId を作成する。
func NewStreamId(value string) StreamId {
	return StreamId{value: value}
}

// String は StreamId の文字列表現を返す。
func (s StreamId) String() string {
	return s.value
}

// EventEnvelope はイベントのエンベロープ。
type EventEnvelope struct {
	EventID    string          `json:"event_id"`
	StreamID   string          `json:"stream_id"`
	Version    uint64          `json:"version"`
	EventType  string          `json:"event_type"`
	Payload    json.RawMessage `json:"payload"`
	Metadata   json.RawMessage `json:"metadata"`
	RecordedAt time.Time       `json:"recorded_at"`
}

// NewEventEnvelope は新しい EventEnvelope を作成する。
func NewEventEnvelope(streamID StreamId, version uint64, eventType string, payload json.RawMessage) *EventEnvelope {
	return &EventEnvelope{
		EventID:    uuid.New(),
		StreamID:   streamID.String(),
		Version:    version,
		EventType:  eventType,
		Payload:    payload,
		Metadata:   json.RawMessage("{}"),
		RecordedAt: time.Now(),
	}
}

// Snapshot はイベントストリームのスナップショット。
type Snapshot struct {
	StreamID  string          `json:"stream_id"`
	Version   uint64          `json:"version"`
	State     json.RawMessage `json:"state"`
	CreatedAt time.Time       `json:"created_at"`
}

// EventStore はイベントストアのインターフェース。
type EventStore interface {
	// Append はイベントをストリームに追加する。expectedVersion が指定された場合、バージョン競合をチェックする。
	Append(ctx context.Context, streamID StreamId, events []*EventEnvelope, expectedVersion *uint64) (uint64, error)
	// Load はストリームの全イベントを読み込む。
	Load(ctx context.Context, streamID StreamId) ([]*EventEnvelope, error)
	// LoadFrom は指定バージョン以降のイベントを読み込む。
	LoadFrom(ctx context.Context, streamID StreamId, fromVersion uint64) ([]*EventEnvelope, error)
	// Exists はストリームが存在するかどうかを返す。
	Exists(ctx context.Context, streamID StreamId) (bool, error)
	// CurrentVersion はストリームの現在のバージョンを返す。
	CurrentVersion(ctx context.Context, streamID StreamId) (uint64, error)
}

// SnapshotStore はスナップショットストアのインターフェース。
type SnapshotStore interface {
	// SaveSnapshot はスナップショットを保存する。
	SaveSnapshot(ctx context.Context, snapshot *Snapshot) error
	// LoadSnapshot はスナップショットを読み込む。
	LoadSnapshot(ctx context.Context, streamID StreamId) (*Snapshot, error)
}

// EventStoreError はイベントストア操作のエラー。
type EventStoreError struct {
	Code    string
	Message string
}

func (e *EventStoreError) Error() string {
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// NewVersionConflictError はバージョン競合エラーを生成する。
func NewVersionConflictError(expected, actual uint64) *EventStoreError {
	return &EventStoreError{
		Code:    "VERSION_CONFLICT",
		Message: fmt.Sprintf("バージョン競合: expected=%d, actual=%d", expected, actual),
	}
}

// NewStreamNotFoundError はストリームが見つからないエラーを生成する。
func NewStreamNotFoundError(streamID string) *EventStoreError {
	return &EventStoreError{
		Code:    "STREAM_NOT_FOUND",
		Message: fmt.Sprintf("ストリームが見つかりません: %s", streamID),
	}
}
