// 本ファイルは BFF rest router の State 系 endpoint のうち、Save / Delete の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - POST /api/state/save: 入力検証 (400) / upstream エラー (502) / 成功 (200 + etag)
//   - POST /api/state/delete: 入力検証 (400) / upstream エラー (502) / 成功 (200)
//
// Get 側の網羅テストは router_test.go に集約済（既存維持）。

package rest

// 標準 / 内部 import。
import (
	// JSON エンコード / デコード。
	"bytes"
	"context"
	"encoding/json"
	// errors 生成。
	"errors"
	// HTTP server。
	"net/http"
	// テスト用 HTTP server。
	"net/http/httptest"
	// テスト frame。
	"testing"
)

// fakeStateSaveDelete は Save / Delete を override した Facade mock。
type fakeStateSaveDelete struct {
	// no-op の基底実装を embed する。
	unimplementedFacade
	// Save 呼出の記録。
	saveCalls []struct {
		Store, Key string
		Data       []byte
	}
	// Save 応答。
	saveEtag string
	saveErr  error
	// Delete 呼出の記録。
	deleteCalls []struct {
		Store, Key, ExpectedEtag string
	}
	// Delete 応答。
	deleteErr error
}

// StateSave を override する。
func (f *fakeStateSaveDelete) StateSave(_ context.Context, store, key string, data []byte) (string, error) {
	f.saveCalls = append(f.saveCalls, struct {
		Store, Key string
		Data       []byte
	}{store, key, data})
	return f.saveEtag, f.saveErr
}

// StateDelete を override する。
func (f *fakeStateSaveDelete) StateDelete(_ context.Context, store, key, expectedEtag string) error {
	f.deleteCalls = append(f.deleteCalls, struct {
		Store, Key, ExpectedEtag string
	}{store, key, expectedEtag})
	return f.deleteErr
}

// newSaveDeleteServer は test 用の httptest.Server を組む。
func newSaveDeleteServer(t *testing.T, fake *fakeStateSaveDelete) *httptest.Server {
	t.Helper()
	mux := http.NewServeMux()
	NewRouter(fake).Register(mux)
	return httptest.NewServer(mux)
}

// postBody は path に JSON body を POST する。
func postBody(t *testing.T, baseURL, path string, body any) (*http.Response, []byte) {
	t.Helper()
	buf, err := json.Marshal(body)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	resp, err := http.Post(baseURL+path, "application/json", bytes.NewReader(buf))
	if err != nil {
		t.Fatalf("post: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	out := new(bytes.Buffer)
	if _, err := out.ReadFrom(resp.Body); err != nil {
		t.Fatalf("read: %v", err)
	}
	return resp, out.Bytes()
}

func TestStateSave_RejectsMissingFields(t *testing.T) {
	srv := newSaveDeleteServer(t, &fakeStateSaveDelete{})
	defer srv.Close()
	cases := []map[string]string{
		{},                        // 両方欠落
		{"store": "s"},            // key 欠落
		{"key": "k"},              // store 欠落
		{"store": "", "key": "k"}, // 空 store
	}
	for i, body := range cases {
		resp, _ := postBody(t, srv.URL, "/api/state/save", body)
		if resp.StatusCode != http.StatusBadRequest {
			t.Errorf("case %d: expected 400, got %d (body=%v)", i, resp.StatusCode, body)
		}
	}
}

func TestStateSave_PropagatesUpstreamError(t *testing.T) {
	fake := &fakeStateSaveDelete{saveErr: errors.New("upstream broken")}
	srv := newSaveDeleteServer(t, fake)
	defer srv.Close()
	resp, _ := postBody(t, srv.URL, "/api/state/save", map[string]string{"store": "s", "key": "k", "data": "d"})
	if resp.StatusCode != http.StatusBadGateway {
		t.Errorf("upstream error should be 502, got %d", resp.StatusCode)
	}
	if len(fake.saveCalls) != 1 || fake.saveCalls[0].Store != "s" || fake.saveCalls[0].Key != "k" || string(fake.saveCalls[0].Data) != "d" {
		t.Errorf("StateSave args lost: calls=%v", fake.saveCalls)
	}
}

func TestStateSave_SuccessReturns200WithEtag(t *testing.T) {
	fake := &fakeStateSaveDelete{saveEtag: "etag-7"}
	srv := newSaveDeleteServer(t, fake)
	defer srv.Close()
	resp, body := postBody(t, srv.URL, "/api/state/save", map[string]string{"store": "s", "key": "k", "data": "payload"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	var got stateSaveResponse
	if err := json.Unmarshal(body, &got); err != nil {
		t.Fatalf("response not json: %v / body=%s", err, body)
	}
	if got.Etag != "etag-7" {
		t.Errorf("expected etag=etag-7, got %q", got.Etag)
	}
}

func TestStateDelete_RejectsMissingFields(t *testing.T) {
	srv := newSaveDeleteServer(t, &fakeStateSaveDelete{})
	defer srv.Close()
	cases := []map[string]string{
		{},
		{"store": "s"},
		{"key": "k"},
	}
	for i, body := range cases {
		resp, _ := postBody(t, srv.URL, "/api/state/delete", body)
		if resp.StatusCode != http.StatusBadRequest {
			t.Errorf("case %d: expected 400, got %d (body=%v)", i, resp.StatusCode, body)
		}
	}
}

func TestStateDelete_PropagatesUpstreamError(t *testing.T) {
	fake := &fakeStateSaveDelete{deleteErr: errors.New("upstream gone")}
	srv := newSaveDeleteServer(t, fake)
	defer srv.Close()
	resp, _ := postBody(t, srv.URL, "/api/state/delete", map[string]string{"store": "s", "key": "k"})
	if resp.StatusCode != http.StatusBadGateway {
		t.Errorf("upstream error should be 502, got %d", resp.StatusCode)
	}
}

func TestStateDelete_SuccessReturns200(t *testing.T) {
	fake := &fakeStateSaveDelete{}
	srv := newSaveDeleteServer(t, fake)
	defer srv.Close()
	resp, _ := postBody(t, srv.URL, "/api/state/delete", map[string]string{"store": "s", "key": "k", "expected_etag": "e1"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	if len(fake.deleteCalls) != 1 || fake.deleteCalls[0].ExpectedEtag != "e1" {
		t.Errorf("expected_etag not propagated: calls=%v", fake.deleteCalls)
	}
}
