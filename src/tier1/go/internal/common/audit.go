// 本ファイルは tier1 facade の Audit 自動発行 interceptor。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「監査と痕跡」:
//       特権操作（Secret 取得・ローテーション、Decision 評価、Feature Flag 変更、
//       Binding 外部送信、Audit 書込、Workflow 状態変更）は自動的に Audit API に
//       イベントを発行する。呼出側が明示的に Audit を呼ぶ必要はない。
//     必須フィールド:
//       - trace_id / span_id（トレース連携）
//       - tenant_id / actor（JWT sub または SPIFFE ID）
//       - action / resource / result（成功/失敗/拒否）
//       - previous_hash / event_hash（改ざん検知のハッシュチェーン）は audit Pod 側で付与
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-MON-001
//
// 実装方針:
//   - AuditEmitter 抽象を経由して特権 RPC 完了後に event を fire-and-forget で送る
//   - production の AuditEmitter は t1-audit Pod の AuditService.Record を gRPC で叩く
//   - test / 早期 dev は no-op emitter（NoopAuditEmitter）で interceptor を無効化
//   - 特権 RPC リストは fullMethod 完全一致の set で管理（誤発火防止）
//   - emit 失敗は handler 結果に影響させない（warn ログのみ、SLO に紐付かないため fail-soft）
//
// アクター解決順序:
//   1. AuthInterceptor が attach した AuthInfo.Subject（JWT sub）
//   2. fallback: TenantContext.subject（proto field）
//   3. それも空なら "unknown"

package common

import (
	// 全 RPC で context を伝搬する。
	"context"
	// fail-soft 時の warn ログ。
	"log"
	// proto request の subject / resource を取り出すための reflection。
	"reflect"

	// OTel trace span から trace_id / span_id を取り出す。
	"go.opentelemetry.io/otel/trace"

	// gRPC server / status。
	"google.golang.org/grpc"
	"google.golang.org/grpc/status"
)

// AuditEvent は AuditEmitter に渡す中間表現。Audit Pod 側で hash chain を付与する。
type AuditEvent struct {
	// テナント識別子（必須）。
	TenantID string
	// 行為主体（JWT sub / SPIFFE ID）。
	Actor string
	// 操作（gRPC fullMethod、例: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"）。
	Action string
	// 対象リソース（secret name / workflow_id / rule_id 等、API 別に解決）。
	Resource string
	// "success" / "failure" / "denied"。
	Result string
	// W3C Trace Context（OTel span から取り出して連携）。
	TraceID string
	SpanID  string
	// gRPC status code（"OK" / "PermissionDenied" / ...）。
	Code string
}

// AuditEmitter は AuditEvent を Audit Pod / store に送る抽象。
// production: t1-audit Pod への gRPC client。
// test / dev: NoopAuditEmitter で無効化。
type AuditEmitter interface {
	// Emit は event を非同期で送る。失敗は呼出側に返さない（warn ログのみ）。
	Emit(ctx context.Context, ev AuditEvent)
}

// NoopAuditEmitter は何もしない emitter（test / 早期 dev 用）。
type NoopAuditEmitter struct{}

// Emit は no-op。
func (NoopAuditEmitter) Emit(_ context.Context, _ AuditEvent) {}

