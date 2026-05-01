// k1s0 Log ラッパー。
//
// SDK の LogClient.Send を per-request tenant 伝搬付きで露出する。
// 重大度は BFF JSON 公開用に文字列で受け、SDK 内部の logv1.Severity に変換する。
// SDK 型を BFF API 表面に漏らさないため、本ラッパーが境界となる。

package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// SDK 高水準 facade（Severity 定数）。
	"github.com/k1s0/sdk-go/k1s0"
)

// LogSeverity は BFF API 公開用の重大度文字列（OTel SeverityNumber と整合）。
type LogSeverity string

// LogSeverity の便利定数。
const (
	// TRACE は最も詳細な診断情報。
	LogSeverityTrace LogSeverity = "TRACE"
	// DEBUG はデバッグ用。
	LogSeverityDebug LogSeverity = "DEBUG"
	// INFO は通常運用時。
	LogSeverityInfo LogSeverity = "INFO"
	// WARN は注意が必要な事象。
	LogSeverityWarn LogSeverity = "WARN"
	// ERROR は処理失敗。
	LogSeverityError LogSeverity = "ERROR"
	// FATAL は復旧不能な障害。
	LogSeverityFatal LogSeverity = "FATAL"
)

// toSDKSeverity は BFF 公開文字列を SDK 内部 Severity に写像する。
// 未知の値は INFO にフォールバックする（不正な severity でログ送信が落ちないため）。
func toSDKSeverity(s LogSeverity) k1s0.Severity {
	switch s {
	case LogSeverityTrace:
		return k1s0.SeverityTrace
	case LogSeverityDebug:
		return k1s0.SeverityDebug
	case LogSeverityInfo:
		return k1s0.SeverityInfo
	case LogSeverityWarn:
		return k1s0.SeverityWarn
	case LogSeverityError:
		return k1s0.SeverityError
	case LogSeverityFatal:
		return k1s0.SeverityFatal
	default:
		return k1s0.SeverityInfo
	}
}

// LogSend は単一エントリ送信。
func (c *Client) LogSend(ctx context.Context, severity LogSeverity, body string, attributes map[string]string) error {
	// SDK facade を呼ぶ。
	return c.client.Log().Send(withTenantFromRequest(ctx), toSDKSeverity(severity), body, attributes)
}
