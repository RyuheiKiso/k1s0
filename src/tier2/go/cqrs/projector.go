// k1s0 tier2 CQRS Go インターフェース定義
// Rust 実装（cqrs/src/projector.rs）と 4 言語等価強度を持つ Go 版
// Outbox リレー経由で受信したドメインイベントを読み取りモデルに投影する（設計方針 15）

// パッケージ名: cqrs
package cqrs

import (
	// context パッケージ: リクエストスコープに使用する
	"context"
	// encoding/json パッケージ: ペイロードの JSON 型に使用する
	"encoding/json"

	// uuid パッケージ: イベント識別子およびテナント識別子型に使用する
	"github.com/google/uuid"
)

// DomainEvent: ドメインイベントの構造体定義（Outbox から受信する共通エンベロープ形式）
// Rust の DomainEvent 構造体に対応する
type DomainEvent struct {
	// イベント一意識別子（UUID v4）
	ID uuid.UUID `json:"id"`
	// テナント識別子（RLS 述語の基底 / AuthContext からのみ注入する）
	TenantID uuid.UUID `json:"tenant_id"`
	// 集約型名（業界中立語のみ使用可）
	AggregateType string `json:"aggregate_type"`
	// イベント種別名
	EventType string `json:"event_type"`
	// ペイロード（JSON Value 形式 / PII フィールドは Outbox 書込前に redact 済み）
	Payload json.RawMessage `json:"payload"`
	// HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
	HlcTimestamp uint64 `json:"hlc_timestamp"`
}

// ReadModelProjector: 読み取りモデル投影器インターフェース
// Rust の ReadModelProjector トレイトに対応する
// すべての読み取りモデル投影器が実装する契約
// Project メソッドはドメインイベントを受け取り読み取りモデルを更新する
type ReadModelProjector interface {
	// Project: ドメインイベントを受け取り読み取りモデルを更新する
	// 失敗した場合は error を返す（Outbox リレーがリトライする）
	Project(ctx context.Context, event *DomainEvent) error

	// HandledEventTypes: 投影器が処理対象とするイベント種別の一覧を返す
	// 登録済み投影器のルーティングに使用する
	HandledEventTypes() []string
}

// ReadModelRegistry: 複数の ReadModelProjector を集約するレジストリ
// イベント種別に基づいて対応する投影器へルーティングする
// Rust の ReadModelRegistry 構造体に対応する
type ReadModelRegistry struct {
	// 登録済み投影器のリスト（ReadModelProjector インターフェース形式で保持する）
	projectors []ReadModelProjector
}

// NewReadModelRegistry: 新規レジストリを生成する（空状態から開始する）
func NewReadModelRegistry() *ReadModelRegistry {
	// 空のレジストリを初期化する
	return &ReadModelRegistry{
		projectors: make([]ReadModelProjector, 0),
	}
}

// Register: 投影器をレジストリに登録する
func (r *ReadModelRegistry) Register(projector ReadModelProjector) {
	// 投影器リストに追加する
	r.projectors = append(r.projectors, projector)
}

// Dispatch: ドメインイベントを対応する投影器へルーティングして投影する
func (r *ReadModelRegistry) Dispatch(ctx context.Context, event *DomainEvent) error {
	// 登録済み投影器を順に検索して対象イベント種別を処理する
	for _, projector := range r.projectors {
		// 投影器がこのイベント種別を処理するか確認する
		if containsEventType(projector.HandledEventTypes(), event.EventType) {
			// 対応投影器に処理を委譲する
			if err := projector.Project(ctx, event); err != nil {
				// 投影エラーを返す（Outbox リレーがリトライする）
				return err
			}
		}
	}
	// 全投影器の処理成功
	return nil
}

// containsEventType: スライスにイベント種別が含まれるか確認するヘルパー関数
func containsEventType(types []string, eventType string) bool {
	// スライスを線形探索する
	for _, t := range types {
		// 一致するイベント種別が見つかった場合は true を返す
		if t == eventType {
			return true
		}
	}
	// 一致なし
	return false
}
