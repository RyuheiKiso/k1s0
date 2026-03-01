package outbox

import (
	"context"
	"encoding/json"
	"fmt"
	"math"
	"time"

	"github.com/google/uuid"
)

// OutboxStatus はアウトボックスメッセージの処理ステータスを表す。
type OutboxStatus string

const (
	// OutboxStatusPending は未処理（初期状態）。
	OutboxStatusPending OutboxStatus = "PENDING"
	// OutboxStatusProcessing は処理中。
	OutboxStatusProcessing OutboxStatus = "PROCESSING"
	// OutboxStatusDelivered は発行完了。
	OutboxStatusDelivered OutboxStatus = "DELIVERED"
	// OutboxStatusFailed は発行失敗（リトライ対象）。
	OutboxStatusFailed OutboxStatus = "FAILED"
	// OutboxStatusDeadLetter は最大リトライ回数超過（Dead Letter）。
	OutboxStatusDeadLetter OutboxStatus = "DEAD_LETTER"
)

// OutboxMessage はアウトボックステーブルに格納するメッセージを表す。
type OutboxMessage struct {
	// ID はメッセージの一意識別子。
	ID string
	// Topic は発行先 Kafka トピック。
	Topic string
	// PartitionKey はパーティションキー。
	PartitionKey string
	// Payload はメッセージペイロード（JSON）。
	Payload json.RawMessage
	// Status は処理ステータス。
	Status OutboxStatus
	// RetryCount はリトライ回数。
	RetryCount int
	// MaxRetries は最大リトライ回数。
	MaxRetries int
	// LastError は最終エラーメッセージ。空文字列はエラーなし。
	LastError string
	// CreatedAt は作成日時。
	CreatedAt time.Time
	// ProcessAfter は次回処理予定日時（リトライバックオフ用）。
	ProcessAfter time.Time
}

// NewOutboxMessage は新しいアウトボックスメッセージを生成する。
func NewOutboxMessage(topic, partitionKey string, payload json.RawMessage) OutboxMessage {
	now := time.Now().UTC()
	return OutboxMessage{
		ID:           uuid.New().String(),
		Topic:        topic,
		PartitionKey: partitionKey,
		Payload:      payload,
		Status:       OutboxStatusPending,
		RetryCount:   0,
		MaxRetries:   3,
		LastError:    "",
		CreatedAt:    now,
		ProcessAfter: now,
	}
}

// MarkProcessing はメッセージを処理中状態に遷移する。
func (m *OutboxMessage) MarkProcessing() {
	m.Status = OutboxStatusProcessing
}

// MarkDelivered はメッセージを配信完了状態に遷移する。
func (m *OutboxMessage) MarkDelivered() {
	m.Status = OutboxStatusDelivered
}

// MarkFailed はメッセージを失敗状態に遷移し、リトライ回数をインクリメントする。
// 最大リトライ回数に達した場合は DeadLetter に遷移する。
func (m *OutboxMessage) MarkFailed(errMsg string) {
	m.RetryCount++
	m.LastError = errMsg
	if m.RetryCount >= m.MaxRetries {
		m.Status = OutboxStatusDeadLetter
	} else {
		m.Status = OutboxStatusFailed
		// Exponential backoff: 2^retry_count 秒後に再処理
		delaySecs := math.Pow(2, float64(m.RetryCount))
		m.ProcessAfter = time.Now().UTC().Add(time.Duration(delaySecs) * time.Second)
	}
}

// IsProcessable はメッセージが処理可能かどうか判定する。
func (m *OutboxMessage) IsProcessable() bool {
	return (m.Status == OutboxStatusPending || m.Status == OutboxStatusFailed) &&
		!m.ProcessAfter.After(time.Now().UTC())
}

// OutboxError はアウトボックス操作に関するエラーを表す。
type OutboxError struct {
	// Kind はエラーの種別。
	Kind OutboxErrorKind
	// Message はエラーの詳細メッセージ。
	Message string
	// Err は元のエラー（あれば）。
	Err error
}

// OutboxErrorKind はアウトボックスエラーの種別を表す。
type OutboxErrorKind int

const (
	// ErrStoreError はストア操作エラー。
	ErrStoreError OutboxErrorKind = iota
	// ErrPublishError は発行エラー。
	ErrPublishError
	// ErrSerializationError はシリアライゼーションエラー。
	ErrSerializationError
	// ErrNotFound はメッセージが見つからないエラー。
	ErrNotFound
)

// Error は OutboxError の文字列表現を返す。
func (e *OutboxError) Error() string {
	kindStr := ""
	switch e.Kind {
	case ErrStoreError:
		kindStr = "store error"
	case ErrPublishError:
		kindStr = "publish error"
	case ErrSerializationError:
		kindStr = "serialization error"
	case ErrNotFound:
		kindStr = "message not found"
	}
	if e.Err != nil {
		return fmt.Sprintf("%s: %s: %v", kindStr, e.Message, e.Err)
	}
	return fmt.Sprintf("%s: %s", kindStr, e.Message)
}

// Unwrap は元のエラーを返す。
func (e *OutboxError) Unwrap() error {
	return e.Err
}

// NewStoreError はストアエラーを生成する。
func NewStoreError(message string, err error) *OutboxError {
	return &OutboxError{Kind: ErrStoreError, Message: message, Err: err}
}

// NewPublishError は発行エラーを生成する。
func NewPublishError(message string, err error) *OutboxError {
	return &OutboxError{Kind: ErrPublishError, Message: message, Err: err}
}

// NewSerializationError はシリアライゼーションエラーを生成する。
func NewSerializationError(message string, err error) *OutboxError {
	return &OutboxError{Kind: ErrSerializationError, Message: message, Err: err}
}

// NewNotFoundError はメッセージが見つからないエラーを生成する。
func NewNotFoundError(message string) *OutboxError {
	return &OutboxError{Kind: ErrNotFound, Message: message}
}

// OutboxStore はアウトボックスメッセージの永続化インターフェース。
type OutboxStore interface {
	// Save はメッセージをアウトボックステーブルに保存する。
	Save(ctx context.Context, msg *OutboxMessage) error
	// FetchPending は処理待ちのメッセージを一覧取得する（最大 limit 件）。
	FetchPending(ctx context.Context, limit int) ([]OutboxMessage, error)
	// Update はメッセージのステータスを更新する。
	Update(ctx context.Context, msg *OutboxMessage) error
	// DeleteDelivered は配信完了メッセージを削除する（保持期間超過後）。
	// 削除した件数を返す。
	DeleteDelivered(ctx context.Context, olderThanDays int) (int64, error)
}

// OutboxPublisher はメッセージを外部に送信するインターフェース。
type OutboxPublisher interface {
	// Publish はメッセージを Kafka に送信する。
	Publish(ctx context.Context, msg *OutboxMessage) error
}
