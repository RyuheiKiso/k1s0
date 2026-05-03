// tests/e2e/user/examples/tier2_dotnet_service_test.go
//
// examples/tier2-dotnet-service/ の動作確認

//go:build user_e2e

package examples

import (
	"testing"
)

// TestTier2DotNetServiceRoundtrip は dotnet example を起動 → SDK で /health 確認
func TestTier2DotNetServiceRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期")
}
