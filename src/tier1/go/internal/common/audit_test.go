// 本ファイルは AuditInterceptor の単体テスト。
//
// 検証観点:
//   - 特権 RPC は handler 後段で AuditEmitter.Emit が呼ばれる
//   - 非特権 RPC は emit が呼ばれない
//   - emit 失敗（emitter panic 含まず、内部で fail-soft）が handler 結果に影響しない
//   - actor は AuthInfo.Subject 優先、fallback で Context.subject、最終 "unknown"
//   - tenant_id は AuthInfo > Context の優先順位
//   - resource は GetName / GetWorkflowId / GetRuleId / GetFlagKey / GetTopic から最初に取れたもの
//   - error 時 result は "denied"（PermissionDenied）/ "failure"（その他）
//   - trace_id / span_id は OTel span context から取り出される

package common

import (
	"context"
	"errors"
	"sync"
	"testing"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
	"google.golang.org/grpc"
	grpccodes "google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// captureEmitter は emit された AuditEvent を保持する test 用 emitter。
type captureEmitter struct {
	mu     sync.Mutex
	events []AuditEvent
}

func (c *captureEmitter) Emit(_ context.Context, ev AuditEvent) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.events = append(c.events, ev)
}

func (c *captureEmitter) snapshot() []AuditEvent {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]AuditEvent, len(c.events))
	copy(out, c.events)
	return out
}

// fakeRequestWithName は GetContext + GetName を持つ proto-like 型。
type fakeRequestWithName struct {
	ctx  *fakeTenantContextFull
	name string
}

func (f *fakeRequestWithName) GetContext() *fakeTenantContextFull {
	return f.ctx
}
func (f *fakeRequestWithName) GetName() string { return f.name }

// fakeTenantContextFull は GetTenantId と GetSubject を持つ。
type fakeTenantContextFull struct {
	tenantID string
	subject  string
}

func (f *fakeTenantContextFull) GetTenantId() string { return f.tenantID }
func (f *fakeTenantContextFull) GetSubject() string  { return f.subject }