// privilegedRPCs は docs §「監査と痕跡」で auto-emit が MUST と規定された fullMethod 集合。
// 該当しない RPC は audit を発行しない（誤発火防止 / 監査ストア肥大化防止）。
// privilegedRPCs は docs NFR-E-MON-001 / 002 / 004 を満たす対象 fullMethod 集合。
// Rust 側 `privileged_rpcs()`（src/tier1/rust/crates/common/src/audit.rs）と完全一致を保つ。
//
// 対象選定基準:
//   - Secret アクセス（Get/BulkGet/GetDynamic/Rotate）: NFR-E-MON-002
//   - Decision 評価 + Rule 登録: NFR-E-MON-001「Decision 評価」+ NFR-E-MON-004
//   - Feature Flag 定義変更（RegisterFlag のみ。Evaluate は高頻度のため除外）: NFR-E-MON-004
//   - Binding 外部送信: NFR-E-MON-001
//   - Workflow 状態変更（Start/Signal/Cancel/Terminate）: NFR-E-MON-001
//   - State 書込（Set/Delete/Transact）: NFR-E-MON-001「tier1 API 呼び出し」
//   - PubSub 発行（Publish/BulkPublish）: NFR-E-MON-001 同上
var privilegedRPCs = map[string]bool{
	// State 書込（NFR-E-MON-001）。Get / BulkGet は高頻度のため除外。
	"/k1s0.tier1.state.v1.StateService/Set":      true,
	"/k1s0.tier1.state.v1.StateService/Delete":   true,
	"/k1s0.tier1.state.v1.StateService/Transact": true,
	// PubSub 発行（NFR-E-MON-001）。Subscribe は受信側のため除外。
	"/k1s0.tier1.pubsub.v1.PubSubService/Publish":     true,
	"/k1s0.tier1.pubsub.v1.PubSubService/BulkPublish": true,
	// Secrets 全アクセス（NFR-E-MON-002）。
	"/k1s0.tier1.secrets.v1.SecretsService/Get":        true,
	"/k1s0.tier1.secrets.v1.SecretsService/BulkGet":    true,
	"/k1s0.tier1.secrets.v1.SecretsService/GetDynamic": true,
	"/k1s0.tier1.secrets.v1.SecretsService/Rotate":     true,
	// Decision 評価 + Admin 系（評価 + 定義変更）。
	"/k1s0.tier1.decision.v1.DecisionService/Evaluate":          true,
	"/k1s0.tier1.decision.v1.DecisionService/BatchEvaluate":     true,
	"/k1s0.tier1.decision.v1.DecisionAdminService/RegisterRule": true,
	// Feature 定義変更（NFR-E-MON-004）。Evaluate* は高頻度のため除外。
	"/k1s0.tier1.feature.v1.FeatureAdminService/RegisterFlag": true,
	// Workflow 状態変更系。Query / GetStatus は読取のため除外。
	"/k1s0.tier1.workflow.v1.WorkflowService/Start":     true,
	"/k1s0.tier1.workflow.v1.WorkflowService/Signal":    true,
	"/k1s0.tier1.workflow.v1.WorkflowService/Cancel":    true,
	"/k1s0.tier1.workflow.v1.WorkflowService/Terminate": true,
	// Binding 外部送信。
	"/k1s0.tier1.binding.v1.BindingService/Invoke": true,
	// Audit: Record（再帰防止のため除外。Audit Pod 側で自己ログ運用とする）。
}

// AuditInterceptor は特権 RPC 完了後に AuditEmitter で event を発行する Unary Server Interceptor。
// 非特権 RPC は素通り（性能影響なし）。
// AuthInterceptor の後段に配置するため、AuthInfo.Subject を actor に使える。
func AuditInterceptor(emitter AuditEmitter) grpc.UnaryServerInterceptor {
	if emitter == nil {
		// nil emitter は no-op interceptor として返す。
		emitter = NoopAuditEmitter{}
	}
	return func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
		// 特権 RPC 以外は素通り。
		if !privilegedRPCs[info.FullMethod] {
			return handler(ctx, req)
		}
		// handler 実行。
		resp, err := handler(ctx, req)
		// AuthInfo（前段 AuthInterceptor が attach）から actor を解決する。
		var actor string
		if ai, ok := AuthFromContext(ctx); ok {
			actor = ai.Subject
		}
		// fallback: proto.TenantContext.subject。
		if actor == "" {
			actor = extractSubject(req)
		}
		if actor == "" {
			actor = "unknown"
		}
		// tenant_id を解決する（JWT > TenantContext の優先順位）。
		var tenantID string
		if ai, ok := AuthFromContext(ctx); ok {
			tenantID = ai.TenantID
		}
		if tenantID == "" {
			tenantID = extractTenantID(req)
		}
		// resource を解決する（API ごとに proto field 名が違うため reflection で複数試行）。
		resource := extractResource(req)
		// trace_id / span_id を OTel span から取り出す（ObservabilityInterceptor が start 済）。
		spanCtx := trace.SpanContextFromContext(ctx)
		traceID := ""
		spanID := ""
		if spanCtx.IsValid() {
			traceID = spanCtx.TraceID().String()
			spanID = spanCtx.SpanID().String()
		}
		// result / code を解決する。
		result := "success"
		code := "OK"
		if err != nil {
			st, _ := status.FromError(err)
			code = st.Code().String()
			// PermissionDenied は "denied"、その他 error は "failure"。
			if code == "PermissionDenied" {
				result = "denied"
			} else {
				result = "failure"
			}
		}
		// fire-and-forget で発行する。失敗は warn ログのみで handler 結果に影響させない。
		emitter.Emit(ctx, AuditEvent{
			TenantID: tenantID,
			Actor:    actor,
			Action:   info.FullMethod,
			Resource: resource,
			Result:   result,
			TraceID:  traceID,
			SpanID:   spanID,
			Code:     code,
		})
		return resp, err
	}
}

