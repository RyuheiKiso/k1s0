// tests/e2e/owner/helpers/chromedp_session.go
//
// tier3-web/ で使う headless Chrome 駆動 helper（chromedp）。
// owner suite の tier3-web 検証は Go test + chromedp で行う設計（ADR-TEST-008 §7、二重提供）。
// 利用者向けの Playwright 経路は src/sdk/typescript/test-fixtures/ で別提供。
//
// 設計正典:
//   ADR-TEST-008 §7（テスト言語、tier3-web 二重提供）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
package helpers

import (
	"context"
	"time"
)

// ChromedpSession は chromedp.Context を wrap した browser session。
// 採用初期で chromedp 依存を go.mod に追加した時に本構造を完成させる。
// リリース時点では skeleton として interface のみ宣言する（依存追加コストの段階展開）。
type ChromedpSession struct {
	// Ctx は chromedp の root context（NewExecAllocator で生成、cancel func を持つ）
	Ctx context.Context
	// Cancel は session 終了時に呼ぶ cancel function
	Cancel context.CancelFunc
}

// NewChromedpSession は headless Chrome を起動して ChromedpSession を返す。
// 採用初期で chromedp.NewExecAllocator + chromedp.NewContext を統合する。
func NewChromedpSession(_ time.Duration) (*ChromedpSession, error) {
	// リリース時点では実装未着手（chromedp 依存追加 + WebSocket 経路整備が前提）。
	// 採用初期で github.com/chromedp/chromedp を go.mod に追加し、
	// chromedp.NewExecAllocator(... headless: true ...) → chromedp.NewContext(...) で実装する。
	return nil, nil
}

// Close は chromedp Ctx の cancel を呼んで browser process を終了する
func (s *ChromedpSession) Close() {
	if s != nil && s.Cancel != nil {
		s.Cancel()
	}
}
