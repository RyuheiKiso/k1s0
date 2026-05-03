// tests/e2e/user/examples/tier3_web_portal_test.go
//
// examples/tier3-web-portal/ の動作確認。利用者の Playwright 経路は
// src/sdk/typescript/test-fixtures/ で別提供（ADR-TEST-010 領域 5）。
// 本 Go test は backend (tier3 BFF) の HTTP 経路のみ検証する。

//go:build user_e2e

package examples

import (
	"testing"
)

// TestTier3WebPortalBackend は tier3 web portal の BFF 経路を Go test の HTTP client で検証
func TestTier3WebPortalBackend(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期")
}
