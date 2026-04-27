// Stock エンティティ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   在庫数量と在庫更新の不変条件（負数禁止 / 同期時刻管理）を Domain 層で保証する。

// Package entity は stock-reconciler のドメインエンティティ。
package entity

// 標準 / 内部 import。
import (
	// エラー文字列整形。
	"fmt"
	// 時刻記録。
	"time"

	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
)

// Stock は SKU 単位の在庫を表すエンティティ。
//
// 内部状態の更新は ApplyDelta / SyncTo メソッドのみで行い、不変条件をエンティティ自身で守る。
type Stock struct {
	// 識別子（SKU）。
	sku value.SKU
	// 現在数量（負数になり得ない）。
	quantity int64
	// 最終同期時刻。
	syncedAt time.Time
}

// NewStock は SKU と初期数量から Stock を生成する。
//
// 数量が負の場合は error を返す（業務不変条件）。
func NewStock(sku value.SKU, quantity int64, syncedAt time.Time) (*Stock, error) {
	// 在庫が負の場合は不正。
	if quantity < 0 {
		// 不変条件違反としてエラーを返す。
		return nil, fmt.Errorf("entity.NewStock: quantity must be >= 0, got %d", quantity)
	}
	// インスタンスを返す。
	return &Stock{sku: sku, quantity: quantity, syncedAt: syncedAt}, nil
}

// SKU は SKU を返す。
func (s *Stock) SKU() value.SKU {
	// 内部の SKU を返す。
	return s.sku
}

// Quantity は現在数量を返す。
func (s *Stock) Quantity() int64 {
	// 内部の数量を返す。
	return s.quantity
}

// SyncedAt は最終同期時刻を返す。
func (s *Stock) SyncedAt() time.Time {
	// 内部の時刻を返す。
	return s.syncedAt
}

// ApplyDelta は数量の差分を適用する。
//
// 差分加算後に負数になる場合は error（出庫超過）。
func (s *Stock) ApplyDelta(delta int64, at time.Time) error {
	// 仮計算する。
	next := s.quantity + delta
	// 在庫が負になるなら拒否する。
	if next < 0 {
		// 出庫超過は不変条件違反。
		return fmt.Errorf("entity.Stock.ApplyDelta: would result in negative quantity %d (current=%d, delta=%d)", next, s.quantity, delta)
	}
	// 数量を更新する。
	s.quantity = next
	// 同期時刻を更新する。
	s.syncedAt = at
	// 正常終了。
	return nil
}

// SyncTo は外部システムから取得した正規値で在庫を上書きする。
//
// 在庫差分検出後の reconcile 適用に使う。負数は拒否する。
func (s *Stock) SyncTo(authoritative int64, at time.Time) error {
	// 負数は許容しない。
	if authoritative < 0 {
		// 不変条件違反。
		return fmt.Errorf("entity.Stock.SyncTo: authoritative must be >= 0, got %d", authoritative)
	}
	// 数量を上書きする。
	s.quantity = authoritative
	// 同期時刻を更新する。
	s.syncedAt = at
	// 正常終了。
	return nil
}

// Diff は他の値との差分を返す（正: 自身が多い / 負: 自身が少ない）。
//
// reconcile での差分検出に使う。
func (s *Stock) Diff(authoritative int64) int64 {
	// 数値差分を返す。
	return s.quantity - authoritative
}
