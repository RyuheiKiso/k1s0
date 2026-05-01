// 本ファイルは BFF rest router の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - POST /api/state/get の入力検証（空 body / 必須項目欠落 → 400）
//   - StateClient interface 経由で mock を注入し、上流エラー → 502 / 成功 → 200 を確認
//   - 成功 response の JSON schema（data / etag / found）
//
// 設計判断:
//   Router は StateClient interface に依存するため、本 test は in-memory mock で
//   外部接続不要に検証する（k1s0client.Client は実 SDK 接続が要るため別途 integration test）。

package rest

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// fakeStateClient は Facade interface を満たす in-memory mock。
// 14 サービス分の no-op method は unimplementedFacade を embed して継承する。
// State.Get のみテスト用に override する。
type fakeStateClient struct {
	// no-op の基底実装を embed する（State.Get 以外は zero value 応答）。
	unimplementedFacade
	// 直前の呼出パラメータ（assert 用）。
	gotStore string
	gotKey   string
	// 返却値。
	respData  []byte
	respEtag  string
	respFound bool
	respErr   error
}

// StateGet を override し、呼出を記録して固定値を返す。
func (f *fakeStateClient) StateGet(_ context.Context, store, key string) ([]byte, string, bool, error) {
	f.gotStore = store
	f.gotKey = key
	return f.respData, f.respEtag, f.respFound, f.respErr
}

func newTestServer(t *testing.T, fake *fakeStateClient) *httptest.Server {
	t.Helper()
	mux := http.NewServeMux()
	NewRouter(fake).Register(mux)
	return httptest.NewServer(mux)
}

func postJSON(t *testing.T, url string, body any) (*http.Response, []byte) {
	t.Helper()
	buf, err := json.Marshal(body)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	resp, err := http.Post(url+"/api/state/get", "application/json", bytes.NewReader(buf))
	if err != nil {
		t.Fatalf("post: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	out := new(bytes.Buffer)
	if _, err := out.ReadFrom(resp.Body); err != nil {
		t.Fatalf("read body: %v", err)
	}
	return resp, out.Bytes()
}

func TestStateGet_RejectsInvalidJSON(t *testing.T) {
	srv := newTestServer(t, &fakeStateClient{})
	defer srv.Close()
	resp, err := http.Post(srv.URL+"/api/state/get", "application/json", strings.NewReader("not-json"))
	if err != nil {
		t.Fatalf("post: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Fatalf("invalid json should be 400, got %d", resp.StatusCode)
	}
}

func TestStateGet_RejectsMissingStoreOrKey(t *testing.T) {
	srv := newTestServer(t, &fakeStateClient{})
	defer srv.Close()

	cases := []map[string]string{
		{},                              // 両方欠落
		{"store": "s"},                  // key 欠落
		{"key": "k"},                    // store 欠落
		{"store": "", "key": "k"},       // 空 store
	}
	for i, body := range cases {
		resp, _ := postJSON(t, srv.URL, body)
		if resp.StatusCode != http.StatusBadRequest {
			t.Errorf("case %d: expected 400, got %d (body=%v)", i, resp.StatusCode, body)
		}
	}
}

func TestStateGet_PropagatesUpstreamError(t *testing.T) {
	fake := &fakeStateClient{respErr: errors.New("upstream gone")}
	srv := newTestServer(t, fake)
	defer srv.Close()
	resp, _ := postJSON(t, srv.URL, map[string]string{"store": "s", "key": "k"})
	if resp.StatusCode != http.StatusBadGateway {
		t.Errorf("upstream error should be 502, got %d", resp.StatusCode)
	}
	// fake 側に呼出が届いている。
	if fake.gotStore != "s" || fake.gotKey != "k" {
		t.Errorf("StateGet args lost: store=%q key=%q", fake.gotStore, fake.gotKey)
	}
}

func TestStateGet_SuccessReturns200WithJSONBody(t *testing.T) {
	fake := &fakeStateClient{
		respData:  []byte("hello"),
		respEtag:  "etag-1",
		respFound: true,
	}
	srv := newTestServer(t, fake)
	defer srv.Close()
	resp, body := postJSON(t, srv.URL, map[string]string{"store": "s", "key": "k"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	if ct := resp.Header.Get("Content-Type"); !strings.HasPrefix(ct, "application/json") {
		t.Errorf("Content-Type = %q", ct)
	}
	var got stateGetResponse
	if err := json.Unmarshal(body, &got); err != nil {
		t.Fatalf("response not json: %v / body=%s", err, body)
	}
	if got.Data != "hello" || got.Etag != "etag-1" || !got.Found {
		t.Errorf("unexpected response: %+v", got)
	}
}

func TestStateGet_NotFoundIsOk200(t *testing.T) {
	// not-found（found=false）も 200 で返す（SDK 仕様、404 化はしない）。
	fake := &fakeStateClient{respFound: false}
	srv := newTestServer(t, fake)
	defer srv.Close()
	resp, body := postJSON(t, srv.URL, map[string]string{"store": "s", "key": "missing"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("not found should still be 200, got %d", resp.StatusCode)
	}
	var got stateGetResponse
	_ = json.Unmarshal(body, &got)
	if got.Found {
		t.Errorf("found should be false, got %+v", got)
	}
}
