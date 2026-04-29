// 本ファイルは BFF GraphQL Resolver の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - HTTP method 制限（POST 以外は 405）
//   - 不正 JSON は 400
//   - stateGet クエリ: success / not-found / upstream error の 3 パス
//   - currentUser クエリ: 既定 anonymous resolver
//   - 未知クエリ: errors[].message="unsupported query"

package graphql

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

// fakeStateClient は StateClient を満たす in-memory mock。
type fakeStateClient struct {
	gotStore  string
	gotKey    string
	respData  []byte
	respEtag  string
	respFound bool
	respErr   error
}

func (f *fakeStateClient) StateGet(_ context.Context, store, key string) ([]byte, string, bool, error) {
	f.gotStore = store
	f.gotKey = key
	return f.respData, f.respEtag, f.respFound, f.respErr
}

func newGraphqlServer(t *testing.T, fake *fakeStateClient) *httptest.Server {
	t.Helper()
	mux := http.NewServeMux()
	mux.HandleFunc("/graphql", NewResolver(fake).Handler())
	return httptest.NewServer(mux)
}

func postGQL(t *testing.T, url, query string, vars map[string]any) (*http.Response, []byte) {
	t.Helper()
	body, err := json.Marshal(graphqlRequest{Query: query, Variables: vars})
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	resp, err := http.Post(url+"/graphql", "application/json", bytes.NewReader(body))
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

func TestRejectsNonPost(t *testing.T) {
	srv := newGraphqlServer(t, &fakeStateClient{})
	defer srv.Close()
	resp, err := http.Get(srv.URL + "/graphql")
	if err != nil {
		t.Fatalf("get: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusMethodNotAllowed {
		t.Errorf("GET should be 405, got %d", resp.StatusCode)
	}
}

func TestRejectsInvalidJSON(t *testing.T) {
	srv := newGraphqlServer(t, &fakeStateClient{})
	defer srv.Close()
	resp, err := http.Post(srv.URL+"/graphql", "application/json", strings.NewReader("not-json"))
	if err != nil {
		t.Fatalf("post: %v", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusBadRequest {
		t.Errorf("invalid json should be 400, got %d", resp.StatusCode)
	}
}

func TestStateGet_SuccessPath(t *testing.T) {
	fake := &fakeStateClient{
		respData:  []byte("payload"),
		respEtag:  "v1",
		respFound: true,
	}
	srv := newGraphqlServer(t, fake)
	defer srv.Close()
	resp, body := postGQL(t, srv.URL,
		`query { stateGet(store: $s, key: $k) { data etag } }`,
		map[string]any{"store": "kvstore", "key": "user/1"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d, body=%s", resp.StatusCode, body)
	}
	// resolver は client から呼出パラメータを受けたか確認。
	if fake.gotStore != "kvstore" || fake.gotKey != "user/1" {
		t.Errorf("StateGet args lost: store=%q key=%q", fake.gotStore, fake.gotKey)
	}
	var got graphqlResponse
	if err := json.Unmarshal(body, &got); err != nil {
		t.Fatalf("response not json: %v", err)
	}
	if len(got.Errors) != 0 {
		t.Errorf("expected no errors, got %v", got.Errors)
	}
	// data.stateGet.data == "payload"
	dataMap, ok := got.Data.(map[string]any)
	if !ok {
		t.Fatalf("data is not map: %T", got.Data)
	}
	stateGet, ok := dataMap["stateGet"].(map[string]any)
	if !ok {
		t.Fatalf("stateGet is not map: %v", dataMap)
	}
	if stateGet["data"] != "payload" || stateGet["etag"] != "v1" {
		t.Errorf("unexpected stateGet: %v", stateGet)
	}
}

func TestStateGet_NotFoundReturnsNullData(t *testing.T) {
	fake := &fakeStateClient{respFound: false}
	srv := newGraphqlServer(t, fake)
	defer srv.Close()
	resp, body := postGQL(t, srv.URL,
		`query { stateGet(store: $s, key: $k) { data } }`,
		map[string]any{"store": "kvstore", "key": "missing"})
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	var got graphqlResponse
	_ = json.Unmarshal(body, &got)
	dataMap, _ := got.Data.(map[string]any)
	if dataMap == nil {
		t.Fatalf("data missing: body=%s", body)
	}
	// not-found は data.stateGet=null。json でデコードしたら nil interface。
	if v, ok := dataMap["stateGet"]; !ok || v != nil {
		t.Errorf("expected stateGet=null, got %v (ok=%v)", v, ok)
	}
}

func TestStateGet_UpstreamErrorPropagated(t *testing.T) {
	fake := &fakeStateClient{respErr: errors.New("upstream broken")}
	srv := newGraphqlServer(t, fake)
	defer srv.Close()
	resp, body := postGQL(t, srv.URL,
		`{ stateGet(store: $s, key: $k) { data } }`,
		map[string]any{"store": "s", "key": "k"})
	// HTTP は 200（GraphQL 慣用: errors 配列に入れる）。
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	var got graphqlResponse
	_ = json.Unmarshal(body, &got)
	if len(got.Errors) == 0 {
		t.Fatalf("expected errors, got %v", got)
	}
	msg, _ := got.Errors[0]["message"].(string)
	if !strings.Contains(msg, "upstream broken") {
		t.Errorf("expected error message to contain upstream cause, got %q", msg)
	}
}

func TestCurrentUser_ReturnsAnonymousPlaceholder(t *testing.T) {
	srv := newGraphqlServer(t, &fakeStateClient{})
	defer srv.Close()
	resp, body := postGQL(t, srv.URL, `{ currentUser { id roles } }`, nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	var got graphqlResponse
	_ = json.Unmarshal(body, &got)
	dataMap, ok := got.Data.(map[string]any)
	if !ok {
		t.Fatalf("data not map: %v", got.Data)
	}
	user, ok := dataMap["currentUser"].(map[string]any)
	if !ok {
		t.Fatalf("currentUser not map: %v", dataMap)
	}
	if user["id"] != "anonymous" {
		t.Errorf("expected id=anonymous, got %v", user["id"])
	}
}

func TestUnknownQuery_ReturnsErrorEntry(t *testing.T) {
	srv := newGraphqlServer(t, &fakeStateClient{})
	defer srv.Close()
	resp, body := postGQL(t, srv.URL, `{ somethingNotImplemented { x } }`, nil)
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
	var got graphqlResponse
	_ = json.Unmarshal(body, &got)
	if len(got.Errors) == 0 {
		t.Fatalf("expected errors[]: body=%s", body)
	}
	msg, _ := got.Errors[0]["message"].(string)
	if msg != "unsupported query" {
		t.Errorf("expected unsupported query, got %q", msg)
	}
}
