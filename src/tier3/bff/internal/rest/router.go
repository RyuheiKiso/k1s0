// 本ファイルは portal-bff / admin-bff 共通の REST エンドポイント定義の起点。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// 設計:
//   tier1 14 公開サービスへの薄い REST 露出を提供する。エンドポイントはサービス別ファイル
//   (state.go / pubsub.go / ...) に分割し、本ファイルは共通の構造体・interface・helper
//   を集約する。Router は Facade interface に依存し、テストでは in-memory mock を渡せる。
//
// 認可:
//   /api/* は呼出側 BFF (portal-bff / admin-bff) で auth middleware を被せている前提。
//   本パッケージ内では認可を扱わない。

// Package rest は portal-bff / admin-bff 共通の REST エンドポイントを提供する。
package rest

// 標準 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード / デコード。
	"encoding/json"
	// HTTP server。
	"net/http"
	// 時刻（AuditQuery で使う）。
	"time"

	// k1s0client の AuditEventSummary / PiiFindingSummary / MetricPoint / LogSeverity を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// Facade は Router が必要とする tier1 14 サービスの最小 interface。
// 実体は k1s0client.Client が満たすが、test では in-memory mock を渡せる。
//
// SDK 型を BFF JSON 表面に漏らさないため、AuditEventSummary / PiiFindingSummary /
// MetricPoint / LogSeverity 等は k1s0client パッケージで定義した軽量構造体を使う。
type Facade interface {
	// State.
	StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error)
	StateSave(ctx context.Context, store, key string, data []byte) (etag string, err error)
	StateDelete(ctx context.Context, store, key, expectedEtag string) error
	// PubSub.
	PubSubPublish(ctx context.Context, topic string, data []byte, contentType, idempotencyKey string, metadata map[string]string) (offset int64, err error)
	// Secrets.
	SecretsGet(ctx context.Context, name string) (values map[string]string, version int32, err error)
	SecretsRotate(ctx context.Context, name string, gracePeriodSec int32, idempotencyKey string) (newVersion, previousVersion int32, err error)
	// Decision.
	DecisionEvaluate(ctx context.Context, ruleID, ruleVersion string, inputJSON []byte, includeTrace bool) (outputJSON, traceJSON []byte, elapsedUs int64, err error)
	// Workflow.
	WorkflowStart(ctx context.Context, workflowType, workflowID string, input []byte, idempotent bool) (returnedID, runID string, err error)
	// Invoke.
	InvokeCall(ctx context.Context, appID, method string, data []byte, contentType string, timeoutMs int32) (responseData []byte, responseContentType string, status int32, err error)
	// Audit.
	AuditRecord(ctx context.Context, actor, action, resource, outcome string, attributes map[string]string, idempotencyKey string) (auditID string, err error)
	AuditQuery(ctx context.Context, from, to time.Time, filters map[string]string, limit int32) ([]k1s0client.AuditEventSummary, error)
	// Log.
	LogSend(ctx context.Context, severity k1s0client.LogSeverity, body string, attributes map[string]string) error
	// Telemetry.
	TelemetryEmitMetric(ctx context.Context, points []k1s0client.MetricPoint) error
	// PII.
	PiiClassify(ctx context.Context, text string) (findings []k1s0client.PiiFindingSummary, containsPii bool, err error)
	PiiMask(ctx context.Context, text string) (maskedText string, findings []k1s0client.PiiFindingSummary, err error)
	// Feature.
	FeatureEvaluateBoolean(ctx context.Context, flagKey string, evalCtx map[string]string) (value bool, variant, reason string, err error)
	// Binding.
	BindingInvoke(ctx context.Context, name, operation string, data []byte, metadata map[string]string) (responseData []byte, responseMetadata map[string]string, err error)
}

// Router は REST ルートを mux に登録する。
type Router struct {
	// tier1 14 サービスへの境界 (テスト容易性のため interface 抽象化)。
	facade Facade
}

// NewRouter は Router を組み立てる。
func NewRouter(facade Facade) *Router {
	return &Router{facade: facade}
}

// Register は mux に REST endpoint を登録する。
// サービスごとに別ファイル (state.go / pubsub.go / ...) に切り出した register* を呼ぶ。
func (r *Router) Register(mux *http.ServeMux) {
	// State (Get / Save / Delete)。
	r.registerState(mux)
	// PubSub (Publish)。
	r.registerPubSub(mux)
	// Secrets (Get / Rotate)。
	r.registerSecrets(mux)
	// Decision (Evaluate)。
	r.registerDecision(mux)
	// Workflow (Start)。
	r.registerWorkflow(mux)
	// Invoke (Call)。
	r.registerInvoke(mux)
	// Audit (Record / Query)。
	r.registerAudit(mux)
	// Log (Send)。
	r.registerLog(mux)
	// Telemetry (EmitMetric)。
	r.registerTelemetry(mux)
	// PII (Classify / Mask)。
	r.registerPii(mux)
	// Feature (EvaluateBoolean)。
	r.registerFeature(mux)
	// Binding (Invoke)。
	r.registerBinding(mux)
}

// errorBody は失敗応答の共通 JSON body。
type errorBody struct {
	// E-T3-BFF-* / E-T1-* 相当のコード。
	Code string `json:"code"`
	// 人間可読メッセージ（PII を含めない）。
	Message string `json:"message"`
}

// writeJSON は status + JSON body を書き出す共通 helper。
func writeJSON(w http.ResponseWriter, status int, body any) {
	// JSON ヘッダを設定する。
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	// status を書く。
	w.WriteHeader(status)
	// body を JSON エンコードする（エラーは応答済のため無視）。
	_ = json.NewEncoder(w).Encode(body)
}

// writeBadRequest は 400 + エラー JSON を返す。
func writeBadRequest(w http.ResponseWriter, code, message string) {
	writeJSON(w, http.StatusBadRequest, errorBody{Code: code, Message: message})
}

// writeBadGateway は 502 + エラー JSON を返す（上流 tier1 のエラー転送用）。
func writeBadGateway(w http.ResponseWriter, code, message string) {
	writeJSON(w, http.StatusBadGateway, errorBody{Code: code, Message: message})
}

// decodeJSON は req.Body を JSON デコードする。失敗時は 400 を書いて false を返す。
func decodeJSON(w http.ResponseWriter, req *http.Request, dst any) bool {
	// JSON デコードを試みる。
	if err := json.NewDecoder(req.Body).Decode(dst); err != nil {
		writeBadRequest(w, "E-T3-BFF-INVALID-JSON", "invalid json: "+err.Error())
		return false
	}
	return true
}
