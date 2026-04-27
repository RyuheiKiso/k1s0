// 本ファイルは Rust 依存方向ルール（tier 判定 + 許容方向写像）。
// 設計正典: docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md
package checker

import "strings"

// Tier は k1s0 リポジトリの Rust crate が属する階層区分。
type Tier string

const (
	TierUnknown   Tier = "unknown"
	TierContracts Tier = "contracts"
	TierSdk       Tier = "sdk"
	TierTier1     Tier = "tier1"
	TierPlatform  Tier = "platform"
	TierTests     Tier = "tests"
)

// SourceTierByPath は Cargo.toml / crate path（リポジトリ root からの相対）から Tier を判定する。
//
// **重要**: SDK workspace（src/sdk/rust/）配下には 2 種の crate が同居する:
//   1. k1s0-sdk-proto / k1s0-sdk-*-proto ... buf 生成 proto stub（contracts material）
//   2. k1s0-sdk                          ... 高水準 SDK facade（k1s0::Client 等）
//
// dep direction では (2) のみが「sdk 層」であり、(1) は contracts 同等で他層からの参照を許容する
// （docs/04_概要設計/.../DS-SW-COMP-132 の「tier1 は k1s0-sdk-proto を path 参照、k1s0-sdk は禁止」と一致）。
// よって crate path に `-proto` を含むものは TierContracts として再分類する。
func SourceTierByPath(rel string) Tier {
	rel = strings.ReplaceAll(rel, "\\", "/")
	// (1) SDK workspace 配下の proto-only crate は contracts material 扱い
	if strings.HasPrefix(rel, "src/sdk/rust/crates/") && strings.Contains(rel, "-proto") {
		return TierContracts
	}
	switch {
	case strings.HasPrefix(rel, "src/contracts/"):
		return TierContracts
	case strings.HasPrefix(rel, "src/sdk/rust/"):
		return TierSdk
	case strings.HasPrefix(rel, "src/tier1/rust/"):
		return TierTier1
	case strings.HasPrefix(rel, "src/platform/"):
		return TierPlatform
	case strings.HasPrefix(rel, "tests/"):
		return TierTests
	default:
		return TierUnknown
	}
}

// IsAllowed は src tier から target tier への path 依存が許容されるかを判定する。
// 一方向ルール:
//   - contracts: 独立、他層を path 参照しない
//   - sdk: 同 workspace 内 + contracts のみ参照可
//   - tier1: 同 workspace 内 + contracts のみ参照可（sdk への path 参照は禁止）
//   - platform: contracts / 同 crate のみ参照可
//   - tests / unknown: 緩く許容（CI 主目的は src/* の検証）
func IsAllowed(src, target Tier) bool {
	// 同 tier 内の参照は常に OK（同 workspace の crate 間 path 依存）
	if src == target {
		return true
	}
	// Unknown は判定対象外（外部 OSS / git submodule 等）
	if src == TierUnknown || target == TierUnknown {
		return true
	}
	// contracts は他層を参照しない
	if src == TierContracts {
		return target == TierContracts
	}
	// sdk → contracts のみ
	if src == TierSdk {
		return target == TierContracts
	}
	// tier1 → contracts のみ
	if src == TierTier1 {
		return target == TierContracts
	}
	// platform → contracts / tier1（雛形 CLI が tier1 公開 API を参照する場合あり）
	if src == TierPlatform {
		return target == TierContracts || target == TierTier1
	}
	// tests は src/ への参照を比較的自由に許容
	if src == TierTests {
		return true
	}
	return false
}
