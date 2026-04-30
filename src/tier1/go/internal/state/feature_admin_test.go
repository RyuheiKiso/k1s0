// 本ファイルは FeatureAdminService 実装の単体テスト。
// RegisterFlag / GetFlag / ListFlags の正常系 + validation 経路を検証する。

package state

import (
	"context"
	"testing"

	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/structpb"
)

// makeStringValue は variants 用の structpb.Value（文字列）を生成する helper。
func makeStringValue(t *testing.T, s string) *structpb.Value {
	t.Helper()
	v, err := structpb.NewValue(s)
	if err != nil {
		t.Fatalf("structpb.NewValue: %v", err)
	}
	return v
}

// makeValidFlagDef は最小要件を満たす有効な FlagDefinition を返す helper。
func makeValidFlagDef(t *testing.T, key string, kind featurev1.FlagKind) *featurev1.FlagDefinition {
	t.Helper()
	return &featurev1.FlagDefinition{
		FlagKey:        key,
		Kind:           kind,
		ValueType:      featurev1.FlagValueType_FLAG_VALUE_STRING,
		DefaultVariant: "default",
		Variants: map[string]*structpb.Value{
			"default": makeStringValue(t, "off"),
			"on":      makeStringValue(t, "on"),
		},
		State:       featurev1.FlagState_FLAG_STATE_ENABLED,
		Description: "test flag",
	}
}

// 正常系: RegisterFlag → GetFlag round-trip で同 flag が読み戻せる。
func TestFeatureAdmin_RegisterAndGet(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	tenant := "T-feat"
	flag := makeValidFlagDef(t, "T-feat.users.beta", featurev1.FlagKind_RELEASE)

	regResp, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{
		Flag:    flag,
		Context: makeTenantCtx(tenant),
	})
	if err != nil {
		t.Fatalf("RegisterFlag: %v", err)
	}
	if regResp.GetVersion() != 1 {
		t.Fatalf("first version: got %d, want 1", regResp.GetVersion())
	}

	getResp, err := srv.GetFlag(ctx, &featurev1.GetFlagRequest{
		FlagKey: flag.GetFlagKey(),
		Context: makeTenantCtx(tenant),
	})
	if err != nil {
		t.Fatalf("GetFlag: %v", err)
	}
	if getResp.GetFlag().GetFlagKey() != flag.GetFlagKey() {
		t.Errorf("flag_key mismatch: got %q want %q", getResp.GetFlag().GetFlagKey(), flag.GetFlagKey())
	}
	if getResp.GetVersion() != 1 {
		t.Errorf("version mismatch: got %d want 1", getResp.GetVersion())
	}
}

// 同 flag_key を 2 回登録すると version が 1 → 2 と増分される。
func TestFeatureAdmin_VersionIncrements(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	tenant := "T-ver"
	first := makeValidFlagDef(t, "T-ver.svc.feat", featurev1.FlagKind_RELEASE)
	second := makeValidFlagDef(t, "T-ver.svc.feat", featurev1.FlagKind_RELEASE)
	second.Description = "v2"

	r1, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: first, Context: makeTenantCtx(tenant)})
	if err != nil {
		t.Fatalf("first Register: %v", err)
	}
	r2, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: second, Context: makeTenantCtx(tenant)})
	if err != nil {
		t.Fatalf("second Register: %v", err)
	}
	if r1.GetVersion() != 1 || r2.GetVersion() != 2 {
		t.Fatalf("versions: got (%d,%d) want (1,2)", r1.GetVersion(), r2.GetVersion())
	}

	// 古い version も明示指定で取り出せる。
	v1Req := int64(1)
	gv1, err := srv.GetFlag(ctx, &featurev1.GetFlagRequest{FlagKey: "T-ver.svc.feat", Version: &v1Req, Context: makeTenantCtx(tenant)})
	if err != nil {
		t.Fatalf("GetFlag v1: %v", err)
	}
	if gv1.GetFlag().GetDescription() != "test flag" {
		t.Errorf("v1 description: got %q want %q", gv1.GetFlag().GetDescription(), "test flag")
	}

	// 最新（version=0）は v2 を返す。
	gLatest, err := srv.GetFlag(ctx, &featurev1.GetFlagRequest{FlagKey: "T-ver.svc.feat", Context: makeTenantCtx(tenant)})
	if err != nil {
		t.Fatalf("GetFlag latest: %v", err)
	}
	if gLatest.GetFlag().GetDescription() != "v2" {
		t.Errorf("latest description: got %q want %q", gLatest.GetFlag().GetDescription(), "v2")
	}
}

// PERMISSION 種別は approval_id 必須（FailedPrecondition）。
func TestFeatureAdmin_PermissionRequiresApproval(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	tenant := "T-perm"
	flag := makeValidFlagDef(t, "T-perm.gate.admin", featurev1.FlagKind_PERMISSION)

	_, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{
		Flag:    flag,
		Context: makeTenantCtx(tenant),
	})
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	if status.Code(err) != codes.FailedPrecondition {
		t.Errorf("code: got %s want FailedPrecondition", status.Code(err))
	}

	// approval_id 付きなら通る。
	_, err = srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{
		Flag:       flag,
		ApprovalId: "PC-2026-Q2-001",
		Context:    makeTenantCtx(tenant),
	})
	if err != nil {
		t.Fatalf("RegisterFlag with approval_id: %v", err)
	}
}

