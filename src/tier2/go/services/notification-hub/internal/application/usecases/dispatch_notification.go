// 通知配信ユースケース。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   通知の入力（チャネル / 受信者 / 件名 / 本文 / メタデータ）を受け、
//   チャネルに応じた Dapr Binding Component を選択して invoke する。

// Package usecases は Application 層のユースケース実装。
package usecases

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// crypto/rand で ID 生成。
	"crypto/rand"
	// JSON シリアライズ。
	"encoding/json"
	// 16 進エンコード。
	"encoding/hex"
	// 文字列整形。
	"fmt"
	// 現在時刻。
	"time"

	// Notification エンティティ。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/domain/entity"
	// Channel 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/domain/value"
	// 設定（BindingsConfig）。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/config"
	// k1s0 SDK ラッパー。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/infrastructure/external"
	// tier2 共通エラー。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
)

// BindingInvoker は k1s0 Binding 呼出の最小 interface（モック容易性のため抽象化）。
type BindingInvoker interface {
	// BindingInvoke は Dapr Binding Component の Invoke を呼ぶ。
	BindingInvoke(ctx context.Context, name, operation string, data []byte, metadata map[string]string) ([]byte, map[string]string, error)
}

// DispatchUseCase は通知配信ユースケース本体。
type DispatchUseCase struct {
	// k1s0 SDK ラッパー（Application 層からは BindingInvoker interface 越しに見える）。
	binding BindingInvoker
	// チャネル別 Binding Component 名（config から渡される）。
	bindings config.BindingsConfig
	// 時刻取得関数（テスト容易性のため注入可能）。
	now func() time.Time
}

// NewDispatchUseCase は UseCase を組み立てる。
func NewDispatchUseCase(k1s0Client *external.K1s0Client, bindings config.BindingsConfig) *DispatchUseCase {
	// 構造体を組み立てる。
	return &DispatchUseCase{
		// k1s0 ラッパーを BindingInvoker として保持する。
		binding: k1s0Client,
		// チャネル別 Binding Component 名。
		bindings: bindings,
		// 既定では UTC 現在時刻。
		now: func() time.Time { return time.Now().UTC() },
	}
}

// DispatchInput は通知配信の入力 DTO。
type DispatchInput struct {
	// チャネル文字列（"email" / "slack" / "webhook"）。
	Channel string
	// 受信者識別子。
	Recipient string
	// 件名。
	Subject string
	// 本文。
	Body string
	// 任意メタデータ。
	Metadata map[string]string
}

// DispatchResult は通知配信の出力 DTO。
type DispatchResult struct {
	// 通知 ID。
	NotificationID string
	// 使用した Binding Component 名。
	BindingName string
	// チャネル文字列。
	Channel string
	// 配信成否。
	Success bool
}

// bindingPayload は Binding Component に渡す JSON ペイロードの統一スキーマ。
//
// 各チャネルの Component は本スキーマを受け取り、内部で SMTP / Slack / HTTP の形式に変換する。
type bindingPayload struct {
	// 受信者識別子。
	To string `json:"to"`
	// 件名。
	Subject string `json:"subject"`
	// 本文。
	Body string `json:"body"`
}

