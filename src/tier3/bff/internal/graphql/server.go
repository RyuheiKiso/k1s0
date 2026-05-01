// GraphQL endpoint（リリース時点 minimal、リリース時点 で gqlgen に置換）。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// 設計:
//   tier1 14 公開サービスを GraphQL Query / Mutation として薄く露出する。リリース時点 では
//   substring match で operation を識別する minimal 実装で、operationName が指定されていれば
//   それを優先する。リリース時点 で gqlgen による型付き resolver に置換する想定。
//
// 露出 Query:
//   stateGet / secretsGet / decisionEvaluate / featureEvaluateBoolean /
//   piiClassify / piiMask / auditQuery / currentUser
// 露出 Mutation:
//   stateSave / stateDelete / pubsubPublish / secretsRotate / workflowStart /
//   invokeCall / auditRecord / logSend / telemetryEmitMetric / bindingInvoke

// Package graphql は portal-bff の GraphQL endpoint を提供する。
package graphql

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード / デコード。
	"encoding/json"
	// HTTP server。
	"net/http"
	// 文字列処理。
	"strings"
	// 時刻パース。
	"time"

	// k1s0client の AuditEventSummary / PiiFindingSummary / MetricPoint / LogSeverity を参照する。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/k1s0client"
)

// Facade は Resolver が必要とする tier1 14 サービスの最小 interface。
// 実体は k1s0client.Client が満たすが、test では in-memory mock を渡せる。
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

// Resolver は GraphQL クエリを解決する Resolver。
type Resolver struct {
	// tier1 14 サービスへの境界（テスト容易性のため interface 抽象化）。
	facade Facade
}

// NewResolver は Resolver を組み立てる。
func NewResolver(facade Facade) *Resolver {
	return &Resolver{facade: facade}
}

// graphqlRequest は POST /graphql の入力 JSON。
type graphqlRequest struct {
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables,omitempty"`
	OperationName string         `json:"operationName,omitempty"`
}

// graphqlResponse は GraphQL 標準応答 JSON。
type graphqlResponse struct {
	Data   any              `json:"data,omitempty"`
	Errors []map[string]any `json:"errors,omitempty"`
}

// resolverFunc は単一 operation を解決する関数の共通シグネチャ。
type resolverFunc func(ctx context.Context, vars map[string]any) (any, error)

// Handler は POST /graphql ハンドラを返す。
func (r *Resolver) Handler() http.HandlerFunc {
	// 各 operation 名と resolver 関数の対応表（operationName / query 文字列 substring の両方で参照）。
	operations := r.operations()
	return func(w http.ResponseWriter, req *http.Request) {
		// POST のみ受ける。
		if req.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		// JSON body をデコードする。
		var gReq graphqlRequest
		if err := json.NewDecoder(req.Body).Decode(&gReq); err != nil {
			http.Error(w, "invalid json: "+err.Error(), http.StatusBadRequest)
			return
		}
		// timeout を被せる。
		ctx, cancel := context.WithTimeout(req.Context(), 5*time.Second)
		defer cancel()
		// operation を解決する。
		opName := pickOperation(gReq.OperationName, gReq.Query, operations)
		// 応答を組み立てる。
		var resp graphqlResponse
		if opName == "" {
			resp.Errors = []map[string]any{{"message": "unsupported query"}}
		} else if fn, ok := operations[opName]; ok {
			data, err := fn(ctx, gReq.Variables)
			if err != nil {
				resp.Errors = []map[string]any{{"message": err.Error()}}
			} else {
				resp.Data = map[string]any{opName: data}
			}
		} else {
			// pickOperation が誤って未登録 op を返すことは無いが念のため。
			resp.Errors = []map[string]any{{"message": "unsupported query"}}
		}
		// JSON で応答する。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		_ = json.NewEncoder(w).Encode(resp)
	}
}

