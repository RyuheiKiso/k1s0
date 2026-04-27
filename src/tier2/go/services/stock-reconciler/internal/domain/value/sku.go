// SKU 値オブジェクト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   在庫管理単位（Stock Keeping Unit）の識別子を表現する。
//   不変・自己バリデーション・等値性を SKU 自身で保証する（プリミティブ漏洩を防ぐ）。

// Package value は stock-reconciler のドメイン値オブジェクト。
package value

// 標準 import。
import (
	// 文字列整形。
	"fmt"
	// 正規表現。
	"regexp"
	// 文字列処理。
	"strings"
)

// skuPattern は SKU の許容形式（英大文字 + 数字 + ハイフン、3〜32 文字）。
//
// 業務ルールではなく実装側の lower bound（運用側組織が独自命名規約を持てるよう緩めに設定）。
var skuPattern = regexp.MustCompile(`^[A-Z0-9][A-Z0-9-]{2,31}$`)

// SKU は在庫管理単位の識別子。生成は NewSKU 経由のみ（フィールドを export しない）。
type SKU struct {
	// 内部値（バリデーション済）。
	value string
}

// NewSKU は文字列から SKU を生成する。
//
// バリデーション失敗時は error を返し、不正な SKU が Domain 層に流入することを防ぐ。
func NewSKU(s string) (SKU, error) {
	// 前後空白を取り除く（人間入力からの保護）。
	trimmed := strings.TrimSpace(s)
	// 大文字に正規化する（小文字 SKU は同一視）。
	upper := strings.ToUpper(trimmed)
	// パターンに合致するかを検査する。
	if !skuPattern.MatchString(upper) {
		// 不正値は明示的にエラー。
		return SKU{}, fmt.Errorf("value.NewSKU: invalid SKU format: %q", s)
	}
	// 正規化済の値で SKU を返す。
	return SKU{value: upper}, nil
}

// MustNewSKU はテスト用の panic 版コンストラクタ。
//
// production code では使わないこと（エラーハンドリングをスキップするため）。
func MustNewSKU(s string) SKU {
	// 通常の NewSKU を試みる。
	sku, err := NewSKU(s)
	// エラーは panic で検査時に検出する。
	if err != nil {
		// テストの failure として顕在化させる。
		panic(err)
	}
	// 正常値を返す。
	return sku
}

// String は人間可読表現を返す（Stringer interface 実装）。
func (s SKU) String() string {
	// 内部値をそのまま返す。
	return s.value
}

// Equal は SKU 同士の等値判定。
//
// 構造体が unexported field のみを持つため `==` 演算子も等価だが、
// 利用側がインタフェース経由で比較する際の意図を明示する。
func (s SKU) Equal(other SKU) bool {
	// 値の同一性で等値とする。
	return s.value == other.value
}
