// Channel 値オブジェクトの単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

package value

// 標準 import。
import (
	// テスト frameworks。
	"testing"
)

// TestNewChannel_KnownValues は既知のチャネル文字列を受け入れることを検証する。
func TestNewChannel_KnownValues(t *testing.T) {
	// 受け入れ可能な入力。
	cases := []struct {
		// 入力。
		in string
		// 期待値。
		want Channel
	}{
		// email。
		{in: "email", want: ChannelEmail},
		// 大文字混在も受け入れる。
		{in: "EMAIL", want: ChannelEmail},
		// 前後空白も受け入れる。
		{in: "  slack  ", want: ChannelSlack},
		// webhook。
		{in: "webhook", want: ChannelWebhook},
	}
	// 各ケースを実行する。
	for _, tc := range cases {
		// サブテスト。
		t.Run(tc.in, func(t *testing.T) {
			// 生成する。
			got, err := NewChannel(tc.in)
			// エラーは想定外。
			if err != nil {
				// 失敗。
				t.Fatalf("NewChannel(%q) error: %v", tc.in, err)
			}
			// 等値判定。
			if !got.Equal(tc.want) {
				// 不一致は失敗。
				t.Errorf("NewChannel(%q) = %q, want %q", tc.in, got, tc.want)
			}
		})
	}
}

// TestNewChannel_UnknownRejected は未知のチャネルを拒否することを検証する。
func TestNewChannel_UnknownRejected(t *testing.T) {
	// 不正な入力。
	cases := []string{
		// 空文字。
		"",
		// 未知の値。
		"sms",
		// 表記揺れも未知扱い。
		"e-mail",
		// 日本語。
		"メール",
	}
	// 各ケースを実行する。
	for _, in := range cases {
		// サブテスト。
		t.Run(in, func(t *testing.T) {
			// 生成を試みる。
			_, err := NewChannel(in)
			// エラーが返るべき。
			if err == nil {
				// nil は失敗。
				t.Errorf("NewChannel(%q) expected error, got nil", in)
			}
		})
	}
}
