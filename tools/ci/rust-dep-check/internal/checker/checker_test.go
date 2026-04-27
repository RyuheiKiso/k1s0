// 本ファイルは rust-dep-check の単体テスト。tier 判定 + IsAllowed の golden 検査。
package checker

import "testing"

// SourceTierByPath が Cargo.toml の path から正しく Tier を判定するかを検証。
func TestSourceTierByPath(t *testing.T) {
	cases := []struct {
		rel  string
		want Tier
	}{
		{"src/sdk/rust/Cargo.toml", TierSdk},
		{"src/sdk/rust/crates/k1s0-sdk/Cargo.toml", TierSdk},
		{"src/sdk/rust/crates/k1s0-sdk-proto/Cargo.toml", TierContracts}, // proto-only crate は contracts 扱い
		{"src/sdk/rust/crates/k1s0-sdk-proto", TierContracts},
		{"src/tier1/rust/Cargo.toml", TierTier1},
		{"src/tier1/rust/crates/audit/Cargo.toml", TierTier1},
		{"src/platform/scaffold/Cargo.toml", TierPlatform},
		{"src/contracts/buf.yaml", TierContracts},
		{"tests/fuzz/rust/Cargo.toml", TierTests},
		{"unrelated/Cargo.toml", TierUnknown},
	}
	for _, c := range cases {
		if got := SourceTierByPath(c.rel); got != c.want {
			t.Errorf("SourceTierByPath(%q) = %v, want %v", c.rel, got, c.want)
		}
	}
}

// IsAllowed が一方向ルールを正しく実装しているかを検証。
func TestIsAllowed(t *testing.T) {
	cases := []struct {
		src, target Tier
		want        bool
		desc        string
	}{
		{TierTier1, TierTier1, true, "同 tier"},
		{TierTier1, TierContracts, true, "tier1 → contracts は OK"},
		{TierTier1, TierSdk, false, "tier1 → sdk は禁止"},
		{TierSdk, TierContracts, true, "sdk → contracts は OK"},
		{TierSdk, TierTier1, false, "sdk → tier1 は禁止"},
		{TierContracts, TierSdk, false, "contracts は独立"},
		{TierPlatform, TierTier1, true, "platform → tier1 は許容（雛形 CLI 用途）"},
		{TierUnknown, TierTier1, true, "unknown は判定対象外"},
	}
	for _, c := range cases {
		if got := IsAllowed(c.src, c.target); got != c.want {
			t.Errorf("IsAllowed(%v, %v) = %v, want %v (%s)", c.src, c.target, got, c.want, c.desc)
		}
	}
}
