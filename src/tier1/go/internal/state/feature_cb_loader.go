// 本ファイルは Feature Circuit Breaker rule の JSON ファイルからの読み込み実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md
//     - FR-T1-FEATURE-003 受け入れ基準: Prometheus クエリを条件に指定可能
//
// 役割:
//   ConfigMap mount された rules.json から FeatureCBRule の配列を読み込む。
//   形式は flat な JSON 配列で、entry は flag_key / promql / threshold /
//   comparator / recover_after_seconds / forced_false の 6 フィールド。

package state

import (
	"encoding/json"
	"fmt"
	"os"
	"time"
)

// featureCBRuleJSON は rules.json 上の 1 件分スキーマ。
//
// recover_after_seconds は省略可（既定 5 分）。comparator も省略可（既定 "gt"）。
type featureCBRuleJSON struct {
	FlagKey             string  `json:"flag_key"`
	PromQL              string  `json:"promql"`
	Threshold           float64 `json:"threshold"`
	Comparator          string  `json:"comparator,omitempty"`
	RecoverAfterSeconds int64   `json:"recover_after_seconds,omitempty"`
	ForcedFalse         bool    `json:"forced_false,omitempty"`
}

// LoadFeatureCBRulesFromFile は rules.json から FeatureCBRule のスライスを読み込む。
// path が空文字 / "off" の場合は空スライスと nil error を返す（無効化経路）。
// JSON parse 失敗 / 必須フィールド不足は error を返し、cmd 側で fatal にする
// （誤設定を見逃さない）。
func LoadFeatureCBRulesFromFile(path string) ([]FeatureCBRule, error) {
	if path == "" || path == "off" {
		return nil, nil
	}
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("tier1/feature: read cb rules file %q: %w", path, err)
	}
	var entries []featureCBRuleJSON
	if err := json.Unmarshal(raw, &entries); err != nil {
		return nil, fmt.Errorf("tier1/feature: parse cb rules file %q: %w", path, err)
	}
	rules := make([]FeatureCBRule, 0, len(entries))
	for i, e := range entries {
		// 必須フィールド検証。flag_key と promql は両方必要。
		if e.FlagKey == "" {
			return nil, fmt.Errorf("tier1/feature: rule[%d]: flag_key required", i)
		}
		if e.PromQL == "" {
			return nil, fmt.Errorf("tier1/feature: rule[%d]: promql required", i)
		}
		// recover_after は秒で受け取り、Duration に変換する。0 は既定 5 分にフォールバック。
		recover := time.Duration(e.RecoverAfterSeconds) * time.Second
		if recover <= 0 {
			recover = 5 * time.Minute
		}
		// comparator は "gt" / "lt" のみ許容。空は "gt" にフォールバック、それ以外はエラー。
		comparator := e.Comparator
		switch comparator {
		case "":
			comparator = "gt"
		case "gt", "lt":
			// OK
		default:
			return nil, fmt.Errorf("tier1/feature: rule[%d]: unsupported comparator %q (use gt or lt)", i, comparator)
		}
		rules = append(rules, FeatureCBRule{
			FlagKey:      e.FlagKey,
			PromQL:       e.PromQL,
			Threshold:    e.Threshold,
			Comparator:   comparator,
			RecoverAfter: recover,
			ForcedFalse:  e.ForcedFalse,
		})
	}
	return rules, nil
}
