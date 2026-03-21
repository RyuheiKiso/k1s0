package uuid

import (
	"regexp"
	"testing"
)

// UUID v4 フォーマットの正規表現パターン。
// xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx の形式を検証する。
// バリアント bits は 8, 9, a, b のいずれかである必要がある。
var uuidV4Pattern = regexp.MustCompile(
	`^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`,
)

// TestNew_Format は New() が UUID v4 の正しいフォーマットを返すことを検証する。
func TestNew_Format(t *testing.T) {
	// UUID v4 のフォーマット要件: バージョンビットが 4、バリアントビットが 10xx であること。
	got := New()
	if !uuidV4Pattern.MatchString(got) {
		t.Errorf("New() = %q はUUID v4フォーマットに一致しません（期待パターン: %s）", got, uuidV4Pattern.String())
	}
}

// TestNew_Uniqueness は New() が一意な UUID を生成することを検証する。
// 100 件生成して重複がないことを確認する。
func TestNew_Uniqueness(t *testing.T) {
	// 一意性の検証: 暗号論的乱数を使用しているため100件程度では衝突は発生しないはずである。
	const count = 100
	seen := make(map[string]struct{}, count)
	for i := range count {
		id := New()
		if _, exists := seen[id]; exists {
			t.Errorf("New() が %d 件目で重複したUUIDを生成しました: %q", i+1, id)
		}
		seen[id] = struct{}{}
	}
}

// TestNew_Version は New() が生成するUUIDのバージョンビットが 4 であることを検証する。
func TestNew_Version(t *testing.T) {
	// UUID の 14 文字目（インデックス 14）がバージョン番号 '4' であることを確認する。
	// フォーマット: xxxxxxxx-xxxx-4xxx-... の '4' の位置
	got := New()
	if len(got) != 36 {
		t.Fatalf("New() の長さが不正: got %d, want 36", len(got))
	}
	if got[14] != '4' {
		t.Errorf("New() のバージョンビットが不正: got %c, want '4'（UUID: %s）", got[14], got)
	}
}

// TestNew_Variant は New() が生成するUUIDのバリアントビットが RFC 4122 準拠であることを検証する。
func TestNew_Variant(t *testing.T) {
	// RFC 4122 のバリアント: ビット列が 10xx であること。
	// UUID の 19 文字目（インデックス 19）が 8, 9, a, b のいずれかであることを確認する。
	got := New()
	if len(got) != 36 {
		t.Fatalf("New() の長さが不正: got %d, want 36", len(got))
	}
	variantChar := got[19]
	validVariants := map[byte]bool{'8': true, '9': true, 'a': true, 'b': true}
	if !validVariants[variantChar] {
		t.Errorf("New() のバリアントビットが不正: got %c, want one of [89ab]（UUID: %s）", variantChar, got)
	}
}

// TestNew_Length は New() が常に 36 文字の UUID を返すことを検証する。
func TestNew_Length(t *testing.T) {
	// UUID の標準的な長さは 36 文字（8-4-4-4-12 形式 + 4つのハイフン）。
	for range 10 {
		got := New()
		if len(got) != 36 {
			t.Errorf("New() の長さが不正: got %d, want 36（UUID: %s）", len(got), got)
		}
	}
}