// Execute は 1 件の通知配信を実施する。
//
// 処理順序:
//
//	1. 入力バリデーション（Channel / Recipient / Subject / Body）
//	2. Channel から Binding Component 名を解決
//	3. ペイロードを JSON 化して Binding.Invoke を呼ぶ
//	4. 通知 ID と結果を返す
func (u *DispatchUseCase) Execute(ctx context.Context, in DispatchInput) (*DispatchResult, error) {
	// チャネルを Domain 値オブジェクトに変換する。
	channel, err := value.NewChannel(in.Channel)
	// 不正なチャネルは VALIDATION。
	if err != nil {
		// caller に DomainError を返す。
		return nil, t2errors.Wrap(t2errors.CategoryValidation, "E-T2-NOTIF-001", "invalid channel", err)
	}
	// Notification を組み立てる（subject/body の必須チェック含む）。
	notif, err := entity.NewNotification(channel, in.Recipient, in.Subject, in.Body, in.Metadata)
	// 不正値は VALIDATION。
	if err != nil {
		// caller に DomainError を返す。
		return nil, t2errors.Wrap(t2errors.CategoryValidation, "E-T2-NOTIF-002", "invalid notification fields", err)
	}
	// チャネルに対応する Binding Component 名を解決する。
	bindingName, err := u.resolveBinding(notif.Channel())
	// 解決失敗は INTERNAL（設定漏れ）。
	if err != nil {
		// caller に DomainError を返す。
		return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-NOTIF-010", "binding not configured for channel", err)
	}
	// 通知 ID を生成する。
	notifID, idErr := newNotificationID()
	// 生成失敗は INTERNAL。
	if idErr != nil {
		// caller に DomainError。
		return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-NOTIF-011", "failed to generate notification id", idErr)
	}
	// Binding 用 payload を組み立てる。
	payload := bindingPayload{
		// 受信者識別子。
		To: notif.Recipient(),
		// 件名。
		Subject: notif.Subject(),
		// 本文。
		Body: notif.Body(),
	}
	// JSON にエンコードする。
	data, err := json.Marshal(payload)
	// 失敗は INTERNAL。
	if err != nil {
		// caller に DomainError。
		return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-NOTIF-012", "failed to marshal payload", err)
	}
	// Binding metadata を組み立てる（Component 側で拡張属性として参照される）。
	metadata := mergeMetadata(notif.Metadata(), map[string]string{
		// 通知 ID（Binding 側で重複排除に利用可能）。
		"notification_id": notifID,
		// チャネル種別（Component 側ログ用）。
		"channel": notif.Channel().String(),
	})
	// Binding.Invoke を呼ぶ（operation はチャネル統一して "create" を採用、Component 側が内部で送信処理にマッピング）。
	if _, _, invokeErr := u.binding.BindingInvoke(ctx, bindingName, "create", data, metadata); invokeErr != nil {
		// 配信失敗は UPSTREAM。
		return nil, t2errors.Wrap(t2errors.CategoryUpstream, "E-T2-NOTIF-013", "binding invoke failed", invokeErr)
	}
	// 結果を組み立てて返す。
	return &DispatchResult{
		// 通知 ID。
		NotificationID: notifID,
		// 使用した Binding Component 名。
		BindingName: bindingName,
		// チャネル文字列。
		Channel: notif.Channel().String(),
		// 配信成功。
		Success: true,
	}, nil
}

// resolveBinding は Channel から Binding Component 名を返す。
func (u *DispatchUseCase) resolveBinding(ch value.Channel) (string, error) {
	// チャネルで分岐する。
	switch {
	// email チャネル。
	case ch.Equal(value.ChannelEmail):
		// 設定値が空なら未設定エラー。
		if u.bindings.Email == "" {
			// caller でログ + alert。
			return "", fmt.Errorf("email binding not configured")
		}
		// 設定値を返す。
		return u.bindings.Email, nil
	// slack チャネル。
	case ch.Equal(value.ChannelSlack):
		// 設定値チェック。
		if u.bindings.Slack == "" {
			// 未設定エラー。
			return "", fmt.Errorf("slack binding not configured")
		}
		// 設定値を返す。
		return u.bindings.Slack, nil
	// webhook チャネル。
	case ch.Equal(value.ChannelWebhook):
		// 設定値チェック。
		if u.bindings.Webhook == "" {
			// 未設定エラー。
			return "", fmt.Errorf("webhook binding not configured")
		}
		// 設定値を返す。
		return u.bindings.Webhook, nil
	// 不明なチャネル（Channel 値オブジェクトで早期に弾けるはずだが防御的に分岐）。
	default:
		// 想定外。
		return "", fmt.Errorf("unsupported channel %q", ch.String())
	}
}

// mergeMetadata は base / extra をコピーしたうえでマージした map を返す。
//
// 同一キーの場合は extra が優先する。base の元 map は変更しない（不変方針）。
func mergeMetadata(base, extra map[string]string) map[string]string {
	// 出力 map を初期化する。
	out := make(map[string]string, len(base)+len(extra))
	// base をコピーする。
	for k, v := range base {
		// キー値をコピーする。
		out[k] = v
	}
	// extra で上書きする。
	for k, v := range extra {
		// キー値を上書きする。
		out[k] = v
	}
	// 出力 map を返す。
	return out
}

// newNotificationID は uuid v4 風の 32 hex 文字列を生成する。
func newNotificationID() (string, error) {
	// 16 byte バッファ。
	b := make([]byte, 16)
	// crypto/rand で埋める。
	if _, err := rand.Read(b); err != nil {
		// 生成失敗は呼出元へ。
		return "", err
	}
	// hex 文字列に変換して返す。
	return hex.EncodeToString(b), nil
}
