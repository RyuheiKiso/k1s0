// Stock エンティティの単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

package entity

// 標準 / 内部 import。
import (
	// テスト frameworks。
	"testing"
	// 時刻記録。
	"time"

	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
)

// TestNewStock_ValidatesNegativeQuantity は負数の初期値を拒否することを検証する。
func TestNewStock_ValidatesNegativeQuantity(t *testing.T) {
	// SKU を組み立てる。
	sku := value.MustNewSKU("ABC-123")
	// 0 と正数は OK。
	if _, err := NewStock(sku, 0, time.Now()); err != nil {
		// 0 は不変条件を満たすため、エラーは想定外。
		t.Errorf("NewStock with 0 should succeed, got: %v", err)
	}
	// 負数は NG。
	if _, err := NewStock(sku, -1, time.Now()); err == nil {
		// nil は失敗扱い。
		t.Error("NewStock with -1 should fail, got nil")
	}
}

// TestStock_ApplyDelta_RejectsNegativeResult は出庫超過を拒否することを検証する。
func TestStock_ApplyDelta_RejectsNegativeResult(t *testing.T) {
	// SKU を組み立てる。
	sku := value.MustNewSKU("ABC-123")
	// 在庫 5 で初期化する。
	stock, err := NewStock(sku, 5, time.Now())
	// 初期化は成功するはず。
	if err != nil {
		// 想定外。
		t.Fatalf("NewStock failed: %v", err)
	}
	// 在庫 - 3 = 2 は OK。
	if err := stock.ApplyDelta(-3, time.Now()); err != nil {
		// 想定外。
		t.Errorf("ApplyDelta(-3) should succeed, got: %v", err)
	}
	// 結果が 2 になることを確認する。
	if stock.Quantity() != 2 {
		// 想定外。
		t.Errorf("Quantity after -3 should be 2, got %d", stock.Quantity())
	}
	// さらに -3 すると -1 になり拒否される。
	if err := stock.ApplyDelta(-3, time.Now()); err == nil {
		// nil は失敗。
		t.Error("ApplyDelta(-3) on quantity=2 should fail, got nil")
	}
	// 拒否されたので数量は変化しないこと。
	if stock.Quantity() != 2 {
		// 状態が壊れているなら失敗。
		t.Errorf("Quantity should remain 2 after rejected ApplyDelta, got %d", stock.Quantity())
	}
}

// TestStock_SyncTo_RejectsNegative は authoritative の負数を拒否する。
func TestStock_SyncTo_RejectsNegative(t *testing.T) {
	// 初期 5 で構築。
	sku := value.MustNewSKU("ABC-123")
	// 在庫を組み立てる。
	stock, _ := NewStock(sku, 5, time.Now())
	// 負数 SyncTo は拒否。
	if err := stock.SyncTo(-1, time.Now()); err == nil {
		// nil は失敗。
		t.Error("SyncTo(-1) should fail, got nil")
	}
}

// TestStock_Diff は差分計算を検証する。
func TestStock_Diff(t *testing.T) {
	// SKU。
	sku := value.MustNewSKU("ABC-123")
	// 在庫 10。
	stock, _ := NewStock(sku, 10, time.Now())
	// 自身が多い: 10 - 7 = 3。
	if got := stock.Diff(7); got != 3 {
		// 不一致。
		t.Errorf("Diff(7) = %d, want 3", got)
	}
	// 自身が少ない: 10 - 15 = -5。
	if got := stock.Diff(15); got != -5 {
		// 不一致。
		t.Errorf("Diff(15) = %d, want -5", got)
	}
	// 同値: 10 - 10 = 0。
	if got := stock.Diff(10); got != 0 {
		// 不一致。
		t.Errorf("Diff(10) = %d, want 0", got)
	}
}
