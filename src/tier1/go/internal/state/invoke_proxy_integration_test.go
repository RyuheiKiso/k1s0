// 本ファイルは FR-T1-INVOKE-002 HTTP/1.1 互換プロキシの統合テスト。
//
// 観点:
//   - /invoke/<target>/<method> 形式の HTTP/1.1 リクエストを送信
//   - 実 invokeHandler 経由で fake adapter にディスパッチされ、認証ヘッダ転写と
//     timeout 適用がエンドツーエンドで動作することを確認

package state

import (
	"bytes"
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	"google.golang.org/grpc/metadata"
)

// TestInvokeProxyIntegration_EndToEnd は HTTP/1.1 → invokeHandler → adapter の
// 全パスを fake adapter で観測する。
func TestInvokeProxyIntegration_EndToEnd(t *testing.T) {
	// adapter の呼出を観測する。
	type captured struct {
		appID       string
		method      string
		body        []byte
		contentType string
		hasDeadline bool
		auth        string
	}
	cap := &captured{}
	a := &fakeInvokeAdapter{
		fn: func(ctx context.Context, req dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			cap.appID = req.AppID
			cap.method = req.Method
			cap.body = req.Data
			cap.contentType = req.ContentType
			_, cap.hasDeadline = ctx.Deadline()
			// outgoing metadata から forwarded auth を取り出す。
			if md, ok := metadata.FromOutgoingContext(ctx); ok {
				if vs := md.Get("authorization"); len(vs) > 0 {
					cap.auth = vs[0]
				}
			}
			return dapr.InvokeResponse{
				Data:        []byte(`{"result":"ok"}`),
				ContentType: "application/json",
				Status:      200,
			}, nil
		},
	}
	deps := Deps{InvokeAdapter: a}
	svc := NewInvokeServiceServer(deps)
	g := common.NewHTTPGateway()
	g.RegisterInvokeProxyRoute(NewInvokeProxyAdapter(svc))
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()

	body := []byte(`{"customer_id":42}`)
	req, _ := http.NewRequest(http.MethodPost, ts.URL+"/invoke/billing-svc/charge", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer test-jwt")
	req.Header.Set("X-K1s0-Tenant-Id", "T-foo")
	req.Header.Set("X-K1s0-Timeout-Ms", "1500")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("http error: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		t.Errorf("status = %d, want 200", resp.StatusCode)
	}
	respBody, _ := io.ReadAll(resp.Body)
	if !bytes.Equal(respBody, []byte(`{"result":"ok"}`)) {
		t.Errorf("body = %q, want {\"result\":\"ok\"}", respBody)
	}
	if ct := resp.Header.Get("Content-Type"); ct != "application/json" {
		t.Errorf("content-type = %q, want application/json", ct)
	}
	// adapter に渡った値を検証する。
	if cap.appID != "billing-svc" || cap.method != "charge" {
		t.Errorf("dispatch mismatch: target=%q method=%q", cap.appID, cap.method)
	}
	if !bytes.Equal(cap.body, body) {
		t.Errorf("body roundtrip mismatch: got %q", cap.body)
	}
	if cap.contentType != "application/json" {
		t.Errorf("content-type forwarded: %q", cap.contentType)
	}
	if !cap.hasDeadline {
		t.Error("adapter context has no deadline; expected timeout applied")
	}
	if cap.auth != "Bearer test-jwt" {
		t.Errorf("authorization not forwarded to adapter outgoing metadata: %q", cap.auth)
	}
}

// TestInvokeProxyIntegration_EnforcesTenant は X-K1s0-Tenant-Id 不在時に
// invokeHandler が tenant_id 必須検証で 400 を返すことを確認する。
func TestInvokeProxyIntegration_EnforcesTenant(t *testing.T) {
	a := &fakeInvokeAdapter{
		fn: func(_ context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			t.Fatal("adapter should not be reached when tenant_id is missing")
			return dapr.InvokeResponse{}, nil
		},
	}
	deps := Deps{InvokeAdapter: a}
	svc := NewInvokeServiceServer(deps)
	g := common.NewHTTPGateway()
	g.RegisterInvokeProxyRoute(NewInvokeProxyAdapter(svc))
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()

	req, _ := http.NewRequest(http.MethodPost, ts.URL+"/invoke/svc/m", bytes.NewReader([]byte("x")))
	req.Header.Set("Content-Type", "application/octet-stream")
	// X-K1s0-Tenant-Id をわざと付けない。
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("http error: %v", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest {
		t.Errorf("status = %d, want 400 (tenant_id required)", resp.StatusCode)
	}
}

// TestInvokeProxyIntegration_AppliesDefaultTimeout は X-K1s0-Timeout-Ms 未指定時に
// invokeDefaultTimeout（3 秒）相当の deadline が adapter に設定されることを確認する。
func TestInvokeProxyIntegration_AppliesDefaultTimeout(t *testing.T) {
	var capturedDeadline time.Time
	a := &fakeInvokeAdapter{
		fn: func(ctx context.Context, _ dapr.InvokeRequest) (dapr.InvokeResponse, error) {
			capturedDeadline, _ = ctx.Deadline()
			return dapr.InvokeResponse{Data: []byte(""), ContentType: "", Status: 200}, nil
		},
	}
	deps := Deps{InvokeAdapter: a}
	svc := NewInvokeServiceServer(deps)
	g := common.NewHTTPGateway()
	g.RegisterInvokeProxyRoute(NewInvokeProxyAdapter(svc))
	ts := httptest.NewServer(g.Handler())
	defer ts.Close()

	req, _ := http.NewRequest(http.MethodPost, ts.URL+"/invoke/svc/m", bytes.NewReader(nil))
	req.Header.Set("Content-Type", "application/octet-stream")
	req.Header.Set("X-K1s0-Tenant-Id", "T-foo")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("http error: %v", err)
	}
	resp.Body.Close()
	// adapter context の deadline が「これから 3 秒」近辺であること。
	if capturedDeadline.IsZero() {
		t.Fatal("no deadline captured")
	}
	d := time.Until(capturedDeadline)
	// HTTP gateway が 5s wrap、その上に invoke handler が 3s wrap → 早い方の 3s が effective。
	if d <= 0 || d > invokeDefaultTimeout {
		t.Errorf("deadline %v not in (0, %v]", d, invokeDefaultTimeout)
	}
}

