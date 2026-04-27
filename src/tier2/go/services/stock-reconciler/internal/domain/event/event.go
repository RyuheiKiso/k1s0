// stock-reconciler のドメインイベント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   reconcile が完了した時に外部に公開するイベント定義。PubSub に流す JSON ペイロードと一致する。

// Package event は stock-reconciler のドメインイベント定義。
package event

// 標準 import。
import (
	// 時刻表現。
	"time"
)

// StockReconciled は在庫同期が完了した時の業務イベント。
//
// PubSub topic `stock.reconciled` に JSON で publish される（CloudEvents の data 部に埋め込み）。
type StockReconciled struct {
	// SKU 文字列（PubSub 受信側がドメイン語彙で扱うため値オブジェクトではなく文字列）。
	SKU string `json:"sku"`
	// reconcile 前の数量。
	BeforeQuantity int64 `json:"before_quantity"`
	// reconcile 後の数量。
	AfterQuantity int64 `json:"after_quantity"`
	// 差分（After - Before）。
	Delta int64 `json:"delta"`
	// reconcile 元の外部システム ID（例: "wms-primary"）。
	SourceSystem string `json:"source_system"`
	// reconcile 完了時刻（UTC）。
	OccurredAt time.Time `json:"occurred_at"`
	// 識別 ID（PubSub 重複排除用、UUID 推奨）。
	EventID string `json:"event_id"`
}

// NewStockReconciled は完了イベントを組み立てる便利コンストラクタ。
//
// caller は EventID を渡す（重複排除のため、ユースケース側で UUID を生成）。
func NewStockReconciled(sku string, before, after int64, source string, eventID string, at time.Time) StockReconciled {
	// 値を構造体に詰めて返す。
	return StockReconciled{
		// SKU 文字列。
		SKU: sku,
		// 旧数量。
		BeforeQuantity: before,
		// 新数量。
		AfterQuantity: after,
		// 差分。
		Delta: after - before,
		// 同期元システム。
		SourceSystem: source,
		// 発生時刻（UTC 正規化は呼出側責務）。
		OccurredAt: at,
		// イベント ID。
		EventID: eventID,
	}
}
