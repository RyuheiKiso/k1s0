// Notification エンティティ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   送信単位の通知を表す。チャネル / 受信者 / 件名 / 本文 / 任意メタデータを保持する。

// Package entity は notification-hub のドメインエンティティ。
package entity

// 標準 / 内部 import。
import (
	// 文字列整形。
	"fmt"
	// 文字列処理。
	"strings"

	// Channel 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/domain/value"
)

// Notification は 1 件の通知。
type Notification struct {
	// 通知チャネル。
	channel value.Channel
	// 受信者識別子（email アドレス / Slack channel / webhook URL）。
	recipient string
	// 件名（チャネル次第で省略可だが API 側で必須化）。
	subject string
	// 本文（チャネル次第でフォーマット差異あり）。
	body string
	// 任意メタデータ（テンプレ展開後のキー値ペア、Binding metadata に流す）。
	metadata map[string]string
}

// NewNotification は値の整合性をチェックして Notification を生成する。
func NewNotification(channel value.Channel, recipient, subject, body string, metadata map[string]string) (*Notification, error) {
	// recipient は必須。
	if strings.TrimSpace(recipient) == "" {
		// 業務不変条件違反。
		return nil, fmt.Errorf("entity.NewNotification: recipient is required")
	}
	// subject も必須（運用要件）。
	if strings.TrimSpace(subject) == "" {
		// 業務不変条件違反。
		return nil, fmt.Errorf("entity.NewNotification: subject is required")
	}
	// body も必須。
	if strings.TrimSpace(body) == "" {
		// 業務不変条件違反。
		return nil, fmt.Errorf("entity.NewNotification: body is required")
	}
	// metadata の nil ガード。
	if metadata == nil {
		// 空 map に正規化する。
		metadata = map[string]string{}
	}
	// インスタンスを返す。
	return &Notification{
		// チャネル。
		channel: channel,
		// 受信者。
		recipient: recipient,
		// 件名。
		subject: subject,
		// 本文。
		body: body,
		// メタデータ。
		metadata: metadata,
	}, nil
}

// Channel は通知チャネルを返す。
func (n *Notification) Channel() value.Channel {
	// 内部値を返す。
	return n.channel
}

// Recipient は受信者識別子を返す。
func (n *Notification) Recipient() string {
	// 内部値を返す。
	return n.recipient
}

// Subject は件名を返す。
func (n *Notification) Subject() string {
	// 内部値を返す。
	return n.subject
}

// Body は本文を返す。
func (n *Notification) Body() string {
	// 内部値を返す。
	return n.body
}

// Metadata はメタデータを返す（呼出側は読取専用として扱う）。
func (n *Notification) Metadata() map[string]string {
	// 内部 map をそのまま露出する。
	return n.metadata
}
