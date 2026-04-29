// 本ファイルは tenant_override.go の単体テスト。
//
// 検証観点:
//   - WithTenant が ctx に override を attach する
//   - 同一 ctx に多段 attach すると最も内側が優先される（標準 context.WithValue 挙動）
//   - tenantOverrideFromContext は未 attach / TenantID 空文字を ok=false で返す
//   - tenantContext(ctx) は override を優先し、未 attach 時は cfg にフォールバックする

package k1s0

import (
	"context"
	"testing"
)

func TestWithTenant_AttachesOverride(t *testing.T) {
	ctx := WithTenant(context.Background(), "tenant-A", "alice")
	ov, ok := tenantOverrideFromContext(ctx)
	if !ok {
		t.Fatalf("override should be attached")
	}
	if ov.TenantID != "tenant-A" || ov.Subject != "alice" {
		t.Errorf("override = %+v", ov)
	}
}

func TestWithTenant_NestedOverride_InnerWins(t *testing.T) {
	ctx := WithTenant(context.Background(), "outer-tenant", "outer-user")
	ctx = WithTenant(ctx, "inner-tenant", "inner-user")
	ov, ok := tenantOverrideFromContext(ctx)
	if !ok || ov.TenantID != "inner-tenant" || ov.Subject != "inner-user" {
		t.Fatalf("inner should win: %+v ok=%v", ov, ok)
	}
}

func TestTenantOverrideFromContext_NotAttached_NotOK(t *testing.T) {
	if _, ok := tenantOverrideFromContext(context.Background()); ok {
		t.Errorf("should be not ok when not attached")
	}
}

func TestTenantOverrideFromContext_EmptyTenantID_NotOK(t *testing.T) {
	// TenantID 空文字 override は無効として扱う（cfg fallback させる）。
	ctx := WithTenantOverride(context.Background(), TenantOverride{Subject: "u"})
	if _, ok := tenantOverrideFromContext(ctx); ok {
		t.Errorf("empty tenant_id override should be ignored")
	}
}

func TestClient_TenantContext_OverrideWins(t *testing.T) {
	c := &Client{cfg: Config{TenantID: "cfg-tenant", Subject: "cfg-subject"}}
	// override 不在 → cfg fallback。
	tc := c.tenantContext(context.Background())
	if tc.GetTenantId() != "cfg-tenant" || tc.GetSubject() != "cfg-subject" {
		t.Errorf("fallback failed: %+v", tc)
	}
	// override あり → override 採用。
	ctx := WithTenant(context.Background(), "req-tenant", "req-subject")
	tc2 := c.tenantContext(ctx)
	if tc2.GetTenantId() != "req-tenant" || tc2.GetSubject() != "req-subject" {
		t.Errorf("override not applied: %+v", tc2)
	}
}
