// 通知チャネル値オブジェクト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   email / slack / webhook 等の通知チャネルを enum 風の値オブジェクトで表現する。
//   生文字列のドメイン層への漏出を防ぎ、未知のチャネルを早期に拒否する。

// Package value は notification-hub のドメイン値オブジェクト。
package value

// 標準 import。
import (
	// 文字列整形。
	"fmt"
	// 文字列処理。
	"strings"
)

// Channel は通知チャネルの値オブジェクト。
type Channel struct {
	// 内部値（バリデーション済の小文字文字列）。
	v string
}

// 既知のチャネル定数。
//
// 値が enum 風に固定されることを保証する。新規追加はテストと一緒に追加すること。
var (
	// ChannelEmail は email チャネル（SMTP / SES 等）。
	ChannelEmail = Channel{v: "email"}
	// ChannelSlack は Slack 通知。
	ChannelSlack = Channel{v: "slack"}
	// ChannelWebhook は汎用 HTTP webhook。
	ChannelWebhook = Channel{v: "webhook"}
)

// allowedChannels は受理する文字列値の集合。
var allowedChannels = map[string]Channel{
	// email。
	"email": ChannelEmail,
	// slack。
	"slack": ChannelSlack,
	// webhook。
	"webhook": ChannelWebhook,
}

// NewChannel は文字列から Channel を生成する。
//
// 大文字小文字混在を許容するが、内部値は小文字に正規化する。
func NewChannel(s string) (Channel, error) {
	// 前後空白除去 + 小文字化。
	normalized := strings.ToLower(strings.TrimSpace(s))
	// 既知の集合を引く。
	ch, ok := allowedChannels[normalized]
	// 未知の値は拒否する。
	if !ok {
		// VALIDATION 相当のエラー。
		return Channel{}, fmt.Errorf("value.NewChannel: unknown channel %q", s)
	}
	// 値オブジェクトを返す。
	return ch, nil
}

// String は人間可読表現を返す。
func (c Channel) String() string {
	// 内部値を返す。
	return c.v
}

// Equal は他の Channel との等値判定。
func (c Channel) Equal(other Channel) bool {
	// 内部値の同一性。
	return c.v == other.v
}
