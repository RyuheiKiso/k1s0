// tests/e2e/user/examples/tier3_native_maui_test.go
//
// examples/tier3-native-maui/ の動作確認（sim 環境で MAUI app を起動 + SDK 呼び出し）

//go:build user_e2e

package examples

import (
	"testing"
)

// TestTier3NativeMAUIRoundtrip は MAUI app 起動 + SDK State.Get round-trip 検証
func TestTier3NativeMAUIRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期 (MAUI sim 起動が必要)")
}
