// 本ファイルは inMemoryDynamic / productionDynamic（FR-T1-SECRETS-002）の単体テスト。
//
// 検証観点:
//   - in-memory: TTL clamp / 必須検証 / lease ID 一意
//   - production: path 構築規則 / SDK Secret → 応答変換 / nil-Secret で NotFound

package openbao

import (
	"context"
	"errors"
	"testing"

	bao "github.com/openbao/openbao/api/v2"
)

// 既定 TTL の clamp 動作を検証する。
func TestInMemoryDynamic_TTLDefaultAndClamp(t *testing.T) {
	// テストケース表。
	cases := []struct {
		name     string
		input    int32
		expected int32
	}{
		// 0 は default 3600 に置換される。
		{"zero -> default 3600", 0, 3600},
		// 負値も default に置換される。
		{"negative -> default 3600", -1, 3600},
		// 範囲内はそのまま。
		{"in-range 1800", 1800, 1800},
		// 上限超過は 86400 にクランプ。
		{"over-max -> 86400", 999999, 86400},
		// 上限ちょうどは そのまま。
		{"max exact", 86400, 86400},
	}
	// 同一 backend で繰り返す（lease ID が連番増えても TTL は独立）。
	d := NewInMemoryDynamic()
	// 各ケースを実行する。
	for _, c := range cases {
		// サブテストとして実行する。
		t.Run(c.name, func(t *testing.T) {
			// 動的 secret を発行する。
			resp, err := d.GetDynamic(context.Background(), DynamicSecretRequest{
				Engine:     "postgres",
				Role:       "app-rw",
				TenantID:   "t1",
				TTLSeconds: c.input,
			})
			// エラーが出てはいけない。
			if err != nil {
				t.Fatalf("GetDynamic err: %v", err)
			}
			// 期待 TTL と一致するか確認する。
			if resp.TTLSeconds != c.expected {
				t.Errorf("ttl: got %d, want %d", resp.TTLSeconds, c.expected)
			}
		})
	}
}

// engine / role / tenant_id 必須の検証。
func TestInMemoryDynamic_RequiredFields(t *testing.T) {
	// 各検証ケース。
	cases := []struct {
		name string
		req  DynamicSecretRequest
	}{
		// engine 空。
		{"empty engine", DynamicSecretRequest{Role: "app-rw", TenantID: "t1"}},
		// role 空。
		{"empty role", DynamicSecretRequest{Engine: "postgres", TenantID: "t1"}},
		// tenant_id 空。
		{"empty tenant", DynamicSecretRequest{Engine: "postgres", Role: "app-rw"}},
	}
	// adapter を 1 つ用意する。
	d := NewInMemoryDynamic()
	// 各ケースで err 必須を確認する。
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			_, err := d.GetDynamic(context.Background(), c.req)
			if err == nil {
				t.Fatal("expected error, got nil")
			}
		})
	}
}

// lease ID と credential が複数回呼出で一意であることを確認する。
func TestInMemoryDynamic_UniqueLeaseAndCreds(t *testing.T) {
	d := NewInMemoryDynamic()
	// 3 回発行する。
	a, _ := d.GetDynamic(context.Background(), DynamicSecretRequest{Engine: "postgres", Role: "rw", TenantID: "t"})
	b, _ := d.GetDynamic(context.Background(), DynamicSecretRequest{Engine: "postgres", Role: "rw", TenantID: "t"})
	c, _ := d.GetDynamic(context.Background(), DynamicSecretRequest{Engine: "postgres", Role: "rw", TenantID: "t"})
	// lease ID は一意。
	if a.LeaseID == b.LeaseID || a.LeaseID == c.LeaseID || b.LeaseID == c.LeaseID {
		t.Errorf("lease ID not unique: %q %q %q", a.LeaseID, b.LeaseID, c.LeaseID)
	}
	// password も一意（crypto/rand 由来なので 16 byte で衝突確率は無視できる）。
	if a.Values["password"] == b.Values["password"] {
		t.Errorf("password collision: %q == %q", a.Values["password"], b.Values["password"])
	}
	// 発行時刻は単調非減少。
	if a.IssuedAtMs > b.IssuedAtMs || b.IssuedAtMs > c.IssuedAtMs {
		t.Errorf("issued_at not monotonic: %d %d %d", a.IssuedAtMs, b.IssuedAtMs, c.IssuedAtMs)
	}
}

