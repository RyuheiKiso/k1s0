// notification-hub のドメインイベント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

// Package event は notification-hub のドメインイベント定義。
package event

// 標準 import。
import (
	// 時刻表現。
	"time"
)

// NotificationDispatched は通知配信が完了したイベント。
//
// 配信成否の audit 用に Audit Service へ流すか、metric として publish する。
// リリース時点 では本構造体は内部 logging のみで利用、リリース時点 で Audit / metrics に拡張。
type NotificationDispatched struct {
	// 通知の一意 ID（idempotency key 兼用）。
	NotificationID string `json:"notification_id"`
	// チャネル文字列（"email" / "slack" / "webhook"）。
	Channel string `json:"channel"`
	// 受信者識別子。
	Recipient string `json:"recipient"`
	// 配信に使った Binding Component 名。
	BindingName string `json:"binding_name"`
	// 配信成否（true=成功 / false=失敗）。
	Success bool `json:"success"`
	// 失敗時のエラーメッセージ（成功時は空）。
	ErrorMessage string `json:"error_message,omitempty"`
	// 配信時刻（UTC）。
	OccurredAt time.Time `json:"occurred_at"`
}
