package outbox

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"
)

// OutboxStatus はアウトボックスメッセージのステータス。
type OutboxStatus string

const (
	// OutboxStatusPending は未処理のメッセージ。
	OutboxStatusPending OutboxStatus = "PENDING"
	// OutboxStatusProcessing は処理中のメッセージ。
	OutboxStatusProcessing OutboxStatus = "PROCESSING"
	// OutboxStatusDelivered は配信済みのメッセージ。
	OutboxStatusDelivered OutboxStatus = "DELIVERED"
	// OutboxStatusFailed は失敗したメッセージ。
	OutboxStatusFailed OutboxStatus = "FAILED"
)

// OutboxMessage はアウトボックスパターンで管理するメッセージ。
type OutboxMessage struct {
	// ID はメッセージの一意な識別子。
	ID string
	// Topic は送信先 Kafka トピック名。
	Topic string
	// EventType はイベントの型名。
	EventType string
	// Payload はメッセージペイロード (JSON 文字列)。
	Payload string
	// Status は現在のステータス。
	Status OutboxStatus
	// RetryCount はリトライ回数。
	RetryCount int
	// ScheduledAt は次回処理予定時刻。
	ScheduledAt time.Time
	// CreatedAt は作成時刻。
	CreatedAt time.Time
	// UpdatedAt は更新時刻。
	UpdatedAt time.Time
	// CorrelationId はリクエスト相関 ID。
	CorrelationId string
}

// NewOutboxMessage は新しい OutboxMessage を生成する。
func NewOutboxMessage(topic, eventType, payload, correlationId string) OutboxMessage {
	now := time.Now().UTC()
	return OutboxMessage{
		ID:            uuid.New().String(),
		Topic:         topic,
		EventType:     eventType,
		Payload:       payload,
		Status:        OutboxStatusPending,
		RetryCount:    0,
		ScheduledAt:   now,
		CreatedAt:     now,
		UpdatedAt:     now,
		CorrelationId: correlationId,
	}
}

// NextScheduledAt は次回処理予定時刻を指数バックオフで計算する。
// min(1<<retryCount, 60) * time.Minute
func NextScheduledAt(retryCount int) time.Time {
	delayMinutes := 1 << retryCount // 2^retryCount
	if delayMinutes > 60 {
		delayMinutes = 60
	}
	return time.Now().UTC().Add(time.Duration(delayMinutes) * time.Minute)
}

// CanTransitionTo は現在のステータスから目的のステータスへ遷移可能かを返す。
// Delivered からは遷移不可。
func (s OutboxStatus) CanTransitionTo(next OutboxStatus) bool {
	switch s {
	case OutboxStatusPending:
		return next == OutboxStatusProcessing
	case OutboxStatusProcessing:
		return next == OutboxStatusDelivered || next == OutboxStatusFailed
	case OutboxStatusFailed:
		return next == OutboxStatusPending // リトライ時に Pending に戻す
	case OutboxStatusDelivered:
		return false // Delivered からは遷移不可
	}
	return false
}

// OutboxStoreError はアウトボックスストアのエラー。
type OutboxStoreError struct {
	Op  string
	Err error
}

// Error は OutboxStoreError の文字列表現を返す。
func (e *OutboxStoreError) Error() string {
	return fmt.Sprintf("outbox store %s: %v", e.Op, e.Err)
}

// Unwrap は元のエラーを返す。
func (e *OutboxStoreError) Unwrap() error {
	return e.Err
}

// OutboxStore はアウトボックスメッセージの永続化インターフェース。
type OutboxStore interface {
	// SaveMessage は新しいメッセージを保存する。
	SaveMessage(ctx context.Context, msg OutboxMessage) error
	// GetPendingMessages は処理待ちメッセージを最大 limit 件取得する。
	GetPendingMessages(ctx context.Context, limit int) ([]OutboxMessage, error)
	// UpdateStatus はメッセージのステータスを更新する。
	UpdateStatus(ctx context.Context, id string, status OutboxStatus) error
	// UpdateStatusWithRetry はリトライ情報とともにステータスを更新する。
	UpdateStatusWithRetry(ctx context.Context, id string, status OutboxStatus, retryCount int, scheduledAt time.Time) error
}

// OutboxPublisher はメッセージを外部に送信するインターフェース。
type OutboxPublisher interface {
	// Publish はメッセージを Kafka に送信する。
	Publish(ctx context.Context, msg OutboxMessage) error
}
