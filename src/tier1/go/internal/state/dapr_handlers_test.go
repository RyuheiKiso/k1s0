// 本ファイルは Invoke / Binding / Feature の 3 ハンドラの単体テスト + gRPC 結線テスト。
// それぞれの fake adapter で SDK / 外部 backend を切り離す。

package state

import (
	"context"
	"errors"
	"net"
	"testing"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
)

// ---------------------------------------------------------------------------
// fake adapters
// ---------------------------------------------------------------------------

type fakeInvokeAdapter struct {
	fn func(ctx context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error)
}

func (f *fakeInvokeAdapter) Invoke(ctx context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
	return f.fn(ctx, req)
}

type fakeBindingAdapter struct {
	fn func(ctx context.Context, req dapr.BindingRequest) (dapr.BindingResponse, error)
}

func (f *fakeBindingAdapter) Invoke(ctx context.Context, req dapr.BindingRequest) (dapr.BindingResponse, error) {
	return f.fn(ctx, req)
}

type fakeFeatureAdapter struct {
	boolFn   func(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error)
	stringFn func(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagStringResponse, error)
	numberFn func(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagNumberResponse, error)
	objectFn func(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagObjectResponse, error)
}

func (f *fakeFeatureAdapter) EvaluateBoolean(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error) {
	return f.boolFn(ctx, req)
}
func (f *fakeFeatureAdapter) EvaluateString(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagStringResponse, error) {
	return f.stringFn(ctx, req)
}
func (f *fakeFeatureAdapter) EvaluateNumber(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagNumberResponse, error) {
	return f.numberFn(ctx, req)
}
func (f *fakeFeatureAdapter) EvaluateObject(ctx context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagObjectResponse, error) {
	return f.objectFn(ctx, req)
}

// ---------------------------------------------------------------------------
// Invoke handler tests
// ---------------------------------------------------------------------------

func TestInvokeHandler_OK(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			if req.AppID != "tier2-tax" || req.Method != "calc" {
				t.Fatalf("args mismatch: %+v", req)
			}
			return dapr.InvokeResponse{Data: []byte("ok"), ContentType: "text/plain", Status: 200}, nil
		},
	}
	h := &invokeHandler{deps: Deps{InvokeAdapter: a}}
	resp, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId:       "tier2-tax",
		Method:      "calc",
		Data:        []byte("input"),
		ContentType: "text/plain",
		Context:     makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if string(resp.GetData()) != "ok" {
		t.Fatalf("data mismatch: %s", resp.GetData())
	}
	if resp.GetStatus() != 200 {
		t.Fatalf("status mismatch: %d", resp.GetStatus())
	}
}

func TestInvokeHandler_NilRequest(t *testing.T) {
	h := &invokeHandler{deps: Deps{InvokeAdapter: &fakeInvokeAdapter{}}}
	_, err := h.Invoke(context.Background(), nil)
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("status: got %v want InvalidArgument", got)
	}
}

func TestInvokeHandler_AdapterError(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{}, errors.New("upstream down")
		},
	}
	h := &invokeHandler{deps: Deps{InvokeAdapter: a}}
	_, err := h.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{AppId: "x", Method: "m", Context: makeTenantCtx("T")})
	if got := status.Code(err); got != codes.Internal {
		t.Fatalf("status: got %v want Internal", got)
	}
}

// ---------------------------------------------------------------------------
// Binding handler tests
// ---------------------------------------------------------------------------