// operations は本 Resolver が解決する operation 名と関数の対応を返す。
// 順序は意味を持たないが、long-name first の優先順位は pickOperation 側で扱う。
func (r *Resolver) operations() map[string]resolverFunc {
	return map[string]resolverFunc{
		// Read 系（Query）。
		"stateGet":               r.resolveStateGet,
		"secretsGet":             r.resolveSecretsGet,
		"decisionEvaluate":       r.resolveDecisionEvaluate,
		"featureEvaluateBoolean": r.resolveFeatureEvaluateBoolean,
		"piiClassify":            r.resolvePiiClassify,
		"piiMask":                r.resolvePiiMask,
		"auditQuery":             r.resolveAuditQuery,
		"currentUser":            r.resolveCurrentUser,
		// Write 系（Mutation）。
		"stateSave":           r.resolveStateSave,
		"stateDelete":         r.resolveStateDelete,
		"pubsubPublish":       r.resolvePubSubPublish,
		"secretsRotate":       r.resolveSecretsRotate,
		"workflowStart":       r.resolveWorkflowStart,
		"invokeCall":          r.resolveInvokeCall,
		"auditRecord":         r.resolveAuditRecord,
		"logSend":             r.resolveLogSend,
		"telemetryEmitMetric": r.resolveTelemetryEmitMetric,
		"bindingInvoke":       r.resolveBindingInvoke,
	}
}

// pickOperation は operationName と query 文字列から実行すべき operation 名を決定する。
// operationName が op 表に存在すればそれを採用、無ければ query 中の最長一致 op 名を採用する。
func pickOperation(opName, query string, operations map[string]resolverFunc) string {
	// operationName が表にあればそれを最優先。
	if opName != "" {
		if _, ok := operations[opName]; ok {
			return opName
		}
	}
	// query 文字列に含まれる op 名のうち、最長一致を採る（"stateGet" が "stateGetExtended" を侵食しない）。
	best := ""
	for name := range operations {
		if strings.Contains(query, name) && len(name) > len(best) {
			best = name
		}
	}
	return best
}

// strVar は variables から string を取り出す（未指定 / 型不一致は ""）。
func strVar(vars map[string]any, key string) string {
	v, _ := vars[key].(string)
	return v
}

// boolVar は variables から bool を取り出す（未指定 / 型不一致は false）。
func boolVar(vars map[string]any, key string) bool {
	v, _ := vars[key].(bool)
	return v
}

// floatVar は variables から float64 を取り出す（未指定 / 型不一致は 0）。
func floatVar(vars map[string]any, key string) float64 {
	v, _ := vars[key].(float64)
	return v
}

// int32Var は variables から int32 を取り出す（JSON 数値は float64 で来るためキャスト）。
func int32Var(vars map[string]any, key string) int32 {
	v, _ := vars[key].(float64)
	return int32(v)
}

// stringMapVar は variables から map[string]string を取り出す（型不一致は nil）。
func stringMapVar(vars map[string]any, key string) map[string]string {
	raw, ok := vars[key].(map[string]any)
	if !ok {
		return nil
	}
	out := make(map[string]string, len(raw))
	for k, v := range raw {
		if s, ok := v.(string); ok {
			out[k] = s
		}
	}
	return out
}

// resolveStateGet は Query stateGet(store, key) を解決する。
func (r *Resolver) resolveStateGet(ctx context.Context, vars map[string]any) (any, error) {
	store := strVar(vars, "store")
	key := strVar(vars, "key")
	data, etag, found, err := r.facade.StateGet(ctx, store, key)
	if err != nil {
		return nil, err
	}
	if !found {
		return nil, nil
	}
	return map[string]any{"data": string(data), "etag": etag}, nil
}

// resolveStateSave は Mutation stateSave(store, key, data) を解決する。
func (r *Resolver) resolveStateSave(ctx context.Context, vars map[string]any) (any, error) {
	etag, err := r.facade.StateSave(ctx, strVar(vars, "store"), strVar(vars, "key"), []byte(strVar(vars, "data")))
	if err != nil {
		return nil, err
	}
	return map[string]any{"etag": etag}, nil
}

// resolveStateDelete は Mutation stateDelete(store, key, expectedEtag?) を解決する。
func (r *Resolver) resolveStateDelete(ctx context.Context, vars map[string]any) (any, error) {
	if err := r.facade.StateDelete(ctx, strVar(vars, "store"), strVar(vars, "key"), strVar(vars, "expectedEtag")); err != nil {
		return nil, err
	}
	return map[string]any{"ok": true}, nil
}

