// 本ファイルは tenant_prefix.go の単体テスト。
//
// 検証観点:
//   - prefixKey / stripKey / prefixKeys / hasTenantPrefix の境界条件
//   - L2 テナント分離（NFR-E-AC-003）の adapter 層挙動が要件と一致すること:
//     物理キーは "<tenant_id>/<key>"、応答は strip 済み "<key>"。
//   - クロステナント越境テスト: テナント A の Set が テナント B の Get に観測されないこと
//     （inMemoryDapr backend を介して end-to-end で確認）。

package dapr

import (
	"context"
	"testing"
)

// prefixKey の境界条件: 通常 / 空テナント / 二重 prefix 抑制。
func TestPrefixKey_Cases(t *testing.T) {
	tests := []struct {
		name     string
		tenantID string
		key      string
		want     string
	}{
		{name: "normal", tenantID: "T", key: "foo", want: "T/foo"},
		{name: "empty tenant returns key as-is", tenantID: "", key: "foo", want: "foo"},
		{name: "already prefixed not double-prefixed", tenantID: "T", key: "T/foo", want: "T/foo"},
		{name: "key contains slash", tenantID: "T", key: "users/42", want: "T/users/42"},
		{name: "key empty", tenantID: "T", key: "", want: "T/"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := prefixKey(tt.tenantID, tt.key); got != tt.want {
				t.Fatalf("prefixKey(%q, %q) = %q; want %q", tt.tenantID, tt.key, got, tt.want)
			}
		})
	}
}

// stripKey の境界条件。
func TestStripKey_Cases(t *testing.T) {
	tests := []struct {
		name     string
		tenantID string
		key      string
		want     string
	}{
		{name: "strip prefix", tenantID: "T", key: "T/foo", want: "foo"},
		{name: "no prefix unchanged", tenantID: "T", key: "foo", want: "foo"},
		{name: "different tenant unchanged", tenantID: "A", key: "B/foo", want: "B/foo"},
		{name: "empty tenant returns as-is", tenantID: "", key: "T/foo", want: "T/foo"},
		{name: "key shorter than prefix", tenantID: "T", key: "x", want: "x"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := stripKey(tt.tenantID, tt.key); got != tt.want {
				t.Fatalf("stripKey(%q, %q) = %q; want %q", tt.tenantID, tt.key, got, tt.want)
			}
		})
	}
}

// prefixKeys は元スライスを破壊せずに新スライスを返す。
func TestPrefixKeys_NonDestructive(t *testing.T) {
	original := []string{"a", "b", "c"}
	out := prefixKeys("T", original)
	if len(out) != 3 || out[0] != "T/a" || out[1] != "T/b" || out[2] != "T/c" {
		t.Fatalf("prefixKeys mismatch: %v", out)
	}
	// 元スライスが破壊されていないこと。
	if original[0] != "a" || original[1] != "b" || original[2] != "c" {
		t.Fatalf("input slice was mutated: %v", original)
	}
}

// クロステナント越境テスト（State）: tenant A の Set は tenant B の Get で見えない。
// inMemoryDapr backend を介した end-to-end の挙動検証。
// docs/03_要件定義/30_非機能要件/E_セキュリティ.md NFR-E-AC-003 の最重要シナリオ。
func TestStateAdapter_CrossTenant_Isolation(t *testing.T) {
	// in-memory backend を Client にラップする（newInMemoryDapr は 5 building block 全実装を持つ）。
	mem := newInMemoryDapr()
	cli := NewWithStateClient("test://noop", mem)
	a := NewStateAdapter(cli)
	ctx := context.Background()

	// tenant A が "shared-key" に "secret-A" を保存する。
	if _, err := a.Set(ctx, StateSetRequest{
		Store: "store", Key: "shared-key", Data: []byte("secret-A"), TenantID: "A",
	}); err != nil {
		t.Fatalf("tenant A Set: %v", err)
	}
	// tenant B が "shared-key" に "secret-B" を保存する。
	if _, err := a.Set(ctx, StateSetRequest{
		Store: "store", Key: "shared-key", Data: []byte("secret-B"), TenantID: "B",
	}); err != nil {
		t.Fatalf("tenant B Set: %v", err)
	}

	// tenant A の Get は "secret-A" を返す。
	rA, err := a.Get(ctx, StateGetRequest{Store: "store", Key: "shared-key", TenantID: "A"})
	if err != nil {
		t.Fatalf("tenant A Get: %v", err)
	}
	if rA.NotFound || string(rA.Data) != "secret-A" {
		t.Fatalf("tenant A leak: got %q (notfound=%v)", rA.Data, rA.NotFound)
	}
	// tenant B の Get は "secret-B" を返す。
	rB, err := a.Get(ctx, StateGetRequest{Store: "store", Key: "shared-key", TenantID: "B"})
	if err != nil {
		t.Fatalf("tenant B Get: %v", err)
	}
	if rB.NotFound || string(rB.Data) != "secret-B" {
		t.Fatalf("tenant B leak: got %q (notfound=%v)", rB.Data, rB.NotFound)
	}
}

// クロステナント越境テスト（State BulkGet）: 応答キーは strip 済で他テナント prefix が漏れない。
func TestStateAdapter_BulkGet_StripsPrefix(t *testing.T) {
	mem := newInMemoryDapr()
	cli := NewWithStateClient("test://noop", mem)
	a := NewStateAdapter(cli)
	ctx := context.Background()

	// tenant A が 2 件保存する。
	for _, kv := range []struct{ k, v string }{{"a", "1"}, {"b", "2"}} {
		if _, err := a.Set(ctx, StateSetRequest{
			Store: "store", Key: kv.k, Data: []byte(kv.v), TenantID: "A",
		}); err != nil {
			t.Fatalf("Set %s: %v", kv.k, err)
		}
	}

	// tenant A が両方を BulkGet する。
	out, err := a.BulkGet(ctx, StateBulkGetRequest{
		Store: "store", Keys: []string{"a", "b"}, TenantID: "A",
	})
	if err != nil {
		t.Fatalf("BulkGet: %v", err)
	}
	// 応答は 2 件、Key は strip 済（"A/a" ではなく "a"）。
	if len(out) != 2 {
		t.Fatalf("len: %d", len(out))
	}
	for _, item := range out {
		if item.Key != "a" && item.Key != "b" {
			t.Fatalf("response leaked physical prefix: %q", item.Key)
		}
	}
}
