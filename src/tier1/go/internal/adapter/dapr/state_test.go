// 本ファイルは daprStateAdapter（State Management ラッパ）の単体テスト。
//
// 試験戦略:
//   Dapr SDK との結合点である `daprStateClient` を fake 実装で差し替え、
//   adapter が SDK へ渡すメソッド・引数・metadata を直接検証する。
//   Dapr sidecar / Valkey の実 deploy は要求しない（CI 上で実行可能）。

package dapr

import (
	"context"
	"errors"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// fakeStateClient は daprStateClient の最小 fake 実装。
// 試験ごとに各メソッドの fn を差し替え、引数キャプチャと戻り値制御を行う。
type fakeStateClient struct {
	// GetState の fn。nil なら fail。
	getFn func(ctx context.Context, store, key string, meta map[string]string) (*daprclient.StateItem, error)
	// SaveState の fn。
	saveFn func(ctx context.Context, store, key string, data []byte, meta map[string]string, so ...daprclient.StateOption) error
	// SaveStateWithETag の fn。
	saveETagFn func(ctx context.Context, store, key string, data []byte, etag string, meta map[string]string, so ...daprclient.StateOption) error
	// DeleteState の fn。
	deleteFn func(ctx context.Context, store, key string, meta map[string]string) error
	// DeleteStateWithETag の fn。
	deleteETagFn func(ctx context.Context, store, key string, etag *daprclient.ETag, meta map[string]string, opts *daprclient.StateOptions) error
}

func (f *fakeStateClient) GetState(ctx context.Context, store, key string, meta map[string]string) (*daprclient.StateItem, error) {
	return f.getFn(ctx, store, key, meta)
}
func (f *fakeStateClient) SaveState(ctx context.Context, store, key string, data []byte, meta map[string]string, so ...daprclient.StateOption) error {
	return f.saveFn(ctx, store, key, data, meta, so...)
}
func (f *fakeStateClient) SaveStateWithETag(ctx context.Context, store, key string, data []byte, etag string, meta map[string]string, so ...daprclient.StateOption) error {
	return f.saveETagFn(ctx, store, key, data, etag, meta, so...)
}
func (f *fakeStateClient) DeleteState(ctx context.Context, store, key string, meta map[string]string) error {
	return f.deleteFn(ctx, store, key, meta)
}
func (f *fakeStateClient) DeleteStateWithETag(ctx context.Context, store, key string, etag *daprclient.ETag, meta map[string]string, opts *daprclient.StateOptions) error {
	return f.deleteETagFn(ctx, store, key, etag, meta, opts)
}

// newAdapterWithFake は test helper。fake から StateAdapter を構築する。
func newAdapterWithFake(t *testing.T, fake *fakeStateClient) StateAdapter {
	t.Helper()
	cli := NewWithStateClient("test://noop", fake)
	return NewStateAdapter(cli)
}

// Get がキー存在時に StateItem.Value と Etag を返却することを検証する。
func TestStateAdapter_Get_Found(t *testing.T) {
	want := []byte("hello")
	fake := &fakeStateClient{
		getFn: func(_ context.Context, store, key string, meta map[string]string) (*daprclient.StateItem, error) {
			if store != "valkey-default" {
				t.Fatalf("store mismatch: got %q want valkey-default", store)
			}
			if key != "user:42" {
				t.Fatalf("key mismatch: got %q", key)
			}
			if got := meta["tenantId"]; got != "tenant-A" {
				t.Fatalf("tenantId metadata mismatch: got %q", got)
			}
			return &daprclient.StateItem{Key: key, Value: want, Etag: "v1"}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Get(context.Background(), StateGetRequest{Store: "valkey-default", Key: "user:42", TenantID: "tenant-A"})
	if err != nil {
		t.Fatalf("Get returned error: %v", err)
	}
	if resp.NotFound {
		t.Fatalf("expected NotFound=false")
	}
	if string(resp.Data) != string(want) {
		t.Fatalf("data mismatch: got %q want %q", resp.Data, want)
	}
	if resp.Etag != "v1" {
		t.Fatalf("etag mismatch: got %q want v1", resp.Etag)
	}
}

// Get がキー未存在時に NotFound=true を返すことを検証する。
// Dapr SDK は Value=nil で未存在を表現する。
func TestStateAdapter_Get_NotFound(t *testing.T) {
	fake := &fakeStateClient{
		getFn: func(_ context.Context, _, _ string, _ map[string]string) (*daprclient.StateItem, error) {
			return &daprclient.StateItem{Key: "x", Value: nil}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Get(context.Background(), StateGetRequest{Store: "s", Key: "x"})
	if err != nil {
		t.Fatalf("Get returned error: %v", err)
	}
	if !resp.NotFound {
		t.Fatalf("expected NotFound=true")
	}
}

// Get が SDK エラーを上位へ透過することを検証する。
func TestStateAdapter_Get_SDKError(t *testing.T) {
	want := errors.New("rpc unavailable")
	fake := &fakeStateClient{
		getFn: func(_ context.Context, _, _ string, _ map[string]string) (*daprclient.StateItem, error) {
			return nil, want
		},
	}
	a := newAdapterWithFake(t, fake)
	_, err := a.Get(context.Background(), StateGetRequest{Store: "s", Key: "x"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: got %v want %v", err, want)
	}
}

// Set が ExpectedEtag 空時に SaveState を、非空時に SaveStateWithETag を呼ぶことを検証する。
func TestStateAdapter_Set_NoEtagThenWithEtag(t *testing.T) {
	saveCalled := 0
	saveETagCalled := 0
	var observedEtag string
	fake := &fakeStateClient{
		saveFn: func(_ context.Context, store, key string, data []byte, meta map[string]string, _ ...daprclient.StateOption) error {
			saveCalled++
			if store != "s" || key != "k" || string(data) != "v" {
				t.Fatalf("save args mismatch: %s %s %s", store, key, data)
			}
			if meta["ttlInSeconds"] != "60" {
				t.Fatalf("ttl metadata mismatch: %v", meta)
			}
			return nil
		},
		saveETagFn: func(_ context.Context, _, _ string, _ []byte, etag string, _ map[string]string, _ ...daprclient.StateOption) error {
			saveETagCalled++
			observedEtag = etag
			return nil
		},
	}
	a := newAdapterWithFake(t, fake)
	// 1. ETag 空 → SaveState
	if _, err := a.Set(context.Background(), StateSetRequest{Store: "s", Key: "k", Data: []byte("v"), TTLSeconds: 60}); err != nil {
		t.Fatalf("Set (no etag) error: %v", err)
	}
	if saveCalled != 1 || saveETagCalled != 0 {
		t.Fatalf("SaveState not called as expected: save=%d saveETag=%d", saveCalled, saveETagCalled)
	}
	// 2. ETag 非空 → SaveStateWithETag
	if _, err := a.Set(context.Background(), StateSetRequest{Store: "s", Key: "k", Data: []byte("v"), ExpectedEtag: "v7"}); err != nil {
		t.Fatalf("Set (with etag) error: %v", err)
	}
	if saveCalled != 1 || saveETagCalled != 1 {
		t.Fatalf("SaveStateWithETag not called as expected: save=%d saveETag=%d", saveCalled, saveETagCalled)
	}
	if observedEtag != "v7" {
		t.Fatalf("etag mismatch: got %q want v7", observedEtag)
	}
}

// Delete が ExpectedEtag 空時に DeleteState を、非空時に DeleteStateWithETag を呼ぶことを検証する。
func TestStateAdapter_Delete_NoEtagThenWithEtag(t *testing.T) {
	delCalled := 0
	delETagCalled := 0
	var observedEtag string
	fake := &fakeStateClient{
		deleteFn: func(_ context.Context, _, _ string, _ map[string]string) error {
			delCalled++
			return nil
		},
		deleteETagFn: func(_ context.Context, _, _ string, etag *daprclient.ETag, _ map[string]string, _ *daprclient.StateOptions) error {
			delETagCalled++
			if etag != nil {
				observedEtag = etag.Value
			}
			return nil
		},
	}
	a := newAdapterWithFake(t, fake)
	if err := a.Delete(context.Background(), StateSetRequest{Store: "s", Key: "k"}); err != nil {
		t.Fatalf("Delete (no etag) error: %v", err)
	}
	if delCalled != 1 || delETagCalled != 0 {
		t.Fatalf("DeleteState path mismatch")
	}
	if err := a.Delete(context.Background(), StateSetRequest{Store: "s", Key: "k", ExpectedEtag: "v9"}); err != nil {
		t.Fatalf("Delete (with etag) error: %v", err)
	}
	if delCalled != 1 || delETagCalled != 1 {
		t.Fatalf("DeleteStateWithETag path mismatch")
	}
	if observedEtag != "v9" {
		t.Fatalf("etag mismatch: got %q want v9", observedEtag)
	}
}

// buildMeta が境界条件で正しい metadata を生成することを検証する。
func TestBuildMeta(t *testing.T) {
	tests := []struct {
		name   string
		tenant string
		ttl    int32
		want   map[string]string
	}{
		{name: "empty returns nil", tenant: "", ttl: 0, want: nil},
		{name: "tenant only", tenant: "T", ttl: 0, want: map[string]string{"tenantId": "T"}},
		{name: "ttl only", tenant: "", ttl: 30, want: map[string]string{"ttlInSeconds": "30"}},
		{name: "both", tenant: "T", ttl: 30, want: map[string]string{"tenantId": "T", "ttlInSeconds": "30"}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := buildMeta(tt.tenant, tt.ttl)
			if len(got) != len(tt.want) {
				t.Fatalf("len mismatch: got %d want %d", len(got), len(tt.want))
			}
			for k, v := range tt.want {
				if got[k] != v {
					t.Fatalf("key %q: got %q want %q", k, got[k], v)
				}
			}
		})
	}
}
