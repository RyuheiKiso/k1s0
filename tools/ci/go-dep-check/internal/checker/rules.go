// 本ファイルは Go 依存方向ルールの本体（許容方向 + 禁止 prefix の写像）。
// 設計正典: docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md
package checker

import "strings"

// Tier は k1s0 リポジトリの Go ソースが属する階層区分。
type Tier string

// 階層区分の列挙。
const (
	TierUnknown   Tier = "unknown"
	TierContracts Tier = "contracts"
	TierSdk       Tier = "sdk"
	TierTier1     Tier = "tier1"
	TierTier2     Tier = "tier2"
	TierTier3     Tier = "tier3"
	TierPlatform  Tier = "platform"
	TierTests     Tier = "tests"
	TierTools     Tier = "tools"
)

// SourceTierByPath はファイル相対 path（リポジトリ root からの相対）から Tier を判定する。
// 走査対象外（docs / .github 等）は TierUnknown を返す。
func SourceTierByPath(rel string) Tier {
	switch {
	case strings.HasPrefix(rel, "src/contracts/"):
		return TierContracts
	case strings.HasPrefix(rel, "src/sdk/go/"):
		return TierSdk
	case strings.HasPrefix(rel, "src/tier1/go/"):
		return TierTier1
	case strings.HasPrefix(rel, "src/tier2/go/"):
		return TierTier2
	case strings.HasPrefix(rel, "src/tier3/bff/"):
		return TierTier3
	case strings.HasPrefix(rel, "src/platform/"):
		return TierPlatform
	case strings.HasPrefix(rel, "tests/"):
		return TierTests
	case strings.HasPrefix(rel, "tools/"):
		return TierTools
	default:
		return TierUnknown
	}
}

// ForbiddenPrefixes は Tier ごとに禁止 import prefix を返す。
// 一方向ルール: tier3 → tier2 → sdk → tier1（→ contracts 独立）。
//
// **重要**: SDK の OSS 公開 module path `github.com/k1s0/sdk-go` は 2 つの実体を内包する。
//   1. `github.com/k1s0/sdk-go/proto/v1/...`  ... buf 生成 proto stub（contracts material）
//   2. `github.com/k1s0/sdk-go/k1s0/...`      ... 高水準 SDK facade（k1s0.Client 等）
// dep direction では (2) のみが「sdk 層」であり、(1) は contracts 同等で他層からの参照を許容する。
// 禁止 prefix は (2) を狙い撃ちする `github.com/k1s0/sdk-go/k1s0` 形式で記述する。
func ForbiddenPrefixes(t Tier) []string {
	switch t {
	case TierContracts:
		// contracts は独立、他層を参照しない（SDK 高水準 facade も generated proto も対象外）
		return []string{
			"github.com/k1s0/k1s0/src/sdk/",
			"github.com/k1s0/k1s0/src/tier1/",
			"github.com/k1s0/k1s0/src/tier2/",
			"github.com/k1s0/k1s0/src/tier3/",
			"github.com/k1s0/sdk-go/k1s0",
		}
	case TierSdk:
		// sdk は contracts 経由のみ、tier1/2/3 を参照しない
		return []string{
			"github.com/k1s0/k1s0/src/tier1/",
			"github.com/k1s0/k1s0/src/tier2/",
			"github.com/k1s0/k1s0/src/tier3/",
		}
	case TierTier1:
		// tier1 は contracts のみ、SDK 高水準 facade（sdk-go/k1s0）/ tier2 / tier3 を参照しない。
		// `github.com/k1s0/sdk-go/proto/...` は generated proto stub（contracts material）として参照可。
		return []string{
			"github.com/k1s0/sdk-go/k1s0",
			"github.com/k1s0/k1s0/src/sdk/go/k1s0",
			"github.com/k1s0/k1s0/src/tier2/",
			"github.com/k1s0/k1s0/src/tier3/",
		}
	case TierTier2:
		// tier2 は sdk / contracts のみ、tier1 / tier3 を参照しない
		return []string{
			"github.com/k1s0/k1s0/src/tier1/",
			"github.com/k1s0/k1s0/src/tier3/",
		}
	case TierTier3:
		// tier3 は tier2 / sdk のみ、tier1 / contracts 直接参照は禁止
		// （contracts は SDK 経由でのみ取得する原則）
		return []string{
			"github.com/k1s0/k1s0/src/tier1/",
			"github.com/k1s0/k1s0/src/contracts/",
		}
	default:
		// platform / tests / tools は dep direction の対象外
		return nil
	}
}