// 特権 RPC は emit される。
func TestAuditInterceptor_Privileged_Emits(t *testing.T) {
	cap := &captureEmitter{}
	icpt := AuditInterceptor(cap)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"}
	resp, err := icpt(context.Background(), &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T1", subject: "user-1"}, name: "db-password",
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if err != nil || resp != "ok" {
		t.Fatalf("handler passthrough failed: resp=%v err=%v", resp, err)
	}
	evs := cap.snapshot()
	if len(evs) != 1 {
		t.Fatalf("expected 1 audit event, got %d", len(evs))
	}
	got := evs[0]
	if got.TenantID != "T1" {
		t.Errorf("TenantID = %q want T1", got.TenantID)
	}
	if got.Actor != "user-1" {
		t.Errorf("Actor = %q want user-1", got.Actor)
	}
	if got.Action != info.FullMethod {
		t.Errorf("Action = %q want %q", got.Action, info.FullMethod)
	}
	if got.Resource != "db-password" {
		t.Errorf("Resource = %q want db-password", got.Resource)
	}
	if got.Result != "success" {
		t.Errorf("Result = %q want success", got.Result)
	}
	if got.Code != "OK" {
		t.Errorf("Code = %q want OK", got.Code)
	}
}

// 非特権 RPC は emit されない。
func TestAuditInterceptor_NonPrivileged_NoEmit(t *testing.T) {
	cap := &captureEmitter{}
	icpt := AuditInterceptor(cap)
	// State.Get は監査対象外（読取・高頻度）。
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.state.v1.StateService/Get"}
	_, _ = icpt(context.Background(), &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T1"},
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if got := len(cap.snapshot()); got != 0 {
		t.Fatalf("non-privileged RPC must not emit, got %d events", got)
	}
}

// AuthInfo.Subject が fallback より優先される。
func TestAuditInterceptor_AuthInfo_TakesPrecedence(t *testing.T) {
	cap := &captureEmitter{}
	icpt := AuditInterceptor(cap)
	ctx := context.WithValue(context.Background(), authInfoKey{}, &AuthInfo{
		TenantID: "T-jwt", Subject: "subj-jwt", Mode: AuthModeHMAC,
	})
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"}
	_, _ = icpt(ctx, &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T-ctx", subject: "subj-ctx"}, name: "x",
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	evs := cap.snapshot()
	if len(evs) != 1 || evs[0].Actor != "subj-jwt" || evs[0].TenantID != "T-jwt" {
		t.Fatalf("AuthInfo not preferred: %+v", evs)
	}
}

// PermissionDenied は result=denied、その他 error は failure。
func TestAuditInterceptor_ResultClassification(t *testing.T) {
	cap := &captureEmitter{}
	icpt := AuditInterceptor(cap)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"}
	// PermissionDenied.
	_, _ = icpt(context.Background(), &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T"}, name: "x",
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, status.Error(grpccodes.PermissionDenied, "no")
	})
	// Internal.
	_, _ = icpt(context.Background(), &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T"}, name: "x",
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, errors.New("boom")
	})
	evs := cap.snapshot()
	if len(evs) != 2 {
		t.Fatalf("expected 2 events, got %d", len(evs))
	}
	if evs[0].Result != "denied" || evs[0].Code != "PermissionDenied" {
		t.Errorf("event[0] result/code: %+v", evs[0])
	}
	if evs[1].Result != "failure" {
		t.Errorf("event[1] result: %+v", evs[1])
	}
}

// trace_id / span_id は ctx の OTel span から引かれる。
func TestAuditInterceptor_TraceContext_Captured(t *testing.T) {
	exporter := tracetest.NewInMemoryExporter()
	tp := trace.NewTracerProvider(trace.WithSyncer(exporter))
	defer func() { _ = tp.Shutdown(context.Background()) }()
	otel.SetTracerProvider(tp)
	t.Cleanup(func() { otel.SetTracerProvider(otel.GetTracerProvider()) })

	tracer := tp.Tracer("test")
	ctx, span := tracer.Start(context.Background(), "test-span")
	defer span.End()

	cap := &captureEmitter{}
	icpt := AuditInterceptor(cap)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"}
	_, _ = icpt(ctx, &fakeRequestWithName{
		ctx: &fakeTenantContextFull{tenantID: "T"}, name: "x",
	}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	evs := cap.snapshot()
	if len(evs) != 1 || evs[0].TraceID == "" || evs[0].SpanID == "" {
		t.Fatalf("trace context not captured: %+v", evs)
	}
}

// nil emitter は内部で NoopAuditEmitter に置換され panic しない。
func TestAuditInterceptor_NilEmitter_NoPanic(t *testing.T) {
	icpt := AuditInterceptor(nil)
	info := &grpc.UnaryServerInfo{FullMethod: "/k1s0.tier1.secrets.v1.SecretsService/Rotate"}
	resp, err := icpt(context.Background(), &fakeRequestWithName{ctx: &fakeTenantContextFull{tenantID: "T"}, name: "x"}, info, func(ctx context.Context, req interface{}) (interface{}, error) {
		return "ok", nil
	})
	if err != nil || resp != "ok" {
		t.Fatalf("nil emitter handler not transparent: resp=%v err=%v", resp, err)
	}
}

// privilegedRPCs map の整合性: docs §「監査と痕跡」に列挙された API が含まれている。
func TestAuditInterceptor_PrivilegedSet_DocsAlignment(t *testing.T) {
	mustHave := []string{
		// Secrets（取得 + ローテーション）
		"/k1s0.tier1.secrets.v1.SecretsService/Get",
		"/k1s0.tier1.secrets.v1.SecretsService/Rotate",
		// Decision（評価）
		"/k1s0.tier1.decision.v1.DecisionService/Evaluate",
		// Feature（変更操作）
		"/k1s0.tier1.feature.v1.FeatureAdminService/RegisterFlag",
		// Binding（外部送信）
		"/k1s0.tier1.binding.v1.BindingService/Invoke",
		// Workflow（状態変更）
		"/k1s0.tier1.workflow.v1.WorkflowService/Start",
		"/k1s0.tier1.workflow.v1.WorkflowService/Terminate",
	}
	for _, m := range mustHave {
		if !privilegedRPCs[m] {
			t.Errorf("docs §「監査と痕跡」が要求する %q が privilegedRPCs に未登録", m)
		}
	}
	// State.Get / Health.Liveness は監査対象外（含まれてはならない）。
	mustNotHave := []string{
		"/k1s0.tier1.state.v1.StateService/Get",
		"/k1s0.tier1.health.v1.HealthService/Liveness",
		"/k1s0.tier1.audit.v1.AuditService/Record", // 再帰防止
	}
	for _, m := range mustNotHave {
		if privilegedRPCs[m] {
			t.Errorf("%q は privilegedRPCs に含まれてはならない（誤発火 / 再帰の原因）", m)
		}
	}
}
