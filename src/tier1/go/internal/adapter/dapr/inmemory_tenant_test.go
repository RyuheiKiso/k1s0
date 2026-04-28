// 本ファイルは in-memory Dapr backend のテナント越境防止（NFR-E-AC-003）動作確認テスト。
//
// 検証ポイント:
//   1. tenant-A が書込んだ key を tenant-B から Get しても未存在として返ること
//   2. 同一 tenant 内では key が見えること
//   3. BulkGet も tenant 境界を越えないこと
//   4. Delete が他 tenant のデータに影響しないこと
//   5. ExecuteStateTransaction が tenant 境界を越えないこと
//   6. Configuration KV の seed が tenant=""（global）に入り、Get で見えること
//
// 設計正典:
//   docs/03_要件定義/30_非機能要件/E_セキュリティ.md（NFR-E-AC-003）
//   docs/04_概要設計/.../02_Daprファサード層コンポーネント.md（DS-SW-COMP-020）

package dapr

import (
	"context"
	"testing"

	daprclient "github.com/dapr/go-sdk/client"
)

// helper: tenantId を含む metadata map を作る。
func tenantMeta(tenant string) map[string]string {
	// "tenantId" キーで tenant を運ぶ。
	return map[string]string{metadataKeyTenant: tenant}
}

// 1. tenant-A 書込 → tenant-B から見ると未存在。
func TestInMemoryDapr_State_TenantIsolation_Get(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// tenant-A で k1 に v1 を書く。
	if err := m.SaveState(context.Background(), "valkey", "k1", []byte("v1"), tenantMeta("tenant-A")); err != nil {
		// 書込失敗は backend バグ。
		t.Fatalf("SaveState(A) err: %v", err)
	}
	// tenant-A で読むと取得できる。
	itemA, err := m.GetState(context.Background(), "valkey", "k1", tenantMeta("tenant-A"))
	// err は nil 期待。
	if err != nil {
		t.Fatalf("GetState(A) err: %v", err)
	}
	// Value は "v1" 期待。
	if got := string(itemA.Value); got != "v1" {
		t.Fatalf("GetState(A) value: want %q got %q", "v1", got)
	}
	// tenant-B で読むと未存在（Value=nil）期待。
	itemB, err := m.GetState(context.Background(), "valkey", "k1", tenantMeta("tenant-B"))
	// err は nil 期待。
	if err != nil {
		t.Fatalf("GetState(B) err: %v", err)
	}
	// Value は nil（未存在）期待。
	if got := itemB.Value; got != nil {
		t.Fatalf("NFR-E-AC-003 violation: tenant-B saw tenant-A data: %q", got)
	}
}

// 2. 同一 tenant の異なる store はそれぞれ独立。
func TestInMemoryDapr_State_StoreIsolation_WithinTenant(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// tenant-A の "valkey" store に k1=v1 を書く。
	if err := m.SaveState(context.Background(), "valkey", "k1", []byte("v1"), tenantMeta("tenant-A")); err != nil {
		t.Fatalf("SaveState(valkey) err: %v", err)
	}
	// tenant-A の "postgres" store の同 key は未存在期待。
	item, err := m.GetState(context.Background(), "postgres", "k1", tenantMeta("tenant-A"))
	// err は nil 期待。
	if err != nil {
		t.Fatalf("GetState(postgres) err: %v", err)
	}
	// Value は nil 期待（store 越境はしない）。
	if got := item.Value; got != nil {
		t.Fatalf("store isolation violation: postgres saw valkey data: %q", got)
	}
}

// 3. BulkGet も tenant 境界を越えない。
func TestInMemoryDapr_State_TenantIsolation_BulkGet(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// tenant-A に 3 key 書く。
	for _, kv := range []struct{ k, v string }{{"k1", "v1"}, {"k2", "v2"}, {"k3", "v3"}} {
		// Save 失敗は backend バグ。
		if err := m.SaveState(context.Background(), "valkey", kv.k, []byte(kv.v), tenantMeta("tenant-A")); err != nil {
			t.Fatalf("SaveState(A,%s): %v", kv.k, err)
		}
	}
	// tenant-B から BulkGet すると全件 nil Value 期待。
	items, err := m.GetBulkState(context.Background(), "valkey", []string{"k1", "k2", "k3"}, tenantMeta("tenant-B"), 0)
	// err は nil 期待。
	if err != nil {
		t.Fatalf("GetBulkState(B): %v", err)
	}
	// 件数は 3 件。
	if len(items) != 3 {
		t.Fatalf("BulkGet len: want 3 got %d", len(items))
	}
	// 全件 Value=nil 期待。
	for _, it := range items {
		// Value 非 nil は tenant 越境バグ。
		if it.Value != nil {
			t.Fatalf("NFR-E-AC-003 violation: tenant-B saw tenant-A %s=%q", it.Key, it.Value)
		}
	}
}

