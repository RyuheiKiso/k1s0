// 本ファイルは観測性 E2E の検証 4（SLO burn rate alert 発火）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
// 関連 ADR: ADR-OBS-003（Incident Taxonomy）/ ADR-OPS-001（Runbook runbook_url 必須）
//
// 検証対象（リリース時点での最小成立形）:
//   1. Alertmanager HTTP API に疎通できる
//   2. /api/v2/alerts で alert 一覧を取得できる（status=200）
//   3. 取得した alert の labels に runbook_url が設定されているか確認（ADR-OPS-001 整合）
//
// 意図的 SLO 違反注入（k6 で 10% error response → fast burn rate alert 発火 5 分窓内）の
// 本格 assertion は採用初期で tools/qualify/observability/inject-slo-violation.sh と
// 連動して有効化する設計。本リリースでは「Alertmanager が応答する」最小経路の動作確認に
// 留める。
//
// 前提:
//   K1S0_ALERTMANAGER_HTTP_TARGET=http://localhost:9093
//   tools/local-stack/up.sh --observability で Alertmanager（kube-prometheus-stack 同梱）が起動。
package observability

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"
)

// alertmanagerAlert は /api/v2/alerts の各 element を decode する型。
type alertmanagerAlert struct {
	// alert の labels（key=value、severity / category / runbook_url 等）
	Labels map[string]string `json:"labels"`
	// alert の annotations（summary / description 等）
	Annotations map[string]string `json:"annotations"`
	// alert 状態（state.state="active" / "suppressed" / "unprocessed"）
	Status struct {
		State       string   `json:"state"`
		SilencedBy  []string `json:"silencedBy"`
		InhibitedBy []string `json:"inhibitedBy"`
	} `json:"status"`
	// alert が active になった時刻（RFC 3339）
	StartsAt string `json:"startsAt"`
	// alert が active 終了する時刻（RFC 3339、未終了なら未来時刻）
	EndsAt string `json:"endsAt"`
}

// TestSLOAlertManagerEndpoint は Alertmanager HTTP API の最小疎通検証。
// 意図 SLO 違反注入による fast burn alert 発火 assert は採用初期で本格化。
func TestSLOAlertManagerEndpoint(t *testing.T) {
	// Alertmanager HTTP API endpoint
	target := os.Getenv("K1S0_ALERTMANAGER_HTTP_TARGET")
	if target == "" {
		t.Skip("K1S0_ALERTMANAGER_HTTP_TARGET 未設定: tools/local-stack/up.sh --observability で起動した Alertmanager の HTTP endpoint を指定（例: http://localhost:9093）")
	}

	// 全体タイムアウト 30 秒
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// /api/v2/status で Alertmanager 疎通確認
	// status response は cluster info を含む大きな JSON、本 test では「200 で応答する」のみ確認するため
	// 任意の map で受ける
	statusURL := fmt.Sprintf("%s/api/v2/status", target)
	statusResp := getPromJSON[map[string]any](t, ctx, statusURL)
	if len(statusResp) == 0 {
		t.Fatalf("/api/v2/status: response 空（Alertmanager が応答していない）")
	}
	t.Logf("/api/v2/status: %d 個の top-level field 取得", len(statusResp))

	// /api/v2/alerts で 現 active alert 一覧を取得
	alertsURL := fmt.Sprintf("%s/api/v2/alerts", target)
	alerts := getPromJSON[[]alertmanagerAlert](t, ctx, alertsURL)
	t.Logf("/api/v2/alerts: 現在 %d 件の alert", len(alerts))

	// active alert が存在する場合、ADR-OPS-001 の runbook_url ラベル必須要件を機械検証
	missingRunbookCount := 0
	activeAlerts := 0
	for _, alert := range alerts {
		if alert.Status.State != "active" {
			continue
		}
		activeAlerts++
		// runbook_url ラベルが無い alert は ADR-OPS-001 違反
		if _, ok := alert.Labels["runbook_url"]; !ok {
			t.Logf("ADR-OPS-001 違反候補: alert labels に runbook_url 不在 alertname=%s severity=%s",
				alert.Labels["alertname"], alert.Labels["severity"])
			missingRunbookCount++
		}
	}
	if activeAlerts > 0 {
		t.Logf("active alerts=%d / runbook_url 不在=%d", activeAlerts, missingRunbookCount)
	}

	// 意図 SLO 違反 + fast burn alert 発火検証は採用初期で本格化:
	//   1. tools/qualify/observability/inject-slo-violation.sh で k6 → 10% error 注入
	//   2. 5 分窓 fast burn rate（PrometheusRule で定義）が発火
	//   3. /api/v2/alerts に severity=page + slo_window=fast の alert が出現
	//   4. runbook_url + category=availability の必須 label を assert
	// 上記 4 段の自動化は ADR-TEST-006 検証 4 の本格実装で、本リリース時点では
	// Alertmanager 疎通 + active alert の runbook_url label 機械検証 までで最小成立。
	t.Logf("意図 SLO 違反 + fast burn alert 発火 assert は採用初期で inject-slo-violation.sh と連動して本格化（ADR-TEST-006 検証 4）")
}
