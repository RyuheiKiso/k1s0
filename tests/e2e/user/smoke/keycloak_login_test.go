// tests/e2e/user/smoke/keycloak_login_test.go
//
// Keycloak OIDC login が user suite の minimum stack で動くことを確認する smoke。
//
// 設計正典:
//   ADR-SEC-001（Keycloak）
//   ADR-TEST-008 §1

//go:build user_e2e

package smoke

import (
	"testing"
)

// TestKeycloakLoginNormal は test-fixtures の Keycloak admin login で access token を取得し、
// /realms/k1s0 が応答することを確認する。
func TestKeycloakLoginNormal(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (test-fixtures Keycloak helper)")
}
