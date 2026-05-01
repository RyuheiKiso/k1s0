// 本ファイルは FR-T1-INVOKE-002 HTTP/1.1 互換プロキシ（POST /invoke/<target>/<method>）の
// 単体テスト。共通 /k1s0/serviceinvoke/invoke 経路（protojson 本体）と異なり、
// path から target/method を分解し、body を raw として透過転送することを確認する。

package common

import (
	"bytes"
	"context"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// metadataFromIncomingContext は metadata.FromIncomingContext のラッパ（型推論を簡素化）。
func metadataFromIncomingContext(ctx context.Context) (metadata.MD, bool) {
	return metadata.FromIncomingContext(ctx)
}

// fakeProxy は呼出を記録する単純な adapter。
type fakeProxy struct {
	captured InvokeProxyRequest
	resp     InvokeProxyResponse
	err      error
}

func (f *fakeProxy) ProxyInvoke(ctx context.Context, req InvokeProxyRequest) (InvokeProxyResponse, error) {
	f.captured = req
	return f.resp, f.err
}

// TestInvokeProxy_ParsesTargetAndMethod は URL から target / method を分解できることを確認する。
func TestInvokeProxy_ParsesTargetAndMethod(t *testing.T) {
	cases := []struct {
		path   string
		target string
		method string
	}{
		{"/invoke/myapp/calc", "myapp", "calc"},
		// target に "/" を含むケース（Dapr の app_id は "/" を許容）。
		{"/invoke/myapp/v1/orders/create", "myapp/v1/orders", "create"},
	}
	for _, c := range cases {
		t.Run(c.path, func(t *testing.T) {
			fp := &fakeProxy{resp: InvokeProxyResponse{Data: []byte("ok"), ContentType: "text/plain", Status: 200}}
			g := NewHTTPGateway()
			g.RegisterInvokeProxyRoute(fp)
			ts := httptest.NewServer(g.Handler())
			defer ts.Close()
			r, _ := http.NewRequest("POST", ts.URL+c.path, bytes.NewReader([]byte("payload")))
			r.Header.Set("Content-Type", "application/octet-stream")
			resp, err := http.DefaultClient.Do(r)
			if err != nil {
				t.Fatalf("http error: %v", err)
			}
			defer resp.Body.Close()
			if resp.StatusCode != 200 {
				t.Errorf("status = %d, want 200", resp.StatusCode)
			}
			if fp.captured.Target != c.target {
				t.Errorf("target = %q, want %q", fp.captured.Target, c.target)
			}
			if fp.captured.Method != c.method {
				t.Errorf("method = %q, want %q", fp.captured.Method, c.method)
			}
		})
	}
}

// TestInvokeProxy_PassesRawBody は body を raw のまま adapter に渡すことを確認する。
func TestInvokeProxy_PassesRawBody(t *testing.T) {
	fp := &fakeProxy{resp: InvokeProxyResponse{Data: []byte("ack"), ContentType: "application/json", Status: 200}}
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(fp)
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	body := []byte(`{"a":1}`)
	r, _ := http.NewRequest("POST", ts.URL+"/invoke/svc/m", bytes.NewReader(body))
	r.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(r)
	if err != nil {
		t.Fatalf("http error: %v", err)
	}
	defer resp.Body.Close()
	if !bytes.Equal(fp.captured.Body, body) {
		t.Errorf("body roundtrip: got %q want %q", fp.captured.Body, body)
	}
	if fp.captured.ContentType != "application/json" {
		t.Errorf("content-type forwarded: got %q", fp.captured.ContentType)
	}
	got, _ := io.ReadAll(resp.Body)
	if !bytes.Equal(got, []byte("ack")) {
		t.Errorf("response body: got %q", got)
	}
	if ct := resp.Header.Get("Content-Type"); ct != "application/json" {
		t.Errorf("response content-type: got %q", ct)
	}
}

// fakeProxyCaptureCtx は呼出時の context を chan に流す adapter（metadata 検証用）。
type fakeProxyCaptureCtx struct {
	ch   chan<- context.Context
	resp InvokeProxyResponse
}

func (f *fakeProxyCaptureCtx) ProxyInvoke(ctx context.Context, req InvokeProxyRequest) (InvokeProxyResponse, error) {
	f.ch <- ctx
	return f.resp, nil
}

// getMD は context の incoming gRPC metadata から最初の値を取り出す（テスト用 helper）。
func getMD(ctx context.Context, key string) string {
	md, _ := metadataFromIncomingContext(ctx)
	if md == nil {
		return ""
	}
	if vs := md.Get(key); len(vs) > 0 {
		return vs[0]
	}
	return ""
}

// TestInvokeProxy_ForwardsHeadersToMetadata は Authorization / traceparent を gRPC metadata に転送することを確認する。
func TestInvokeProxy_ForwardsHeadersToMetadata(t *testing.T) {
	captured := make(chan context.Context, 1)
	fp := &fakeProxyCaptureCtx{ch: captured, resp: InvokeProxyResponse{Data: []byte("ok"), Status: 200}}
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(fp)
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	r, _ := http.NewRequest("POST", ts.URL+"/invoke/svc/m", bytes.NewReader(nil))
	r.Header.Set("Authorization", "Bearer test")
	r.Header.Set("Traceparent", "00-aabbcc-1122-01")
	r.Header.Set("X-K1s0-Tenant-Id", "T-foo")
	r.Header.Set("X-K1s0-Idempotency-Key", "K-1")
	r.Header.Set("Content-Type", "application/octet-stream")
	resp, err := http.DefaultClient.Do(r)
	if err != nil {
		t.Fatalf("http error: %v", err)
	}
	resp.Body.Close()
	ctx := <-captured
	// gRPC incoming metadata から Authorization 等が取り出せるはず。
	if v := getMD(ctx, "authorization"); v != "Bearer test" {
		t.Errorf("authorization metadata: got %q", v)
	}
	if v := getMD(ctx, "traceparent"); v != "00-aabbcc-1122-01" {
		t.Errorf("traceparent metadata: got %q", v)
	}
	if v := getMD(ctx, "x-k1s0-tenant-id"); v != "T-foo" {
		t.Errorf("tenant metadata: got %q", v)
	}
	if v := getMD(ctx, "x-k1s0-idempotency-key"); v != "K-1" {
		t.Errorf("idempotency metadata: got %q", v)
	}
}

// TestInvokeProxy_RejectsNonPost は GET 等の非 POST を 405 で弾くことを確認する。
func TestInvokeProxy_RejectsNonPost(t *testing.T) {
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(&fakeProxy{})
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	resp, _ := http.Get(ts.URL + "/invoke/svc/m")
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusMethodNotAllowed {
		t.Errorf("status = %d, want 405", resp.StatusCode)
	}
	if resp.Header.Get("Allow") != "POST" {
		t.Errorf("allow header missing: got %q", resp.Header.Get("Allow"))
	}
}

// TestInvokeProxy_RejectsMalformedPath は target/method 不在 path を 400 で弾く。
func TestInvokeProxy_RejectsMalformedPath(t *testing.T) {
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(&fakeProxy{})
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	cases := []string{
		// target / method の片方しかない。
		"/invoke/m",
		// trailing slash で method が空。
		"/invoke/svc/m/",
	}
	for _, p := range cases {
		t.Run(p, func(t *testing.T) {
			resp, _ := http.Post(ts.URL+p, "application/octet-stream", bytes.NewReader(nil))
			defer resp.Body.Close()
			if resp.StatusCode != http.StatusBadRequest {
				t.Errorf("path %q: status = %d, want 400", p, resp.StatusCode)
			}
		})
	}
}

// TestInvokeProxy_MapsGRPCErrorToHTTPStatus は adapter が返した gRPC status を HTTP に変換することを確認する。
func TestInvokeProxy_MapsGRPCErrorToHTTPStatus(t *testing.T) {
	fp := &fakeProxy{err: status.Error(codes.PermissionDenied, "denied")}
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(fp)
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	resp, _ := http.Post(ts.URL+"/invoke/svc/m", "application/octet-stream", bytes.NewReader(nil))
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusForbidden {
		t.Errorf("status = %d, want 403 (PermissionDenied)", resp.StatusCode)
	}
}

// TestInvokeProxy_PassesAdapterError は adapter が plain error を返した場合に 500 になることを確認する。
func TestInvokeProxy_PassesAdapterError(t *testing.T) {
	fp := &fakeProxy{err: errors.New("boom")}
	g := NewHTTPGateway()
	g.RegisterInvokeProxyRoute(fp)
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()
	resp, _ := http.Post(ts.URL+"/invoke/svc/m", "application/octet-stream", bytes.NewReader(nil))
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusInternalServerError {
		t.Errorf("status = %d, want 500", resp.StatusCode)
	}
}
