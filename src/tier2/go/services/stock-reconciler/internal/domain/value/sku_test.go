// SKU 値オブジェクトの単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   SKU の不変条件（前後空白除去 / 大文字正規化 / パターン拒否）を network / DB なしで網羅検証する。

package value

// 標準 import。
import (
	// テスト frameworks。
	"testing"
)

// TestNewSKU_ValidPatterns は正常系を網羅する。
func TestNewSKU_ValidPatterns(t *testing.T) {
	// テーブル駆動。
	cases := []struct {
		// テスト名。
		name string
		// 入力。
		input string
		// 期待される正規化値。
		want string
	}{
		// 大文字英数字 + ハイフン。
		{name: "uppercase alnum hyphen", input: "ABC-123", want: "ABC-123"},
		// 小文字を大文字に正規化する。
		{name: "lowercase normalized to upper", input: "abc-123", want: "ABC-123"},
		// 前後空白を除去する。
		{name: "trim whitespace", input: "  ABC-XYZ  ", want: "ABC-XYZ"},
		// 最短 3 文字。
		{name: "min length 3", input: "ABC", want: "ABC"},
		// 最長 32 文字。
		{name: "max length 32", input: "A1234567890123456789012345678901", want: "A1234567890123456789012345678901"},
	}
	// 各ケースを実行する。
	for _, tc := range cases {
		// 名前付きサブテスト。
		t.Run(tc.name, func(t *testing.T) {
			// SKU を生成する。
			got, err := NewSKU(tc.input)
			// 正常系では error nil。
			if err != nil {
				// 期待外。
				t.Fatalf("NewSKU(%q) error: %v", tc.input, err)
			}
			// 値を比較する。
			if got.String() != tc.want {
				// 不一致は失敗。
				t.Errorf("NewSKU(%q) = %q, want %q", tc.input, got.String(), tc.want)
			}
		})
	}
}

// TestNewSKU_InvalidPatterns は異常系を網羅する。
func TestNewSKU_InvalidPatterns(t *testing.T) {
	// 不正パターン。
	cases := []struct {
		// テスト名。
		name string
		// 入力。
		input string
	}{
		// 空文字。
		{name: "empty", input: ""},
		// 先頭ハイフン（パターン違反）。
		{name: "leading hyphen", input: "-ABC"},
		// 1 文字（最短 3 違反）。
		{name: "too short 1", input: "A"},
		// 2 文字（最短 3 違反）。
		{name: "too short 2", input: "AB"},
		// 33 文字（最長 32 違反）。
		{name: "too long", input: "AB12345678901234567890123456789012"},
		// アンダースコア禁止。
		{name: "underscore", input: "ABC_123"},
		// 日本語禁止。
		{name: "japanese", input: "在庫A"},
		// 空白のみ。
		{name: "whitespace only", input: "   "},
	}
	// 各ケースを実行する。
	for _, tc := range cases {
		// 名前付きサブテスト。
		t.Run(tc.name, func(t *testing.T) {
			// SKU 生成を試みる。
			_, err := NewSKU(tc.input)
			// 異常系では error が返るべき。
			if err == nil {
				// nil なら失敗。
				t.Errorf("NewSKU(%q) expected error, got nil", tc.input)
			}
		})
	}
}

// TestSKU_Equal は等値判定を検証する。
func TestSKU_Equal(t *testing.T) {
	// 大文字小文字混在で生成する。
	a := MustNewSKU("abc-123")
	// 別表記で同値の SKU を生成する。
	b := MustNewSKU("  ABC-123  ")
	// 異なる値の SKU を生成する。
	c := MustNewSKU("XYZ-999")
	// a == b は true。
	if !a.Equal(b) {
		// 大文字正規化と空白除去の効果が出ているか確認する。
		t.Errorf("expected %q == %q after normalization", a, b)
	}
	// a == c は false。
	if a.Equal(c) {
		// 別 SKU 同士は false。
		t.Errorf("expected %q != %q", a, c)
	}
}
