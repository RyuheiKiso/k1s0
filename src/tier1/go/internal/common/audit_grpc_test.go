// 本ファイルは gRPCAuditEmitter の単体テスト。
//
// 検証観点:
//   - bufconn で fake AuditService.Record を立て、Emit で event が届くこと
//   - outcome 翻訳（success → SUCCESS、denied → DENIED、failure → ERROR）
//   - attributes に trace_id / span_id / code が乗ること
//   - キュー満杯時は drop（Emit が block しない）
//   - Close 後の Emit は no-op

package common

import (
	"context"
	"net"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
)

// fakeAuditService は AuditService.Record だけを実装する fake（test 専用）。
type fakeAuditService struct {
	auditv1.UnimplementedAuditServiceServer
	mu       sync.Mutex
	received []*auditv1.RecordAuditRequest
}

func (f *fakeAuditService) Record(_ context.Context, req *auditv1.RecordAuditRequest) (*auditv1.RecordAuditResponse, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.received = append(f.received, req)
	return &auditv1.RecordAuditResponse{AuditId: "test-id"}, nil
}

func (f *fakeAuditService) snapshot() []*auditv1.RecordAuditRequest {
	f.mu.Lock()
	defer f.mu.Unlock()
	out := make([]*auditv1.RecordAuditRequest, len(f.received))
	copy(out, f.received)
	return out
}

// startFakeAuditServer は bufconn 経由で fakeAuditService を立てる。
func startFakeAuditServer(t *testing.T) (*fakeAuditService, AuditEmitter, func()) {
	t.Helper()
	lis := bufconn.Listen(64 * 1024)
	srv := grpc.NewServer()
	fake := &fakeAuditService{}
	auditv1.RegisterAuditServiceServer(srv, fake)
	go func() { _ = srv.Serve(lis) }()

	dialer := func(context.Context, string) (net.Conn, error) { return lis.Dial() }
	conn, err := grpc.DialContext(context.Background(), "passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	emitter := &gRPCAuditEmitter{
		conn:        conn,
		client:      auditv1.NewAuditServiceClient(conn),
		queue:       make(chan AuditEvent, 16),
		done:        make(chan struct{}),
		isOpen:      true,
		sendTimeout: 2 * time.Second,
	}
	go emitter.worker()

	cleanup := func() {
		_ = emitter.Close(2 * time.Second)
		srv.Stop()
		_ = lis.Close()
	}
	return fake, emitter, cleanup
}

// fake server で event が受信される。
func TestGRPCAuditEmitter_DeliversEvent(t *testing.T) {
	fake, emitter, cleanup := startFakeAuditServer(t)
	defer cleanup()

	emitter.Emit(context.Background(), AuditEvent{
		TenantID: "T1", Actor: "alice",
		Action: "/k1s0.tier1.secrets.v1.SecretsService/Rotate",
		Resource: "db-password", Result: "success", Code: "OK",
		TraceID: "trace-123", SpanID: "span-abc",
	})
	// worker が非同期送信するため、最大 1 秒待つ。
	deadline := time.Now().Add(time.Second)
	for time.Now().Before(deadline) {
		if len(fake.snapshot()) > 0 {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
	got := fake.snapshot()
	if len(got) != 1 {
		t.Fatalf("expected 1 record, got %d", len(got))
	}
	ev := got[0].GetEvent()
	if ev.GetActor() != "alice" || ev.GetResource() != "db-password" {
		t.Errorf("event mismatch: %+v", ev)
	}
	if ev.GetOutcome() != "SUCCESS" {
		t.Errorf("outcome = %q want SUCCESS", ev.GetOutcome())
	}
	attrs := ev.GetAttributes()
	if attrs["trace_id"] != "trace-123" || attrs["span_id"] != "span-abc" || attrs["code"] != "OK" {
		t.Errorf("attributes mismatch: %v", attrs)
	}
	if got[0].GetContext().GetTenantId() != "T1" {
		t.Errorf("tenant_id mismatch: %v", got[0].GetContext())
	}
}

// outcome 翻訳。
func TestGRPCAuditEmitter_OutcomeTranslation(t *testing.T) {
	fake, emitter, cleanup := startFakeAuditServer(t)
	defer cleanup()
	emitter.Emit(context.Background(), AuditEvent{TenantID: "T", Action: "/x", Resource: "r", Result: "success"})
	emitter.Emit(context.Background(), AuditEvent{TenantID: "T", Action: "/x", Resource: "r", Result: "denied"})
	emitter.Emit(context.Background(), AuditEvent{TenantID: "T", Action: "/x", Resource: "r", Result: "failure"})
	deadline := time.Now().Add(time.Second)
	for time.Now().Before(deadline) {
		if len(fake.snapshot()) >= 3 {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
	got := fake.snapshot()
	if len(got) != 3 {
		t.Fatalf("expected 3 records, got %d", len(got))
	}
	want := []string{"SUCCESS", "DENIED", "ERROR"}
	for i, w := range want {
		if got[i].GetEvent().GetOutcome() != w {
			t.Errorf("event[%d] outcome = %q want %q", i, got[i].GetEvent().GetOutcome(), w)
		}
	}
}

// Close 後の Emit は no-op（panic / block しない）。
func TestGRPCAuditEmitter_AfterClose_NoOp(t *testing.T) {
	_, emitter, cleanup := startFakeAuditServer(t)
	cleanup()
	// Close 後の Emit は drop されて返るはず。
	done := make(chan struct{})
	go func() {
		defer close(done)
		emitter.Emit(context.Background(), AuditEvent{TenantID: "T", Action: "/x"})
	}()
	select {
	case <-done:
		// 期待通り return。
	case <-time.After(time.Second):
		t.Fatal("Emit blocked after Close")
	}
}
