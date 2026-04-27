// Stock Repository インタフェース。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   Domain 層から見た永続化境界。Infrastructure 層の k1s0 State / RDB 実装が本 interface を実装する。
//   依存性逆転により Domain は Infrastructure を知らない。

// Package repository は Domain 層の永続化境界。
package repository

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// Stock エンティティ。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/entity"
	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
)

// StockRepository は Stock の永続化を抽象化する。
type StockRepository interface {
	// FindBySKU は SKU で Stock を取得する。未存在なら ErrNotFound（domain.repository.ErrNotFound）。
	FindBySKU(ctx context.Context, sku value.SKU) (*entity.Stock, error)
	// Save は Stock を保存する（新規 / 更新は実装側で判定）。楽観的排他は ETag で実装側が扱う。
	Save(ctx context.Context, stock *entity.Stock) error
}

// ErrNotFound は FindBySKU で対象が見つからなかった時に返すセンチネル。
//
// utility 化のため Domain 層に固定値として配置する（Infrastructure 層からも参照される）。
var ErrNotFound = errNotFoundSentinel{}

// errNotFoundSentinel は ErrNotFound のための型。
type errNotFoundSentinel struct{}

// Error は標準 error interface 実装。
func (errNotFoundSentinel) Error() string {
	// 固定文字列。
	return "stock repository: not found"
}