// flag_key 命名規則違反 / state 未指定 / default_variant 不在 / value_type 未指定 は InvalidArgument。
func TestFeatureAdmin_Validation(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	tenant := "T-val"
	good := makeValidFlagDef(t, "T-val.svc.feat", featurev1.FlagKind_RELEASE)

	cases := []struct {
		name string
		mut  func(*featurev1.FlagDefinition)
	}{
		{"bad flag_key", func(d *featurev1.FlagDefinition) { d.FlagKey = "no-dot-key" }},
		{"empty flag_key", func(d *featurev1.FlagDefinition) { d.FlagKey = "" }},
		{"value_type unspecified", func(d *featurev1.FlagDefinition) { d.ValueType = featurev1.FlagValueType_FLAG_VALUE_UNSPECIFIED }},
		{"state unspecified", func(d *featurev1.FlagDefinition) { d.State = featurev1.FlagState_FLAG_STATE_UNSPECIFIED }},
		{"default_variant absent", func(d *featurev1.FlagDefinition) { d.DefaultVariant = "ghost" }},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			d := makeValidFlagDef(t, good.GetFlagKey(), featurev1.FlagKind_RELEASE)
			tc.mut(d)
			_, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: d, Context: makeTenantCtx(tenant)})
			if err == nil {
				t.Fatal("expected error")
			}
			if status.Code(err) != codes.InvalidArgument {
				t.Errorf("code: got %s want InvalidArgument", status.Code(err))
			}
		})
	}
}

// tenant_id 不在は InvalidArgument。
func TestFeatureAdmin_TenantRequired(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	flag := makeValidFlagDef(t, "T.svc.f", featurev1.FlagKind_RELEASE)

	for _, name := range []string{"Register", "Get", "List"} {
		t.Run(name, func(t *testing.T) {
			var err error
			switch name {
			case "Register":
				_, err = srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: flag})
			case "Get":
				_, err = srv.GetFlag(ctx, &featurev1.GetFlagRequest{FlagKey: "T.svc.f"})
			case "List":
				_, err = srv.ListFlags(ctx, &featurev1.ListFlagsRequest{})
			}
			if err == nil {
				t.Fatalf("%s: expected error", name)
			}
			if status.Code(err) != codes.InvalidArgument {
				t.Errorf("%s: code: got %s want InvalidArgument", name, status.Code(err))
			}
		})
	}
}

// 異テナント間で flag は不可視（テナント越境防止）。
func TestFeatureAdmin_TenantIsolation(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	flag := makeValidFlagDef(t, "T1.svc.f", featurev1.FlagKind_RELEASE)
	if _, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: flag, Context: makeTenantCtx("T1")}); err != nil {
		t.Fatalf("Register T1: %v", err)
	}
	// 別テナント T2 から Get → NotFound
	_, err := srv.GetFlag(ctx, &featurev1.GetFlagRequest{FlagKey: "T1.svc.f", Context: makeTenantCtx("T2")})
	if err == nil {
		t.Fatal("expected NotFound")
	}
	if status.Code(err) != codes.NotFound {
		t.Errorf("code: got %s want NotFound", status.Code(err))
	}
	// T1 自身からは取れる
	if _, err := srv.GetFlag(ctx, &featurev1.GetFlagRequest{FlagKey: "T1.svc.f", Context: makeTenantCtx("T1")}); err != nil {
		t.Fatalf("Get T1 own: %v", err)
	}
}

// ListFlags の kind / state フィルタが効く。
func TestFeatureAdmin_ListFilters(t *testing.T) {
	srv := NewFeatureAdminServiceServer(NewFlagRegistry())
	ctx := context.Background()
	tenant := "T-list"
	tc := makeTenantCtx(tenant)

	regs := []*featurev1.FlagDefinition{
		makeValidFlagDef(t, "T-list.s.a", featurev1.FlagKind_RELEASE),
		makeValidFlagDef(t, "T-list.s.b", featurev1.FlagKind_EXPERIMENT),
	}
	disabled := makeValidFlagDef(t, "T-list.s.c", featurev1.FlagKind_RELEASE)
	disabled.State = featurev1.FlagState_FLAG_STATE_DISABLED
	regs = append(regs, disabled)

	for _, r := range regs {
		if _, err := srv.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{Flag: r, Context: tc}); err != nil {
			t.Fatalf("Register %s: %v", r.FlagKey, err)
		}
	}

	// 既定（state=ENABLED）で 2 件（RELEASE/a + EXPERIMENT/b）。
	resp, err := srv.ListFlags(ctx, &featurev1.ListFlagsRequest{Context: tc})
	if err != nil {
		t.Fatalf("List: %v", err)
	}
	if len(resp.GetFlags()) != 2 {
		t.Errorf("default list: got %d flags want 2", len(resp.GetFlags()))
	}

	// kind=RELEASE で 1 件（a のみ。c は DISABLED）。
	releaseKind := featurev1.FlagKind_RELEASE
	resp2, err := srv.ListFlags(ctx, &featurev1.ListFlagsRequest{Kind: &releaseKind, Context: tc})
	if err != nil {
		t.Fatalf("List kind=RELEASE: %v", err)
	}
	if len(resp2.GetFlags()) != 1 || resp2.GetFlags()[0].GetFlagKey() != "T-list.s.a" {
		t.Errorf("kind=RELEASE: got %v", resp2.GetFlags())
	}

	// state=DISABLED で c のみ。
	disabledState := featurev1.FlagState_FLAG_STATE_DISABLED
	resp3, err := srv.ListFlags(ctx, &featurev1.ListFlagsRequest{State: &disabledState, Context: tc})
	if err != nil {
		t.Fatalf("List state=DISABLED: %v", err)
	}
	if len(resp3.GetFlags()) != 1 || resp3.GetFlags()[0].GetFlagKey() != "T-list.s.c" {
		t.Errorf("state=DISABLED: got %v", resp3.GetFlags())
	}
}
