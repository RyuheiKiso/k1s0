// k1s0 PII ラッパー。
//
// SDK の PiiClient.Classify / Mask を per-request tenant 伝搬付きで露出する。
// SDK の proto PiiFinding 型を露出させないため、軽量構造体 PiiFindingSummary に詰め替える。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// PiiFindingSummary は BFF JSON 応答用の PII 検出結果。
type PiiFindingSummary struct {
	// 検出された PII 種別（NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等）。
	Type string
	// 文字列内の開始位置（0 始まり、文字単位）。
	Start int32
	// 文字列内の終了位置（exclusive）。
	End int32
	// 検出器の信頼度（0.0〜1.0）。
	Confidence float64
}

// PiiClassify はテキスト中の PII を分類する（マスクはせず、検出のみ）。
func (c *Client) PiiClassify(ctx context.Context, text string) (findings []PiiFindingSummary, containsPii bool, err error) {
	// SDK facade を呼ぶ。
	raw, contains, err := c.client.Pii().Classify(withTenantFromRequest(ctx), text)
	if err != nil {
		return nil, false, err
	}
	// proto 型を BFF 用構造体に詰め替える。
	out := make([]PiiFindingSummary, 0, len(raw))
	for _, f := range raw {
		out = append(out, PiiFindingSummary{
			Type:       f.GetType(),
			Start:      f.GetStart(),
			End:        f.GetEnd(),
			Confidence: f.GetConfidence(),
		})
	}
	return out, contains, nil
}

// PiiMask はテキスト中の PII をマスクして返す。findings には検出位置も入る。
func (c *Client) PiiMask(ctx context.Context, text string) (maskedText string, findings []PiiFindingSummary, err error) {
	// SDK facade を呼ぶ。
	masked, raw, err := c.client.Pii().Mask(withTenantFromRequest(ctx), text)
	if err != nil {
		return "", nil, err
	}
	// proto 型を BFF 用構造体に詰め替える。
	out := make([]PiiFindingSummary, 0, len(raw))
	for _, f := range raw {
		out = append(out, PiiFindingSummary{
			Type:       f.GetType(),
			Start:      f.GetStart(),
			End:        f.GetEnd(),
			Confidence: f.GetConfidence(),
		})
	}
	return masked, out, nil
}
