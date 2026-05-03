// tests/e2e/owner/helpers/k6_runner.go
//
// perf/ + observability/ 検証 4 (SLO 違反注入) で使う k6 spawn ラッパ。
// k6 binary を child process として起動し、JSON summary を parse する。
//
// 設計正典:
//   ADR-TEST-008（perf 部位 + 観測性検証 4）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/10_owner_suite/02_ディレクトリ構造.md
package helpers

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
)

// K6Runner は k6 binary を呼び出すための薄い wrapper
type K6Runner struct {
	// BinaryPath は k6 binary の path（既定: PATH 上の "k6"）
	BinaryPath string
}

// NewK6Runner は default 設定で K6Runner を生成
func NewK6Runner() *K6Runner {
	return &K6Runner{BinaryPath: "k6"}
}

// K6Summary は k6 run --summary-export で出力される JSON 構造（簡略版）
type K6Summary struct {
	// Metrics は metric 名 → 値の map（http_req_duration / http_reqs 等）
	Metrics map[string]K6Metric `json:"metrics"`
	// State は実行状態
	State K6State `json:"state"`
}

// K6Metric は単一 metric の集計値（thresholds / values）
type K6Metric struct {
	Type     string             `json:"type"`
	Contains string             `json:"contains,omitempty"`
	Values   map[string]float64 `json:"values"`
}

// K6State は k6 run の実行状態（test 完了時間等）
type K6State struct {
	TestRunDurationMs float64 `json:"testRunDurationMs"`
}

// Run は k6 script を実行し、--summary-export で取得した JSON を parse して返す。
// scriptPath: k6 JS script path（perf 試験本体）
// summaryPath: --summary-export 出力先（呼び出し側で artifact 化）
func (r *K6Runner) Run(ctx context.Context, scriptPath, summaryPath string) (*K6Summary, error) {
	// k6 run --summary-export=<path> <script>
	cmd := exec.CommandContext(ctx, r.BinaryPath, "run", "--summary-export="+summaryPath, scriptPath)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("k6 run 失敗: %w", err)
	}
	// summary JSON を読み込んで parse
	body, err := os.ReadFile(summaryPath)
	if err != nil {
		return nil, fmt.Errorf("k6 summary 読み込み失敗: %w", err)
	}
	var summary K6Summary
	if err := json.Unmarshal(body, &summary); err != nil {
		return nil, fmt.Errorf("k6 summary decode 失敗: %w", err)
	}
	return &summary, nil
}
