// 本ファイルは inMemoryDynamic（FR-T1-SECRETS-002）の単体テスト。
//
// 検証観点:
//   - TTL clamp（0 → 3600 / 上限超 → 86400）
//   - engine / role / tenant_id 必須検証（errEmptyTenant 返却）
//   - lease ID の重複なし
//   - credential 値（username / password）の一意性

package openbao

import (
	"context"
	"testing"
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