// fakeReader は productionDynamic のテストで dynamicReader interface を満たす。
type fakeReader struct {
	// 直前に呼ばれた path（path 構築規則の検証用）。
	lastPath string
	// 返却 Secret（nil で NotFound 検証）。
	resp *bao.Secret
	// 返却 err。
	err error
}

func (f *fakeReader) ReadWithContext(_ context.Context, path string) (*bao.Secret, error) {
	f.lastPath = path
	return f.resp, f.err
}

// production 経路で path が "<engine>/creds/<tenant>/<role>" 形式で組まれることを検証する。
func TestProductionDynamic_PathConvention(t *testing.T) {
	fake := &fakeReader{
		resp: &bao.Secret{
			LeaseID:       "postgres/creds/t1/app-rw/lease-abc",
			LeaseDuration: 1800,
			Data: map[string]interface{}{
				"username": "v-token-app-rw-XXXX",
				"password": "secretpw",
			},
		},
	}
	a := NewProductionDynamicFromReader(fake)
	resp, err := a.GetDynamic(context.Background(), DynamicSecretRequest{
		Engine:     "postgres",
		Role:       "app-rw",
		TenantID:   "t1",
		TTLSeconds: 3600,
	})
	if err != nil {
		t.Fatalf("GetDynamic err: %v", err)
	}
	if fake.lastPath != "postgres/creds/t1/app-rw" {
		t.Errorf("path: got %q, want %q", fake.lastPath, "postgres/creds/t1/app-rw")
	}
	if resp.LeaseID != "postgres/creds/t1/app-rw/lease-abc" {
		t.Errorf("lease_id: got %q", resp.LeaseID)
	}
	if resp.Values["username"] != "v-token-app-rw-XXXX" {
		t.Errorf("username: got %q", resp.Values["username"])
	}
	if resp.TTLSeconds != 1800 {
		t.Errorf("ttl: got %d, want 1800 (from LeaseDuration)", resp.TTLSeconds)
	}
}

// nil の Secret（role 不在 / policy 不足）を ErrSecretNotFound に翻訳することを検証する。
func TestProductionDynamic_NilSecretReturnsNotFound(t *testing.T) {
	fake := &fakeReader{resp: nil, err: nil}
	a := NewProductionDynamicFromReader(fake)
	_, err := a.GetDynamic(context.Background(), DynamicSecretRequest{
		Engine:   "postgres",
		Role:     "missing",
		TenantID: "t1",
	})
	if !errors.Is(err, ErrSecretNotFound) {
		t.Errorf("err: got %v, want ErrSecretNotFound", err)
	}
}

// reader 未注入は ErrNotWired を返すことを検証する。
func TestProductionDynamic_NoReaderReturnsNotWired(t *testing.T) {
	a := NewProductionDynamicFromReader(nil)
	_, err := a.GetDynamic(context.Background(), DynamicSecretRequest{
		Engine:   "postgres",
		Role:     "rw",
		TenantID: "t",
	})
	if !errors.Is(err, ErrNotWired) {
		t.Errorf("err: got %v, want ErrNotWired", err)
	}
}

// 必須フィールド検証は in-memory と同じセマンティクスにする。
func TestProductionDynamic_RequiredFields(t *testing.T) {
	fake := &fakeReader{}
	a := NewProductionDynamicFromReader(fake)
	cases := []DynamicSecretRequest{
		{Role: "r", TenantID: "t"},                      // engine 空
		{Engine: "postgres", TenantID: "t"},             // role 空
		{Engine: "postgres", Role: "r"},                 // tenant 空
	}
	for _, c := range cases {
		if _, err := a.GetDynamic(context.Background(), c); err == nil {
			t.Errorf("expected error for %+v, got nil", c)
		}
	}
}
