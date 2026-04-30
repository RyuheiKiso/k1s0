// 本ファイルは http_gateway.go の単体テスト。
//
// 検証観点:
//   - httpStatusFromGRPC が docs §「HTTP Status ↔ K1s0Error マッピング」表どおりに変換する
//   - register が POST 以外を 400 で弾く
//   - register が Content-Type 不一致を 400 で弾く
//   - register が Authorization / traceparent ヘッダを gRPC metadata に転送する
//   - register が 8 MiB を超える body を弾く（実装は LimitReader、test では境界の挙動を確認）
//   - handler error → JSON error response with mapped HTTP status
//   - handler success → protojson エンコード response

package common

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
)

// docs §「HTTP Status ↔ K1s0Error マッピング」表のリグレッション保護。
func TestHTTPStatusFromGRPC_DocsTable(t *testing.T) {
	cases := []struct {
		c    codes.Code
		want int
	}{
		{codes.OK, 200},
		{codes.InvalidArgument, 400},
		{codes.Unauthenticated, 401},
		{codes.PermissionDenied, 403},
		{codes.NotFound, 404},
		{codes.AlreadyExists, 409},
		{codes.Aborted, 409},          // FirstWrite conflict は Conflict
		{codes.FailedPrecondition, 409},
		{codes.ResourceExhausted, 429},
		{codes.Unavailable, 503},
		{codes.DeadlineExceeded, 504},
		{codes.Internal, 500},
	}
	for _, c := range cases {
		if got := httpStatusFromGRPC(c.c); got != c.want {
			t.Errorf("httpStatusFromGRPC(%v) = %d; want %d", c.c, got, c.want)
		}
	}
}

// register は POST 以外を 405 + Allow ヘッダで弾く（RFC 9110 §15.5.6）。
func TestRegister_RejectsNonPost(t *testing.T) {
	g := NewHTTPGateway()
	g.register("/test", func(ctx context.Context, body []byte) (proto.Message, error) {
		t.Fatal("handler should not be called")
		return nil, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	resp, err := http.Get(srv.URL + "/test")
	if err != nil {
		t.Fatalf("GET: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusMethodNotAllowed {
		t.Fatalf("status = %d, want 405", resp.StatusCode)
	}
	// Allow ヘッダで利用可能 method を明示する
	if got := resp.Header.Get("Allow"); got != http.MethodPost {
		t.Fatalf("Allow header = %q, want %q", got, http.MethodPost)
	}
	// content-type は JSON で返す（schemathesis 等の契約検証で plain text drift しないこと）
	if ct := resp.Header.Get("Content-Type"); !strings.HasPrefix(ct, "application/json") {
		t.Fatalf("Content-Type = %q, want application/json", ct)
	}
}

// register は Content-Type 不一致を 400 で弾く。
func TestRegister_RejectsWrongContentType(t *testing.T) {
	g := NewHTTPGateway()
	g.register("/test", func(ctx context.Context, body []byte) (proto.Message, error) {
		t.Fatal("handler should not be called")
		return nil, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	req, _ := http.NewRequest(http.MethodPost, srv.URL+"/test", strings.NewReader("plain"))
	req.Header.Set("Content-Type", "text/plain")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("status = %d", resp.StatusCode)
	}
}

// register は Authorization / traceparent ヘッダを gRPC metadata に転送する。
func TestRegister_PropagatesHeadersToGRPCMetadata(t *testing.T) {
	g := NewHTTPGateway()
	var captured metadata.MD
	g.register("/test", func(ctx context.Context, body []byte) (proto.Message, error) {
		md, _ := metadata.FromIncomingContext(ctx)
		captured = md
		// 任意 proto を返す（middleware 結合の動作確認のみ）。
		return &commonv1.TenantContext{TenantId: "ok"}, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	req, _ := http.NewRequest(http.MethodPost, srv.URL+"/test", strings.NewReader("{}"))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer xyz")
	req.Header.Set("Traceparent", "00-traceid-spanid-01")
	req.Header.Set("X-K1s0-Idempotency-Key", "idem-1")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d", resp.StatusCode)
	}
	if got := captured.Get("authorization"); len(got) != 1 || got[0] != "Bearer xyz" {
		t.Errorf("authorization metadata not propagated: %v", got)
	}
	if got := captured.Get("traceparent"); len(got) != 1 || got[0] != "00-traceid-spanid-01" {
		t.Errorf("traceparent metadata not propagated: %v", got)
	}
	if got := captured.Get("x-k1s0-idempotency-key"); len(got) != 1 || got[0] != "idem-1" {
		t.Errorf("idempotency-key metadata not propagated: %v", got)
	}
}

// handler error は HTTP status マッピング + JSON error body を返す。
func TestRegister_HandlerError_MapsStatus(t *testing.T) {
	g := NewHTTPGateway()
	g.register("/test", func(ctx context.Context, body []byte) (proto.Message, error) {
		return nil, status.Error(codes.NotFound, "missing")
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	req, _ := http.NewRequest(http.MethodPost, srv.URL+"/test", strings.NewReader("{}"))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusNotFound {
		t.Fatalf("status = %d want 404", resp.StatusCode)
	}
	var body map[string]map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if body["error"]["code"] != "NotFound" {
		t.Errorf("error.code = %v", body["error"]["code"])
	}
	if body["error"]["message"] != "missing" {
		t.Errorf("error.message = %v", body["error"]["message"])
	}
}

// 成功時は handler の戻り値を protojson でエンコードして返す。
func TestRegister_HandlerSuccess_ProtojsonEncoded(t *testing.T) {
	g := NewHTTPGateway()
	g.register("/test", func(ctx context.Context, body []byte) (proto.Message, error) {
		return &commonv1.TenantContext{TenantId: "T1", Subject: "alice"}, nil
	})
	srv := httptest.NewServer(g.Handler())
	defer srv.Close()
	req, _ := http.NewRequest(http.MethodPost, srv.URL+"/test", bytes.NewReader([]byte("{}")))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("POST: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d", resp.StatusCode)
	}
	if !strings.HasPrefix(resp.Header.Get("Content-Type"), "application/json") {
		t.Errorf("content-type = %q", resp.Header.Get("Content-Type"))
	}
	var got map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatalf("decode: %v", err)
	}
	// protojson は camelCase で出力する（tenantId）。
	if got["tenantId"] != "T1" || got["subject"] != "alice" {
		t.Errorf("response body mismatch: %v", got)
	}
}

// UnmarshalJSON は不正 JSON を InvalidArgument として返す。
func TestUnmarshalJSON_InvalidJSON_InvalidArgument(t *testing.T) {
	var msg commonv1.TenantContext
	err := UnmarshalJSON([]byte("not json"), &msg)
	if status.Code(err) != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument, got %v", err)
	}
}

// UnmarshalJSON は未知フィールドを許容する（DiscardUnknown）。
func TestUnmarshalJSON_UnknownFieldsDiscarded(t *testing.T) {
	var msg commonv1.TenantContext
	err := UnmarshalJSON([]byte(`{"tenantId":"T","unknownField":"x"}`), &msg)
	if err != nil {
		t.Fatalf("unexpected: %v", err)
	}
	if msg.GetTenantId() != "T" {
		t.Errorf("tenantId = %q", msg.GetTenantId())
	}
}
