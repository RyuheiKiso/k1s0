// 本ファイルは観測性 E2E の検証 5（Grafana dashboard goldenfile test）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
//
// 検証対象:
//   infra/observability/grafana/dashboards/*.json の各 dashboard が
//   tests/e2e/observability/dashboard-goldenfile/baselines/<name>.json と完全一致することを assert。
//   panel 数 / query 内の PromQL / threshold 値の意図しない破壊を機械検出する。
//
// 採用初期で baseline 更新フロー（dashboard JSON 編集 PR で baseline も同 commit で更新）
// を CODEOWNERS に組み込む。
package observability

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

// TestDashboardGoldenfile は dashboard JSON と baseline JSON の構造的 diff を取る。
func TestDashboardGoldenfile(t *testing.T) {
	// 実 dashboard 配置先
	dashboardsDir := "../../../infra/observability/grafana/dashboards"
	// baseline 配置先（本 test と相対）
	baselinesDir := "dashboard-goldenfile/baselines"

	matches, err := filepath.Glob(filepath.Join(dashboardsDir, "*.json"))
	if err != nil {
		t.Fatalf("Glob: %v", err)
	}
	if len(matches) == 0 {
		t.Fatalf("dashboards ディレクトリに JSON が無い（%s）", dashboardsDir)
	}

	for _, dashboardPath := range matches {
		name := filepath.Base(dashboardPath)
		t.Run(name, func(t *testing.T) {
			baselinePath := filepath.Join(baselinesDir, name)

			dashboardBytes, err := os.ReadFile(dashboardPath)
			if err != nil {
				t.Fatalf("read dashboard %s: %v", dashboardPath, err)
			}
			baselineBytes, err := os.ReadFile(baselinePath)
			if err != nil {
				t.Fatalf("read baseline %s（baseline 不在は意図的更新の場合 baselines/ に commit が必要）: %v", baselinePath, err)
			}

			// 構造的比較: JSON を Marshal で正規化（key 順 / 空白の差を吸収）
			dashboardCanon, err := canonicalizeJSON(dashboardBytes)
			if err != nil {
				t.Fatalf("canonicalize dashboard: %v", err)
			}
			baselineCanon, err := canonicalizeJSON(baselineBytes)
			if err != nil {
				t.Fatalf("canonicalize baseline: %v", err)
			}

			if !bytes.Equal(dashboardCanon, baselineCanon) {
				t.Fatalf("%s: dashboard JSON と baseline が不一致（意図的更新なら baselines/ に commit して PR レビュー）", name)
			}
			t.Logf("%s: baseline と一致（panel / query / threshold 不変）", name)
		})
	}
}

// canonicalizeJSON は JSON を decode → encode で key 順を正規化する。
func canonicalizeJSON(b []byte) ([]byte, error) {
	var v any
	if err := json.Unmarshal(b, &v); err != nil {
		return nil, err
	}
	return json.Marshal(v)
}
