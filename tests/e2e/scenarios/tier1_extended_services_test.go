// 本ファイルは tier1 公開 12 service のうち tenant_onboarding / audit_pii / payroll で
// カバーしていない service の **OK 限定 PASS** 検証。
//
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 / ADR-TEST-002 / ADR-TIER1-001 / SHIP_STATUS §9
//
// 厳密化方針:
//   - **err == nil 限定 PASS**（Unimplemented / NotFound / InvalidArgument を許容しない）
//   - dev/CI in-memory backend で **必ず OK を返す** 4 service のみを対象とする
//     - PubSub.Publish（in-memory queue が必ず offset を返す）
//     - Feature.EvaluateBoolean（in-memory backend が default 値を返す）
//     - Telemetry.EmitMetric（OTel pass-through で必ず OK）
//     - Log.Info（OTel pass-through で必ず OK）
//   - Workflow.Start / Secrets.Get / ServiceInvoke.Call / Binding.Invoke は seed/register が
//     必要なため、**本 test の射程外**とする。採用初期で seed 整備後に別 test として実装する
//     旨を docs/40_運用ライフサイクル/e2e-results.md で明示する。
//
// 環境前提:
//   K1S0_TIER1_TARGET=localhost:50001  tier1-facade-state（5 API Router）
package scenarios

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/k1s0/sdk-go/k1s0"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"

	"github.com/k1s0/k1s0/tests/e2e/helpers"
)

// TestTier1ExtendedServices は tier1 残 4 service の OK 限定 gRPC 疎通検証。
func TestTier1ExtendedServices(t *testing.T) {
	cluster := helpers.SetupCluster(t)
	defer cluster.Teardown(t)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	target := os.Getenv("K1S0_TIER1_TARGET")
	if target == "" {
		t.Skip("K1S0_TIER1_TARGET 未設定")
	}

	client, err := k1s0.New(ctx, k1s0.Config{
		Target:   target,
		TenantID: "demo-tenant",
		Subject:  "e2e-test/tier1-extended",
		UseTLS:   false,
	})
	if err != nil {
		t.Fatalf("k1s0.New: %v", err)
	}
	defer func() { _ = client.Close() }()

	// 1. PubSub.Publish — in-memory backend が必ず offset を返す
	t.Run("pubsub_publish", func(t *testing.T) {
		offset, err := client.PubSub().Publish(ctx, "pubsub.in-memory", []byte(`{"e2e":true}`), "application/json")
		if err != nil {
			t.Fatalf("PubSub.Publish: %v", err)
		}
		t.Logf("PubSub.Publish: offset=%d", offset)
	})

	// 2. Feature.EvaluateBoolean — in-memory backend が default false を返す
	t.Run("feature_evaluate_boolean", func(t *testing.T) {
		value, variant, reason, err := client.Feature().EvaluateBoolean(ctx, "e2e.test.flag", nil)
		if err != nil {
			t.Fatalf("Feature.EvaluateBoolean: %v", err)
		}
		t.Logf("Feature.EvaluateBoolean: value=%v variant=%s reason=%s", value, variant, reason)
	})

	// 3. Telemetry.EmitMetric — OTel pass-through で必ず OK
	t.Run("telemetry_emit_metric", func(t *testing.T) {
		err := client.Telemetry().EmitMetric(ctx, []*telemetryv1.Metric{
			{Name: "e2e.test.counter", Kind: telemetryv1.MetricKind_COUNTER, Value: 1.0},
		})
		if err != nil {
			t.Fatalf("Telemetry.EmitMetric: %v", err)
		}
		t.Logf("Telemetry.EmitMetric: OK")
	})

	// 4. Log.Info — OTel pass-through で必ず OK
	t.Run("log_info", func(t *testing.T) {
		err := client.Log().Info(ctx, "e2e tier1-extended-test", map[string]string{"e2e": "true"})
		if err != nil {
			t.Fatalf("Log.Info: %v", err)
		}
		t.Logf("Log.Info: OK")
	})
}
