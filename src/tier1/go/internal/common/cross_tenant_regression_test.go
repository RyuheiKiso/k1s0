// 本ファイルは G1〜G10 で潰した bug の regression を防ぐ unit test。
// CI で再発検知できるよう 1 ファイルに集約する。

package common

import (
	"context"
	"strings"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// G3 regression: cross-tenant boundary violation (NFR-E-AC-003)
// HTTP gateway 経由で AuthInterceptor が req=nil で skip しても、
// EnforceTenantBoundary が claims と body.tenant_id 不一致を PermissionDenied で reject。
func TestEnforceTenantBoundary_CrossTenantRejected(t *testing.T) {
	ctx := context.WithValue(context.Background(), authInfoKey{}, &AuthInfo{
		TenantID: "tenant-A",
		Subject:  "alice",
	})
	_, err := EnforceTenantBoundary(ctx, "tenant-B", "State.Get")
	if err == nil {
		t.Fatal("cross-tenant request should be rejected, got nil error")
	}
	if got := status.Code(err); got != codes.PermissionDenied {
		t.Fatalf("expected PermissionDenied, got %v", got)
	}
	if !strings.Contains(err.Error(), "cross-tenant") {
		t.Fatalf("error message should mention 'cross-tenant', got: %v", err)
	}
	if !strings.Contains(err.Error(), `jwt="tenant-A"`) ||
		!strings.Contains(err.Error(), `body="tenant-B"`) {
		t.Fatalf("error should expose jwt + body tenants for debugging, got: %v", err)
	}
}

// claims が一致するなら body の tenant_id を返す（happy path）。
func TestEnforceTenantBoundary_MatchingTenantOK(t *testing.T) {
	ctx := context.WithValue(context.Background(), authInfoKey{}, &AuthInfo{
		TenantID: "tenant-A",
	})
	tid, err := EnforceTenantBoundary(ctx, "tenant-A", "State.Get")
	if err != nil {
		t.Fatalf("matching tenant should succeed, got error: %v", err)
	}
	if tid != "tenant-A" {
		t.Fatalf("returned tenant_id mismatch: got %q want tenant-A", tid)
	}
}

// AuthInfo が context に無い場合（auth=off の dev）は body 由来をそのまま採用。
func TestEnforceTenantBoundary_AuthOffPassThrough(t *testing.T) {
	ctx := context.Background() // no AuthInfo
	tid, err := EnforceTenantBoundary(ctx, "any-tenant", "State.Get")
	if err != nil {
		t.Fatalf("auth=off path should accept body, got error: %v", err)
	}
	if tid != "any-tenant" {
		t.Fatalf("returned tenant_id mismatch: got %q", tid)
	}
}

// 空 body tenant_id は InvalidArgument で reject。
func TestEnforceTenantBoundary_EmptyBodyRejected(t *testing.T) {
	ctx := context.Background()
	_, err := EnforceTenantBoundary(ctx, "", "State.Get")
	if err == nil {
		t.Fatal("empty body tenant_id should be rejected")
	}
	if got := status.Code(err); got != codes.InvalidArgument {
		t.Fatalf("expected InvalidArgument, got %v", got)
	}
}

// AuthInfo が存在するが TenantID が空（malformed JWT 等）の場合は mismatch 検査
// を skip し body 由来を採用。これは spec 上の安全側挙動: claims に tenant_id が
// 無ければ「不一致判定不能」のため、body をそのまま信用するしか無い。
func TestEnforceTenantBoundary_EmptyClaimsTenantSkipsCheck(t *testing.T) {
	ctx := context.WithValue(context.Background(), authInfoKey{}, &AuthInfo{
		TenantID: "", // empty
		Subject:  "anyone",
	})
	tid, err := EnforceTenantBoundary(ctx, "tenant-X", "State.Get")
	if err != nil {
		t.Fatalf("empty claims tenant should not block, got: %v", err)
	}
	if tid != "tenant-X" {
		t.Fatalf("body tenant_id should be returned: got %q", tid)
	}
}