// 4. Delete は他 tenant のデータに影響しない。
func TestInMemoryDapr_State_TenantIsolation_Delete(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// tenant-A と tenant-B に同一 key を書く。
	for _, tenant := range []string{"tenant-A", "tenant-B"} {
		// Save 失敗は backend バグ。
		if err := m.SaveState(context.Background(), "valkey", "k1", []byte("v-"+tenant), tenantMeta(tenant)); err != nil {
			t.Fatalf("SaveState(%s): %v", tenant, err)
		}
	}
	// tenant-A の k1 を削除する。
	if err := m.DeleteState(context.Background(), "valkey", "k1", tenantMeta("tenant-A")); err != nil {
		// Delete 失敗は backend バグ。
		t.Fatalf("DeleteState(A): %v", err)
	}
	// tenant-A の k1 は未存在期待。
	itemA, err := m.GetState(context.Background(), "valkey", "k1", tenantMeta("tenant-A"))
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetState(A): %v", err)
	}
	// Value=nil 期待。
	if got := itemA.Value; got != nil {
		t.Fatalf("Delete(A) failed: A still sees %q", got)
	}
	// tenant-B の k1 は依然として存在期待。
	itemB, err := m.GetState(context.Background(), "valkey", "k1", tenantMeta("tenant-B"))
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetState(B): %v", err)
	}
	// Value は "v-tenant-B" 期待（A の Delete に巻き込まれない）。
	if got := string(itemB.Value); got != "v-tenant-B" {
		t.Fatalf("tenant-B Delete contamination: want %q got %q", "v-tenant-B", got)
	}
}

// 5. ExecuteStateTransaction も tenant 境界を越えない。
func TestInMemoryDapr_State_TenantIsolation_Transaction(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// tenant-A に Upsert+Delete を 1 トランザクションで実行する。
	ops := []*daprclient.StateOperation{
		{Type: daprclient.StateOperationTypeUpsert, Item: &daprclient.SetStateItem{Key: "k1", Value: []byte("v1")}},
		{Type: daprclient.StateOperationTypeUpsert, Item: &daprclient.SetStateItem{Key: "k2", Value: []byte("v2")}},
	}
	// Transact 失敗は backend バグ。
	if err := m.ExecuteStateTransaction(context.Background(), "valkey", tenantMeta("tenant-A"), ops); err != nil {
		t.Fatalf("Transact(A): %v", err)
	}
	// tenant-B から見ると k1 / k2 ともに未存在期待。
	for _, k := range []string{"k1", "k2"} {
		// 各 key を Get する。
		item, err := m.GetState(context.Background(), "valkey", k, tenantMeta("tenant-B"))
		// err nil 期待。
		if err != nil {
			t.Fatalf("GetState(B,%s): %v", k, err)
		}
		// Value=nil 期待。
		if got := item.Value; got != nil {
			t.Fatalf("Transaction tenant leak: tenant-B saw %s=%q", k, got)
		}
	}
}

// 6. Configuration KV の seed → Get の往復確認（global namespace のみ、テナント分離は production の Component 責務）。
func TestInMemoryDapr_Configuration_GlobalNamespace_Roundtrip(t *testing.T) {
	// 空 backend を作る。
	m := newInMemoryDapr()
	// global namespace に flag を 1 件 seed する。
	m.PutConfigurationItem("flagd-default", "feature.x", &daprclient.ConfigurationItem{Value: "true"})
	// 同一 store/key で Get すると seed した値が返る。
	item, err := m.GetConfigurationItem(context.Background(), "flagd-default", "feature.x")
	// err nil 期待。
	if err != nil {
		t.Fatalf("GetConfigurationItem: %v", err)
	}
	// item nil なら seed 配線バグ。
	if item == nil {
		t.Fatalf("seeded configuration item not retrievable")
	}
	// Value は "true" 期待。
	if got := item.Value; got != "true" {
		t.Fatalf("config Value: want %q got %q", "true", got)
	}
}
