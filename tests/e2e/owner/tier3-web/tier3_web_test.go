// tests/e2e/owner/tier3-web/tier3_web_test.go
//
// owner suite tier3-web/ — tier3 Web frontend の Keycloak ログイン → BFF → tier1
// round-trip を chromedp (headless Chrome) で検証する。
//
// 利用者向けの TS Playwright 経路は src/sdk/typescript/test-fixtures/ で別提供
// （ADR-TEST-008 §7 二重提供）。
//
// 設計正典:
//   ADR-TEST-008 §1 tier3-web 配置 / §7 二重提供
//   ADR-TEST-010（test-fixtures、TS Playwright fixture）

//go:build owner_e2e

package tier3web

import (
	"testing"
)

// TestTier3WebKeycloakLogin は Keycloak login → tier3 web dashboard 表示の
// E2E flow を chromedp で検証する。
func TestTier3WebKeycloakLogin(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §7 tier3-web 二重提供)")
}

// TestTier3WebBFFRoundtrip は tier3 web → BFF → tier1 facade の HTTP/gRPC round-trip 検証
func TestTier3WebBFFRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (ADR-TEST-008 §7 tier3-web)")
}