func TestBindingHandler_OK(t *testing.T) {
	a := &fakeBindingAdapter{
		fn: func(_ context.Context, req dapr.BindingRequest) (dapr.BindingResponse, error) {
			if req.Name != "s3-archive" || req.Operation != "create" {
				t.Fatalf("args mismatch: %+v", req)
			}
			return dapr.BindingResponse{Data: []byte("ack"), Metadata: map[string]string{"etag": "e1"}}, nil
		},
	}
	h := &bindingHandler{deps: Deps{BindingAdapter: a}}
	resp, err := h.Invoke(context.Background(), &bindingv1.InvokeBindingRequest{
		Name:      "s3-archive",
		Operation: "create",
		Data:      []byte("payload"),
		Context:   makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke error: %v", err)
	}
	if string(resp.GetData()) != "ack" {
		t.Fatalf("data mismatch: %s", resp.GetData())
	}
	if resp.GetMetadata()["etag"] != "e1" {
		t.Fatalf("metadata mismatch: %v", resp.GetMetadata())
	}
}

// ---------------------------------------------------------------------------
// Feature handler tests
// ---------------------------------------------------------------------------

func TestFeatureHandler_EvaluateBoolean_OK(t *testing.T) {
	a := &fakeFeatureAdapter{
		boolFn: func(_ context.Context, req dapr.FlagEvaluateRequest) (dapr.FlagBooleanResponse, error) {
			if req.FlagKey != "checkout.fast" {
				t.Fatalf("flag key mismatch: %s", req.FlagKey)
			}
			return dapr.FlagBooleanResponse{Value: true, Variant: "v1", Reason: "TARGETING_MATCH"}, nil
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateBoolean(context.Background(), &featurev1.EvaluateRequest{FlagKey: "checkout.fast", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("EvaluateBoolean error: %v", err)
	}
	if !resp.GetValue() {
		t.Fatalf("expected Value=true")
	}
	if resp.GetMetadata().GetVariant() != "v1" {
		t.Fatalf("variant mismatch: %s", resp.GetMetadata().GetVariant())
	}
}

func TestFeatureHandler_EvaluateString_OK(t *testing.T) {
	a := &fakeFeatureAdapter{
		stringFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagStringResponse, error) {
			return dapr.FlagStringResponse{Value: "premium", Variant: "v3", Reason: "SPLIT"}, nil
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateString(context.Background(), &featurev1.EvaluateRequest{FlagKey: "tier", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("EvaluateString error: %v", err)
	}
	if resp.GetValue() != "premium" {
		t.Fatalf("value mismatch: %s", resp.GetValue())
	}
}

func TestFeatureHandler_EvaluateNumber_OK(t *testing.T) {
	a := &fakeFeatureAdapter{
		numberFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagNumberResponse, error) {
			return dapr.FlagNumberResponse{Value: 0.05, Variant: "v1", Reason: "TARGETING_MATCH"}, nil
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateNumber(context.Background(), &featurev1.EvaluateRequest{FlagKey: "rate", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("EvaluateNumber error: %v", err)
	}
	if resp.GetValue() != 0.05 {
		t.Fatalf("value mismatch: %v", resp.GetValue())
	}
}

func TestFeatureHandler_EvaluateObject_OK(t *testing.T) {
	a := &fakeFeatureAdapter{
		objectFn: func(_ context.Context, _ dapr.FlagEvaluateRequest) (dapr.FlagObjectResponse, error) {
			return dapr.FlagObjectResponse{ValueJSON: []byte(`{"x":1}`), Variant: "v1", Reason: "TARGETING_MATCH"}, nil
		},
	}
	h := &featureHandler{deps: Deps{FeatureAdapter: a}}
	resp, err := h.EvaluateObject(context.Background(), &featurev1.EvaluateRequest{FlagKey: "rate-limit", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("EvaluateObject error: %v", err)
	}
	if string(resp.GetValueJson()) != `{"x":1}` {
		t.Fatalf("json mismatch: %s", resp.GetValueJson())
	}
}

// ---------------------------------------------------------------------------
// In-process gRPC integration: InvokeStream chunking
// ---------------------------------------------------------------------------

func TestInvokeService_InvokeStream_Chunking(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	// 10 KiB の応答を 4 KiB chunk × 3 で送信する想定。
	body := make([]byte, 10*1024)
	for i := range body {
		body[i] = byte(i & 0xFF)
	}
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{Data: body, ContentType: "application/octet-stream", Status: 200}, nil
		},
	}
	srv := grpc.NewServer()
	serviceinvokev1.RegisterInvokeServiceServer(srv, &invokeHandler{deps: Deps{InvokeAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	defer conn.Close()
	client := serviceinvokev1.NewInvokeServiceClient(conn)
	stream, err := client.InvokeStream(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId: "tier2-foo", Method: "stream-bar", Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("InvokeStream: %v", err)
	}
	collected := make([]byte, 0, len(body))
	chunkCount := 0
	eofSeen := false
	for {
		chunk, err := stream.Recv()
		if err != nil {
			break
		}
		chunkCount++
		collected = append(collected, chunk.GetData()...)
		if chunk.GetEof() {
			eofSeen = true
			break
		}
	}
	if !eofSeen {
		t.Fatalf("eof flag not seen on last chunk")
	}
	if chunkCount != 3 {
		t.Fatalf("expected 3 chunks for 10 KiB body with 4 KiB chunk size, got %d", chunkCount)
	}
	if len(collected) != len(body) {
		t.Fatalf("collected size %d != body size %d", len(collected), len(body))
	}
	for i := range body {
		if collected[i] != body[i] {
			t.Fatalf("byte %d mismatch", i)
		}
	}
}

func TestInvokeService_InvokeStream_EmptyBody(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			return dapr.InvokeResponse{Data: nil, Status: 200}, nil
		},
	}
	srv := grpc.NewServer()
	serviceinvokev1.RegisterInvokeServiceServer(srv, &invokeHandler{deps: Deps{InvokeAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, _ := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	defer conn.Close()
	client := serviceinvokev1.NewInvokeServiceClient(conn)
	stream, err := client.InvokeStream(context.Background(), &serviceinvokev1.InvokeRequest{AppId: "x", Method: "y", Context: makeTenantCtx("T")})
	if err != nil {
		t.Fatalf("InvokeStream: %v", err)
	}
	chunk, err := stream.Recv()
	if err != nil {
		t.Fatalf("Recv: %v", err)
	}
	if !chunk.GetEof() {
		t.Fatalf("expected eof=true on empty body")
	}
	if len(chunk.GetData()) != 0 {
		t.Fatalf("expected empty data, got %d bytes", len(chunk.GetData()))
	}
}

// ---------------------------------------------------------------------------
// In-process gRPC integration: Invoke
// ---------------------------------------------------------------------------

func TestInvokeService_OverGRPC(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	captured := struct {
		appID, method string
	}{}
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			captured.appID = req.AppID
			captured.method = req.Method
			return dapr.InvokeResponse{Data: []byte("hello-world"), ContentType: "text/plain", Status: 200}, nil
		},
	}
	srv := grpc.NewServer()
	serviceinvokev1.RegisterInvokeServiceServer(srv, &invokeHandler{deps: Deps{InvokeAdapter: a}})
	go func() { _ = srv.Serve(lis) }()
	defer srv.Stop()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	conn, err := grpc.NewClient(
		"passthrough://bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("grpc.NewClient: %v", err)
	}
	defer conn.Close()

	client := serviceinvokev1.NewInvokeServiceClient(conn)
	resp, err := client.Invoke(context.Background(), &serviceinvokev1.InvokeRequest{
		AppId:   "tier2-foo",
		Method:  "bar",
		Context: makeTenantCtx("T"),
	})
	if err != nil {
		t.Fatalf("Invoke over gRPC: %v", err)
	}
	if string(resp.GetData()) != "hello-world" {
		t.Fatalf("data mismatch: %s", resp.GetData())
	}
	if captured.appID != "tier2-foo" || captured.method != "bar" {
		t.Fatalf("captured args: %+v", captured)
	}
}
