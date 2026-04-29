// 本ファイルは openbaoSecretsAdapter の単体テスト。
// OpenBao SDK の KVv2 を fake で差し替え、adapter のロジックを検証する。

package openbao

import (
	"context"
	"errors"
	"testing"

	bao "github.com/openbao/openbao/api/v2"
)

// fakeKVClient は kvClient interface の最小 fake 実装。
type fakeKVClient struct {
	getFn        func(ctx context.Context, path string) (*bao.KVSecret, error)
	getVersionFn func(ctx context.Context, path string, version int) (*bao.KVSecret, error)
	putFn        func(ctx context.Context, path string, data map[string]interface{}, opts ...bao.KVOption) (*bao.KVSecret, error)
}

func (f *fakeKVClient) Get(ctx context.Context, path string) (*bao.KVSecret, error) {
	return f.getFn(ctx, path)
}
func (f *fakeKVClient) GetVersion(ctx context.Context, path string, version int) (*bao.KVSecret, error) {
	return f.getVersionFn(ctx, path, version)
}
func (f *fakeKVClient) Put(ctx context.Context, path string, data map[string]interface{}, opts ...bao.KVOption) (*bao.KVSecret, error) {
	return f.putFn(ctx, path, data, opts...)
}

func newAdapterWithFake(t *testing.T, fake *fakeKVClient) SecretsAdapter {
	t.Helper()
	return NewSecretsAdapter(NewWithKVClient("test://noop", fake))
}

