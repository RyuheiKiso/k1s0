// 本ファイルは go-dep-check の単体テスト。tier 判定 + 禁止 prefix 検証を golden で検査する。
package checker

import "testing"

// SourceTierByPath が path prefix から正しく Tier を識別するかを検証。
func TestSourceTierByPath(t *testing.T) {
	cases := []struct {
		rel  string
		want Tier
	}{
		{"src/contracts/buf.yaml", TierContracts},
		{"src/sdk/go/k1s0/client.go", TierSdk},
		{"src/tier1/go/cmd/state/main.go", TierTier1},
		{"src/tier2/go/services/foo/main.go", TierTier2},
		{"src/tier3/bff/cmd/portal-bff/main.go", TierTier3},
		{"src/platform/scaffold/Cargo.toml", TierPlatform},
		{"tests/integration/x_test.go", TierTests},
		{"tools/ci/go-dep-check/cmd/go-dep-check/main.go", TierTools},
		{"docs/INDEX.md", TierUnknown},
	}
	for _, c := range cases {
		if got := SourceTierByPath(c.rel); got != c.want {
			t.Errorf("SourceTierByPath(%q) = %v, want %v", c.rel, got, c.want)
		}
	}
}

// ForbiddenPrefixes が tier ごとに正しい禁止集合を返すかを検証。
func TestForbiddenPrefixes(t *testing.T) {
	cases := []struct {
		tier         Tier
		mustForbid   []string // 含まれているべき
		mustNotMatch []string // 含まれていないべき（許容方向）
	}{
		{
			tier:         TierTier1,
			mustForbid:   []string{"github.com/k1s0/sdk-go/k1s0", "github.com/k1s0/k1s0/src/tier2/", "github.com/k1s0/k1s0/src/tier3/"},
			mustNotMatch: []string{"github.com/k1s0/k1s0/src/contracts/", "github.com/k1s0/sdk-go/proto"},
		},
		{
			tier:         TierTier3,
			mustForbid:   []string{"github.com/k1s0/k1s0/src/tier1/", "github.com/k1s0/k1s0/src/contracts/"},
			mustNotMatch: []string{"github.com/k1s0/k1s0/src/tier2/", "github.com/k1s0/sdk-go/k1s0"},
		},
		{
			tier:       TierContracts,
			mustForbid: []string{"github.com/k1s0/k1s0/src/sdk/", "github.com/k1s0/sdk-go/k1s0"},
		},
	}
	for _, c := range cases {
		got := ForbiddenPrefixes(c.tier)
		gotSet := make(map[string]bool, len(got))
		for _, p := range got {
			gotSet[p] = true
		}
		for _, p := range c.mustForbid {
			if !gotSet[p] {
				t.Errorf("Tier %v should forbid %q but does not (got %v)", c.tier, p, got)
			}
		}
		for _, p := range c.mustNotMatch {
			if gotSet[p] {
				t.Errorf("Tier %v should NOT forbid %q but does (got %v)", c.tier, p, got)
			}
		}
	}
}