// extractSubject は proto request の TenantContext.subject を reflection で取り出す。
// extractTenantID と同形のため一括で扱う（GetContext() → GetSubject()）。
func extractSubject(req interface{}) string {
	return reflectStringField(req, "GetContext", "GetSubject")
}

// extractResource は API ごとに異なる "対象リソース" 識別子を取り出す。
// 試行順序: GetName → GetWorkflowId → GetRuleId → GetFlagKey → GetTopic
// 最初にマッチしたものを返し、すべて空なら "" を返す。
func extractResource(req interface{}) string {
	for _, m := range []string{"GetName", "GetWorkflowId", "GetRuleId", "GetFlagKey", "GetTopic"} {
		if v := reflectStringMethod(req, m); v != "" {
			return v
		}
	}
	return ""
}

// reflectStringField は req の chain メソッド呼出（例: GetContext → GetSubject）で文字列を取り出す。
// 途中で nil / 未定義に当たれば "" を返す。
func reflectStringField(req interface{}, methodChain ...string) string {
	if req == nil || len(methodChain) == 0 {
		return ""
	}
	v := reflect.ValueOf(req)
	for i, m := range methodChain {
		if !v.IsValid() {
			return ""
		}
		if v.Kind() == reflect.Ptr && v.IsNil() {
			return ""
		}
		fn := v.MethodByName(m)
		if !fn.IsValid() || fn.Type().NumIn() != 0 || fn.Type().NumOut() != 1 {
			return ""
		}
		out := fn.Call(nil)
		if len(out) != 1 {
			return ""
		}
		v = out[0]
		// 最後のメソッドなら string を取り出して返す。
		if i == len(methodChain)-1 {
			if v.Kind() != reflect.String {
				return ""
			}
			return v.String()
		}
		// 途中段で nil interface / nil pointer はそこで終了。
		switch v.Kind() {
		case reflect.Ptr, reflect.Interface:
			if v.IsNil() {
				return ""
			}
		}
	}
	return ""
}

// reflectStringMethod は req の単一メソッドで文字列を取り出す（chain 1 段版）。
func reflectStringMethod(req interface{}, method string) string {
	return reflectStringField(req, method)
}

// LogAuditEmitter は stderr に JSON で event を吐くだけの簡易 emitter（dev / debug 用）。
// production では gRPC client backed emitter を別途実装して注入する。
type LogAuditEmitter struct{}

// Emit は stderr に JSON 風の単一行ログを書く。fail-soft（書込失敗は無視）。
func (LogAuditEmitter) Emit(_ context.Context, ev AuditEvent) {
	log.Printf(
		"audit event: tenant_id=%q actor=%q action=%q resource=%q result=%q code=%q trace_id=%q span_id=%q",
		ev.TenantID, ev.Actor, ev.Action, ev.Resource, ev.Result, ev.Code, ev.TraceID, ev.SpanID,
	)
}
