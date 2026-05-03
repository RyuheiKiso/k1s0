// tests/e2e/user/examples/tier3_bff_graphql_test.go
//
// examples/tier3-bff-graphql/ の動作確認

//go:build user_e2e

package examples

import (
	"testing"
)

// TestTier3BFFGraphQLRoundtrip は tier3 BFF GraphQL endpoint への query → tier1 round-trip 検証
func TestTier3BFFGraphQLRoundtrip(t *testing.T) {
	t.Skip("PHASE: release-initial, real impl from 採用初期")
}