// Get の正常系: KVSecret から Values と Version が返ることを検証。
// L2 テナント分離: 物理 path は `<tenantID>/<name>` で wrap される。
// 本テストは raw name "db/master" を渡し、SDK には "tenant-A/db/master" が届くことを検証する。
func TestSecretsAdapter_Get_Latest(t *testing.T) {
	fake := &fakeKVClient{
		getFn: func(_ context.Context, path string) (*bao.KVSecret, error) {
			// L2 分離: 物理 path は "tenant-A/db/master"（raw "db/master" + prefix）。
			if path != "tenant-A/db/master" {
				t.Fatalf("physical path mismatch: got %q want %q", path, "tenant-A/db/master")
			}
			return &bao.KVSecret{
				Data: map[string]interface{}{
					"username": "k1s0app",
					"password": "s3cret",
					"port":     5432, // int → string 化される
				},
				VersionMetadata: &bao.KVVersionMetadata{Version: 7},
			}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Get(context.Background(), SecretGetRequest{
		// raw name を渡す（adapter 側で prefix wrap される）。
		Name:     "db/master",
		TenantID: "tenant-A",
	})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if resp.Values["username"] != "k1s0app" {
		t.Fatalf("username mismatch: %s", resp.Values["username"])
	}
	if resp.Values["password"] != "s3cret" {
		t.Fatalf("password mismatch: %s", resp.Values["password"])
	}
	if resp.Values["port"] != "5432" {
		t.Fatalf("port stringification mismatch: %s", resp.Values["port"])
	}
	if resp.Version != 7 {
		t.Fatalf("version mismatch: %d", resp.Version)
	}
}

// Get の特定バージョン: GetVersion が呼ばれることを検証。
func TestSecretsAdapter_Get_SpecificVersion(t *testing.T) {
	called := 0
	fake := &fakeKVClient{
		getVersionFn: func(_ context.Context, _ string, version int) (*bao.KVSecret, error) {
			called++
			if version != 3 {
				t.Fatalf("version mismatch: %d", version)
			}
			return &bao.KVSecret{
				Data:            map[string]interface{}{"key": "v3-value"},
				VersionMetadata: &bao.KVVersionMetadata{Version: 3},
			}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Get(context.Background(), SecretGetRequest{Name: "x", Version: 3})
	if err != nil {
		t.Fatalf("Get error: %v", err)
	}
	if called != 1 {
		t.Fatalf("GetVersion not called once: %d", called)
	}
	if resp.Values["key"] != "v3-value" {
		t.Fatalf("value mismatch: %s", resp.Values["key"])
	}
}

// Get の SDK エラー透過。
func TestSecretsAdapter_Get_SDKError(t *testing.T) {
	want := errors.New("403 forbidden")
	fake := &fakeKVClient{
		getFn: func(_ context.Context, _ string) (*bao.KVSecret, error) {
			return nil, want
		},
	}
	a := newAdapterWithFake(t, fake)
	_, err := a.Get(context.Background(), SecretGetRequest{Name: "x"})
	if !errors.Is(err, want) {
		t.Fatalf("error not transparent: %v", err)
	}
}

// Get で nil secret 返却時 ErrSecretNotFound に変換される。
func TestSecretsAdapter_Get_NilSecret_NotFound(t *testing.T) {
	fake := &fakeKVClient{
		getFn: func(_ context.Context, _ string) (*bao.KVSecret, error) {
			return nil, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	_, err := a.Get(context.Background(), SecretGetRequest{Name: "x"})
	if !errors.Is(err, ErrSecretNotFound) {
		t.Fatalf("expected ErrSecretNotFound, got %v", err)
	}
}

// BulkGet は name 毎に Get を呼ぶ。
// L2 テナント分離: 物理 path は `tenant-A/<name>` で wrap される。
// store のキーも prefix 込みで定義し、BulkGet が prefix 適用後に到達することを確認する。
func TestSecretsAdapter_BulkGet_Multiple(t *testing.T) {
	store := map[string]*bao.KVSecret{
		// L2 分離: store は物理 path で indexed。
		"tenant-A/db/master":   {Data: map[string]interface{}{"u": "u1"}, VersionMetadata: &bao.KVVersionMetadata{Version: 1}},
		"tenant-A/db/replica":  {Data: map[string]interface{}{"u": "u2"}, VersionMetadata: &bao.KVVersionMetadata{Version: 2}},
		"tenant-A/absent/path": nil, // NotFound として skip される
	}
	fake := &fakeKVClient{
		getFn: func(_ context.Context, path string) (*bao.KVSecret, error) {
			s := store[path]
			return s, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	// raw name を渡す。adapter が `tenant-A/` prefix を付けて store を引く。
	results, err := a.BulkGet(context.Background(), []string{"db/master", "db/replica", "absent/path"}, "tenant-A")
	if err != nil {
		t.Fatalf("BulkGet error: %v", err)
	}
	if len(results) != 2 {
		t.Fatalf("expected 2 results (NotFound skipped), got %d", len(results))
	}
	if results["db/master"].Values["u"] != "u1" {
		t.Fatalf("db/master mismatch")
	}
	if results["db/replica"].Version != 2 {
		t.Fatalf("db/replica version mismatch")
	}
}

// クロステナント越境テスト（OpenBao secrets）: 同一論理 name を異なるテナントが Get しても
// 互いの値が見えないことを確認する。secretPath による物理 path 分離（NFR-E-AC-003 / L2）の検証。
func TestSecretsAdapter_CrossTenant_Isolation(t *testing.T) {
	// store は物理 path で indexed する。異なる tenant が同じ raw 名 "db/password" を使う想定。
	store := map[string]*bao.KVSecret{
		"T-alpha/db/password": {Data: map[string]interface{}{"value": "secret-A"}, VersionMetadata: &bao.KVVersionMetadata{Version: 1}},
		"T-bravo/db/password": {Data: map[string]interface{}{"value": "secret-B"}, VersionMetadata: &bao.KVVersionMetadata{Version: 1}},
	}
	fake := &fakeKVClient{
		getFn: func(_ context.Context, path string) (*bao.KVSecret, error) {
			return store[path], nil
		},
	}
	a := newAdapterWithFake(t, fake)
	// tenant A は自テナントの値を取得できる。
	rA, err := a.Get(context.Background(), SecretGetRequest{Name: "db/password", TenantID: "T-alpha"})
	if err != nil {
		t.Fatalf("tenant A Get: %v", err)
	}
	if rA.Values["value"] != "secret-A" {
		t.Fatalf("tenant A leak: got %q", rA.Values["value"])
	}
	// tenant B は自テナントの値を取得できる（A の値ではない）。
	rB, err := a.Get(context.Background(), SecretGetRequest{Name: "db/password", TenantID: "T-bravo"})
	if err != nil {
		t.Fatalf("tenant B Get: %v", err)
	}
	if rB.Values["value"] != "secret-B" {
		t.Fatalf("tenant B leak: got %q", rB.Values["value"])
	}
}

// secretPath / stripSecretPath の境界条件。
func TestSecretPath_Cases(t *testing.T) {
	cases := []struct {
		name     string
		tenantID string
		input    string
		want     string
	}{
		{name: "normal", tenantID: "T", input: "db/master", want: "T/db/master"},
		{name: "empty tenant returns as-is", tenantID: "", input: "db/master", want: "db/master"},
		{name: "already prefixed not double-prefixed", tenantID: "T", input: "T/db/master", want: "T/db/master"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			if got := secretPath(c.tenantID, c.input); got != c.want {
				t.Fatalf("secretPath(%q, %q) = %q; want %q", c.tenantID, c.input, got, c.want)
			}
		})
	}
}

// Rotate: 現在値を読んで同じ値で Put し、新バージョンが返ることを検証。
func TestSecretsAdapter_Rotate_BumpsVersion(t *testing.T) {
	current := &bao.KVSecret{
		Data:            map[string]interface{}{"password": "old-secret"},
		VersionMetadata: &bao.KVVersionMetadata{Version: 5},
	}
	putCalled := 0
	fake := &fakeKVClient{
		getFn: func(_ context.Context, _ string) (*bao.KVSecret, error) {
			return current, nil
		},
		putFn: func(_ context.Context, _ string, data map[string]interface{}, _ ...bao.KVOption) (*bao.KVSecret, error) {
			putCalled++
			// put される data は元の data と同一であること。
			if data["password"] != "old-secret" {
				t.Fatalf("data mismatch on put: %v", data)
			}
			// バージョンを +1 して返す。
			return &bao.KVSecret{
				Data:            data,
				VersionMetadata: &bao.KVVersionMetadata{Version: 6},
			}, nil
		},
	}
	a := newAdapterWithFake(t, fake)
	resp, err := a.Rotate(context.Background(), SecretRotateRequest{Name: "db/master"})
	if err != nil {
		t.Fatalf("Rotate error: %v", err)
	}
	if putCalled != 1 {
		t.Fatalf("Put not called once: %d", putCalled)
	}
	if resp.Version != 6 {
		t.Fatalf("expected new version 6, got %d", resp.Version)
	}
}