// resolvePubSubPublish は Mutation pubsubPublish(topic, data, contentType?, idempotencyKey?, metadata?) を解決する。
func (r *Resolver) resolvePubSubPublish(ctx context.Context, vars map[string]any) (any, error) {
	contentType := strVar(vars, "contentType")
	if contentType == "" {
		contentType = "application/json"
	}
	offset, err := r.facade.PubSubPublish(ctx, strVar(vars, "topic"), []byte(strVar(vars, "data")), contentType, strVar(vars, "idempotencyKey"), stringMapVar(vars, "metadata"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"offset": offset}, nil
}

// resolveSecretsGet は Query secretsGet(name) を解決する。
func (r *Resolver) resolveSecretsGet(ctx context.Context, vars map[string]any) (any, error) {
	values, version, err := r.facade.SecretsGet(ctx, strVar(vars, "name"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"values": values, "version": version}, nil
}

// resolveSecretsRotate は Mutation secretsRotate(name, gracePeriodSec?, idempotencyKey?) を解決する。
func (r *Resolver) resolveSecretsRotate(ctx context.Context, vars map[string]any) (any, error) {
	newV, prevV, err := r.facade.SecretsRotate(ctx, strVar(vars, "name"), int32Var(vars, "gracePeriodSec"), strVar(vars, "idempotencyKey"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"newVersion": newV, "previousVersion": prevV}, nil
}

// resolveDecisionEvaluate は Query decisionEvaluate(ruleId, ruleVersion?, inputJson, includeTrace?) を解決する。
func (r *Resolver) resolveDecisionEvaluate(ctx context.Context, vars map[string]any) (any, error) {
	out, trace, elapsed, err := r.facade.DecisionEvaluate(ctx, strVar(vars, "ruleId"), strVar(vars, "ruleVersion"), []byte(strVar(vars, "inputJson")), boolVar(vars, "includeTrace"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"outputJson": string(out), "traceJson": string(trace), "elapsedUs": elapsed}, nil
}

// resolveWorkflowStart は Mutation workflowStart(workflowType, workflowId, input?, idempotent?) を解決する。
func (r *Resolver) resolveWorkflowStart(ctx context.Context, vars map[string]any) (any, error) {
	wfID, runID, err := r.facade.WorkflowStart(ctx, strVar(vars, "workflowType"), strVar(vars, "workflowId"), []byte(strVar(vars, "input")), boolVar(vars, "idempotent"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"workflowId": wfID, "runId": runID}, nil
}

// resolveInvokeCall は Mutation invokeCall(appId, method, data?, contentType?, timeoutMs?) を解決する。
func (r *Resolver) resolveInvokeCall(ctx context.Context, vars map[string]any) (any, error) {
	contentType := strVar(vars, "contentType")
	if contentType == "" {
		contentType = "application/json"
	}
	respData, respCT, status, err := r.facade.InvokeCall(ctx, strVar(vars, "appId"), strVar(vars, "method"), []byte(strVar(vars, "data")), contentType, int32Var(vars, "timeoutMs"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"data": string(respData), "contentType": respCT, "status": status}, nil
}

// resolveAuditRecord は Mutation auditRecord(actor, action, resource, outcome, attributes?, idempotencyKey?) を解決する。
func (r *Resolver) resolveAuditRecord(ctx context.Context, vars map[string]any) (any, error) {
	auditID, err := r.facade.AuditRecord(ctx, strVar(vars, "actor"), strVar(vars, "action"), strVar(vars, "resource"), strVar(vars, "outcome"), stringMapVar(vars, "attributes"), strVar(vars, "idempotencyKey"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"auditId": auditID}, nil
}

// resolveAuditQuery は Query auditQuery(from?, to?, filters?, limit?) を解決する。
// from / to は RFC3339 文字列（空なら zero time）。
func (r *Resolver) resolveAuditQuery(ctx context.Context, vars map[string]any) (any, error) {
	var from, to time.Time
	if s := strVar(vars, "from"); s != "" {
		t, err := time.Parse(time.RFC3339, s)
		if err != nil {
			return nil, err
		}
		from = t
	}
	if s := strVar(vars, "to"); s != "" {
		t, err := time.Parse(time.RFC3339, s)
		if err != nil {
			return nil, err
		}
		to = t
	}
	events, err := r.facade.AuditQuery(ctx, from, to, stringMapVar(vars, "filters"), int32Var(vars, "limit"))
	if err != nil {
		return nil, err
	}
	out := make([]any, 0, len(events))
	for _, e := range events {
		out = append(out, map[string]any{
			"occurredAtMillis": e.OccurredAtMillis,
			"actor":            e.Actor,
			"action":           e.Action,
			"resource":         e.Resource,
			"outcome":          e.Outcome,
			"attributes":       e.Attributes,
		})
	}
	return out, nil
}

// resolveLogSend は Mutation logSend(severity?, body, attributes?) を解決する。
func (r *Resolver) resolveLogSend(ctx context.Context, vars map[string]any) (any, error) {
	severity := k1s0client.LogSeverity(strVar(vars, "severity"))
	if severity == "" {
		severity = k1s0client.LogSeverityInfo
	}
	if err := r.facade.LogSend(ctx, severity, strVar(vars, "body"), stringMapVar(vars, "attributes")); err != nil {
		return nil, err
	}
	return map[string]any{"ok": true}, nil
}

// resolveTelemetryEmitMetric は Mutation telemetryEmitMetric(name, value, labels?) を解決する。
// minimal 実装では single-point 受付のみ（複数 point は REST の方を使う）。
func (r *Resolver) resolveTelemetryEmitMetric(ctx context.Context, vars map[string]any) (any, error) {
	point := k1s0client.MetricPoint{
		Name:   strVar(vars, "name"),
		Value:  floatVar(vars, "value"),
		Labels: stringMapVar(vars, "labels"),
	}
	if err := r.facade.TelemetryEmitMetric(ctx, []k1s0client.MetricPoint{point}); err != nil {
		return nil, err
	}
	return map[string]any{"ok": true}, nil
}

// resolvePiiClassify は Query piiClassify(text) を解決する。
func (r *Resolver) resolvePiiClassify(ctx context.Context, vars map[string]any) (any, error) {
	findings, contains, err := r.facade.PiiClassify(ctx, strVar(vars, "text"))
	if err != nil {
		return nil, err
	}
	return map[string]any{
		"findings":    findingsToJSON(findings),
		"containsPii": contains,
	}, nil
}

// resolvePiiMask は Query piiMask(text) を解決する。
func (r *Resolver) resolvePiiMask(ctx context.Context, vars map[string]any) (any, error) {
	masked, findings, err := r.facade.PiiMask(ctx, strVar(vars, "text"))
	if err != nil {
		return nil, err
	}
	return map[string]any{
		"maskedText": masked,
		"findings":   findingsToJSON(findings),
	}, nil
}

// findingsToJSON は PiiFindingSummary を GraphQL 応答用 map に詰め替える。
func findingsToJSON(in []k1s0client.PiiFindingSummary) []any {
	out := make([]any, 0, len(in))
	for _, f := range in {
		out = append(out, map[string]any{
			"type":       f.Type,
			"start":      f.Start,
			"end":        f.End,
			"confidence": f.Confidence,
		})
	}
	return out
}

// resolveFeatureEvaluateBoolean は Query featureEvaluateBoolean(flagKey, evalCtx?) を解決する。
func (r *Resolver) resolveFeatureEvaluateBoolean(ctx context.Context, vars map[string]any) (any, error) {
	value, variant, reason, err := r.facade.FeatureEvaluateBoolean(ctx, strVar(vars, "flagKey"), stringMapVar(vars, "evalCtx"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"value": value, "variant": variant, "reason": reason}, nil
}

// resolveBindingInvoke は Mutation bindingInvoke(name, operation, data?, metadata?) を解決する。
func (r *Resolver) resolveBindingInvoke(ctx context.Context, vars map[string]any) (any, error) {
	respData, respMeta, err := r.facade.BindingInvoke(ctx, strVar(vars, "name"), strVar(vars, "operation"), []byte(strVar(vars, "data")), stringMapVar(vars, "metadata"))
	if err != nil {
		return nil, err
	}
	return map[string]any{"data": string(respData), "metadata": respMeta}, nil
}

// resolveCurrentUser は Query currentUser を解決する（auth middleware が attach する subject 前提）。
func (r *Resolver) resolveCurrentUser(_ context.Context, _ map[string]any) (any, error) {
	// 実装は auth middleware 完成後に subject / roles を context から取り出す。
	return map[string]any{"id": "anonymous", "roles": []string{}}, nil
}
