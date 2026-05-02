// 本ファイルは tier1 公開 12 service のうち tenant_onboarding / audit_pii / payroll で
// カバーしていない service の **OK 限定 PASS** 検証。
//
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
// 関連 ADR: ADR-TEST-001 / ADR-TEST-002 / ADR-TIER1-001 / SHIP_STATUS §9
//
// 厳密化方針:
//   - **err == nil 限定 PASS**（Unimplemented / NotFound / InvalidArgument を許容しない）
//   - dev/CI in-memory backend で **必ず OK を返す** 7 service を対象とする
//     - PubSub.Publish（in-memory queue が必ず offset を返す）
//     - Feature.EvaluateBoolean（in-memory backend が default 値を返す）
//     - Telemetry.EmitMetric（OTel pass-through で必ず OK）
//     - Log.Info（OTel pass-through で必ず OK）
//     - Binding.Invoke（in-memory backend が no-op で OK）
//     - Invoke.Call（in-memory backend が echo で OK）
//     - Workflow.RunShort（BACKEND_DAPR + in-memory Dapr Workflow adapter で OK）
//   - Secrets / ServiceInvoke / Workflow.Query 等の register 必須経路は別 test で対応
//     （payroll_full_flow_test.go 側 Secrets.Encrypt + 本 test 側 Workflow.Start で
//     OK 限定 PASS を達成）
//
// 環境前提:
//   K1S0_TIER1_TARGET=localhost:50001          tier1-facade-state（5 API Router）
//   K1S0_TIER1_WORKFLOW_TARGET=localhost:50005 tier1-facade-workflow（Workflow 専用 Pod）
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

// TestTier1ExtendedServices は tier1 残 7 service の OK 限定 gRPC 疎通検証。
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

	// 5. Binding.Invoke — tier1-state Pod の in-memory Dapr backend は InvokeBinding を
	// no-op で OK 返却する（src/tier1/go/internal/adapter/dapr/inmemory_misc.go 参照）。
	// 採用初期で本番は OPERATOR-001 が定める実 binding component（SMTP / S3 / HTTP）に切替。
	t.Run("binding_invoke", func(t *testing.T) {
		respData, respMeta, err := client.Binding().Invoke(
			ctx,
			"binding.in-memory",
			"create",
			[]byte(`{"e2e":true}`),
			map[string]string{"k1s0-test": "binding-invoke"},
		)
		if err != nil {
			t.Fatalf("Binding.Invoke: %v", err)
		}
		t.Logf("Binding.Invoke: response_len=%d metadata_len=%d", len(respData), len(respMeta))
	})

	// 6. Invoke.Call — tier1-state Pod の in-memory Dapr backend は InvokeMethodWithCustomContent を
	// echo で OK 返却する（呼出 data をそのまま返す）。
	// 採用初期で本番は実 app（test-app Pod / dapr app id 登録）に切替。
	t.Run("invoke_call", func(t *testing.T) {
		// echo 用 payload。in-memory は data をそのまま echo するため round-trip 確認可能。
		input := []byte(`{"echo":"e2e-test"}`)
		respData, _, statusCode, err := client.Invoke().Call(
			ctx,
			"echo-app",
			"echo",
			input,
			"application/json",
			5000,
		)
		if err != nil {
			t.Fatalf("Invoke.Call: %v", err)
		}
		t.Logf("Invoke.Call: response_len=%d status=%d", len(respData), statusCode)
	})

	// 7. Workflow.RunShort — tier1-workflow Pod（K1S0_TIER1_WORKFLOW_TARGET）の
	// BACKEND_DAPR + in-memory Dapr Workflow adapter は workflow_type 未登録でも
	// Start を OK で返す（src/tier1/go/internal/adapter/daprwf/inmemory.go 参照）。
	// 採用初期で本番は実 worker（tier2 RegisterWorkflow）に切替。
	t.Run("workflow_run_short", func(t *testing.T) {
		wfTarget := os.Getenv("K1S0_TIER1_WORKFLOW_TARGET")
		if wfTarget == "" {
			t.Skip("K1S0_TIER1_WORKFLOW_TARGET 未設定")
		}
		// tier1-workflow Pod 直結 Client（state Pod とは Pod 別なため別 Config 要）。
		wfClient, werr := k1s0.New(ctx, k1s0.Config{
			Target:   wfTarget,
			TenantID: "demo-tenant",
			Subject:  "e2e-test/tier1-extended-workflow",
			UseTLS:   false,
		})
		if werr != nil {
			t.Fatalf("k1s0.New(workflow target=%s): %v", wfTarget, werr)
		}
		defer func() { _ = wfClient.Close() }()
		// RunShort で BACKEND_DAPR を明示し in-memory adapter 経路に乗せる。
		retID, runID, err := wfClient.Workflow().RunShort(
			ctx,
			"e2e.echo.workflow",
			"",
			[]byte(`{"e2e":true}`),
			false,
		)
		if err != nil {
			t.Fatalf("Workflow.RunShort: %v", err)
		}
		if retID == "" {
			t.Fatalf("Workflow.RunShort: workflow_id 空")
		}
		t.Logf("Workflow.RunShort: workflow_id=%s run_id=%s", retID, runID)
	})
}
