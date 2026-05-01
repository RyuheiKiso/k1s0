// テストヘルパ: 14 サービス分の Facade method を no-op でデフォルト実装する基底型。
//
// 設計動機:
//   テストごとに必要な method だけ override したいが、Facade には 17 method があり、
//   毎回全件記述するのは煩雑。embed 用の基底型を 1 つ用意して、テスト用 fake が
//   この型を embed すると、未 override の method は no-op (zero value 応答) となる。

package graphql

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 時刻。
	"time"

	// k1s0client の軽量構造体型を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// unimplementedFacade は Facade interface の全 method を no-op (zero value 応答) で実装する。
// テスト用 fake が embed することで、未 override method は安全に呼ばれて zero value を返す。
type unimplementedFacade struct{}

// State.Get を no-op 実装する。
func (unimplementedFacade) StateGet(_ context.Context, _, _ string) ([]byte, string, bool, error) {
	return nil, "", false, nil
}

// State.Save を no-op 実装する。
func (unimplementedFacade) StateSave(_ context.Context, _, _ string, _ []byte) (string, error) {
	return "", nil
}

// State.Delete を no-op 実装する。
func (unimplementedFacade) StateDelete(_ context.Context, _, _, _ string) error {
	return nil
}

// PubSub.Publish を no-op 実装する。
func (unimplementedFacade) PubSubPublish(_ context.Context, _ string, _ []byte, _, _ string, _ map[string]string) (int64, error) {
	return 0, nil
}

// Secrets.Get を no-op 実装する。
func (unimplementedFacade) SecretsGet(_ context.Context, _ string) (map[string]string, int32, error) {
	return nil, 0, nil
}

// Secrets.Rotate を no-op 実装する。
func (unimplementedFacade) SecretsRotate(_ context.Context, _ string, _ int32, _ string) (int32, int32, error) {
	return 0, 0, nil
}

// Decision.Evaluate を no-op 実装する。
func (unimplementedFacade) DecisionEvaluate(_ context.Context, _, _ string, _ []byte, _ bool) ([]byte, []byte, int64, error) {
	return nil, nil, 0, nil
}

// Workflow.Start を no-op 実装する。
func (unimplementedFacade) WorkflowStart(_ context.Context, _, _ string, _ []byte, _ bool) (string, string, error) {
	return "", "", nil
}

// Invoke.Call を no-op 実装する。
func (unimplementedFacade) InvokeCall(_ context.Context, _, _ string, _ []byte, _ string, _ int32) ([]byte, string, int32, error) {
	return nil, "", 0, nil
}

// Audit.Record を no-op 実装する。
func (unimplementedFacade) AuditRecord(_ context.Context, _, _, _, _ string, _ map[string]string, _ string) (string, error) {
	return "", nil
}

// Audit.Query を no-op 実装する。
func (unimplementedFacade) AuditQuery(_ context.Context, _, _ time.Time, _ map[string]string, _ int32) ([]k1s0client.AuditEventSummary, error) {
	return nil, nil
}

// Log.Send を no-op 実装する。
func (unimplementedFacade) LogSend(_ context.Context, _ k1s0client.LogSeverity, _ string, _ map[string]string) error {
	return nil
}

// Telemetry.EmitMetric を no-op 実装する。
func (unimplementedFacade) TelemetryEmitMetric(_ context.Context, _ []k1s0client.MetricPoint) error {
	return nil
}

// PII.Classify を no-op 実装する。
func (unimplementedFacade) PiiClassify(_ context.Context, _ string) ([]k1s0client.PiiFindingSummary, bool, error) {
	return nil, false, nil
}

// PII.Mask を no-op 実装する。
func (unimplementedFacade) PiiMask(_ context.Context, _ string) (string, []k1s0client.PiiFindingSummary, error) {
	return "", nil, nil
}

// Feature.EvaluateBoolean を no-op 実装する。
func (unimplementedFacade) FeatureEvaluateBoolean(_ context.Context, _ string, _ map[string]string) (bool, string, string, error) {
	return false, "", "", nil
}

// Binding.Invoke を no-op 実装する。
func (unimplementedFacade) BindingInvoke(_ context.Context, _, _ string, _ []byte, _ map[string]string) ([]byte, map[string]string, error) {
	return nil, nil, nil
}
