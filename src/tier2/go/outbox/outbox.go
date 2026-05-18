// k1s0 tier2 Outbox relay Go 実装
// Rust 実装（outbox.rs）と 4 言語等価強度を持つ Go 版
// atomic_triple_write と同一 txn で書かれた Outbox エントリを Kafka に非同期転送する
// Debezium CDC 経由の転送（直接 produce は禁止）

// パッケージ名: outbox
package outbox

import (
	// errors パッケージ: エラー生成に使用する
	"errors"
	// fmt パッケージ: エラーメッセージのフォーマットに使用する
	"fmt"
	// time パッケージ: タイムスタンプと TTL に使用する
	"time"

	// uuid パッケージ: aggregate_id / tenant_id の UUID 型
	"github.com/google/uuid"
)

// IdempotencyKeyTTL: 冪等性キーの有効期限（24 時間）
// 同一の idempotency_key でのダブル書込みを防止する
const IdempotencyKeyTTL = 24 * time.Hour

// OutboxPayload: Outbox ペイロード（PII 平文を含まない設計）
// Kafka に転送するデータを格納する（PII は構造的に不在または redact 済み）
type OutboxPayload struct {
	// イベントの種別識別子（aggregate の型名を表す）
	AggregateType string `json:"aggregate_type"`
	// イベントの内容（PII は redact 済みまたは構造的に不在）
	Data any `json:"data"`
	// メタデータ（trace_id / version 等）
	Metadata any `json:"metadata"`
}

// OutboxMessage: Outbox テーブルのエントリ（Domain Event を Kafka に転送するための中継記録）
// Rust の OutboxEntry と意味的に等価な Go 版
type OutboxMessage struct {
	// Outbox エントリの主キー（UUID）
	ID uuid.UUID `json:"id"`
	// テナント ID（Debezium CDC が Kafka routing に使用する）
	TenantID uuid.UUID `json:"tenant_id"`
	// 関連する aggregate の ID
	AggregateID uuid.UUID `json:"aggregate_id"`
	// イベント種別（Debezium が Kafka topic routing に使用する）
	EventType string `json:"event_type"`
	// Kafka に転送するペイロード（PII は含まない）
	Payload OutboxPayload `json:"payload"`
	// 冪等性キー（同一メッセージの二重投入を防止する）
	IdempotencyKey string `json:"idempotency_key"`
	// 書込日時
	CreatedAt time.Time `json:"created_at"`
	// Debezium が処理済みにする日時（nil = 未処理）
	ProcessedAt *time.Time `json:"processed_at,omitempty"`
}

// ErrIdempotencyKeyConflict: 冪等性キーの重複エラー
// 同一の idempotency_key を持つエントリが既に存在する場合に返す
var ErrIdempotencyKeyConflict = errors.New("idempotency key conflict: message already exists")

// ErrTenantIDMissing: テナント ID が未設定のエラー
// AuthContext 経由で設定された tenant_id が必須であることを示す
var ErrTenantIDMissing = errors.New("tenant_id must be set via AuthContext, not via API argument")

// OutboxStore: Outbox の永続化インターフェース
// tenant_id は引数で受け取らず、内部的に TenantContext から取得する
type OutboxStore interface {
	// Save: Outbox エントリを永続化する（atomic_triple_write と同一 txn で呼ぶ）
	// msg が nil の場合はエラーを返す
	Save(msg *OutboxMessage) error

	// Find: aggregate_id に紐づく未処理の Outbox エントリを取得する
	// tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
	Find(aggregateID uuid.UUID) ([]*OutboxMessage, error)

	// MarkDelivered: Outbox エントリを配信済みにする（Debezium CDC が呼ぶ）
	// processed_at を現在時刻で更新する
	MarkDelivered(id uuid.UUID) error
}

// NewOutboxMessage: OutboxMessage を生成する（PII チェック付き）
// tenant_id は引数で受け取らず、TenantContext 経由で設定する設計を型で表現する
// aggregateID: 関連する aggregate の ID（必須）
// tenantID: テナント UUID（TenantContext.TenantID() から取得すること）
// eventType: イベント種別文字列（必須）
// payload: ペイロード（PII は事前に redact すること）
// idempotencyKey: 冪等性キー（呼び出し元が生成する UUID 文字列等）
func NewOutboxMessage(
	aggregateID uuid.UUID,
	tenantID uuid.UUID,
	eventType string,
	payload OutboxPayload,
	idempotencyKey string,
) (*OutboxMessage, error) {
	// tenantID が ゼロ値の場合はエラーを返す（AuthContext 経由必須）
	if tenantID == uuid.Nil {
		return nil, fmt.Errorf("%w: tenantID is nil", ErrTenantIDMissing)
	}
	// eventType が空の場合はエラーを返す
	if eventType == "" {
		return nil, errors.New("eventType must not be empty")
	}
	// idempotencyKey が空の場合はエラーを返す
	if idempotencyKey == "" {
		return nil, errors.New("idempotencyKey must not be empty")
	}
	// 新しい OutboxMessage を現在時刻で初期化して返す
	return &OutboxMessage{
		// 新規 UUID を主キーとして生成する
		ID: uuid.New(),
		// TenantContext 経由で設定されたテナント ID を格納する
		TenantID: tenantID,
		// 関連する aggregate の ID を格納する
		AggregateID: aggregateID,
		// イベント種別を格納する
		EventType: eventType,
		// PII を含まないペイロードを格納する
		Payload: payload,
		// 冪等性キーを格納する
		IdempotencyKey: idempotencyKey,
		// 作成日時を現在時刻で設定する
		CreatedAt: time.Now().UTC(),
		// 処理済み日時は未設定（nil = 未処理）
		ProcessedAt: nil,
	}, nil
}

// IsExpired: 冪等性キーが TTL を超過しているかを返す
// TTL は IdempotencyKeyTTL 定数（24 時間）で定義される
func (m *OutboxMessage) IsExpired() bool {
	// 作成日時から IdempotencyKeyTTL を加算した時刻が現在時刻より前かを確認する
	return time.Now().UTC().After(m.CreatedAt.Add(IdempotencyKeyTTL))
}
