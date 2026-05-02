// 本ファイルはテナント新規オンボーディング E2E シナリオ。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 (Test Pyramid + testcontainers) / ADR-TEST-002 (E2E 自動化)
//
// 前提: tools/local-stack/up.sh --role tier1-go-dev (or 同等以上の role) で kind cluster
// + 本番再現スタック起動済。t1-state / t1-secret / t1-workflow / Dapr / CNPG / Strimzi
// Kafka / Valkey / OpenBao / Temporal が namespace 内で running 状態。
//
// 本テストは「リリース時点」の最小貫通検証として、kind cluster 上の tier1 namespace に
// Running Pod が存在することを確認する。tier1 公開 API への gRPC 呼び出し
// （CreateTenant / RegisterUser / Login 等）は採用初期で順次拡張する。
package scenarios

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestTenantOnboarding は tier1 → tier2 → tier3 を貫通する最小フローを検証する。
// 並列実行不可（cluster 状態を共有するため、t.Parallel() を呼ばない）。
//
// 検証段階:
//   - リリース時点: tier1 namespace に Running Pod が存在することを確認（本実装）
//   - 採用初期: tier1 公開 API（State.Set / Get、Workflow.Start 等）への gRPC 疎通追加
//   - 採用後の運用拡大時: tier2 BFF + tier3 Web を経由する完全 chain 検証
func TestTenantOnboarding(t *testing.T) {
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	ctx, cancel := context.WithTimeout(context.Background(), 90*time.Second)
	defer cancel()

	// tier1 系 namespace の Running Pod を確認。
	// local-stack が起動する tier1 namespace は role により異なるが、最低限
	// dapr-system は role=tier1-* / sdk-dev / infra-ops / full でデプロイされる。
	tier1Namespaces := []string{"dapr-system"}

	for _, ns := range tier1Namespaces {
		t.Run(ns, func(t *testing.T) {
			running, err := cluster.WaitForRunningPodInNamespace(ctx, ns, 60*time.Second)
			if err != nil {
				t.Fatalf("namespace %q で Running Pod を待機中に失敗: %v", ns, err)
			}
			t.Logf("namespace %q: %d Running Pod 確認", ns, running)
		})
	}
}
