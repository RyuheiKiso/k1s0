// 本ファイルは k1s0client の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - withTenantFromRequest: auth middleware が ctx に attach した tenant_id を
//     k1s0.WithTenant で SDK ctx に伝搬する（NFR-E-AC-003 違反防止の中核ロジック）。
//   - middleware 未経由 ctx は素通り（cfg.TenantID にフォールバック）。
//   - Close() は nil-safe。

package k1s0client

import (
	"context"
	"testing"

	"github.com/k1s0/k1s0/src/tier3/bff/internal/auth"
)

// authCtxWith は auth middleware が attach する 3 つの context value を直接 set した
// ctx を返す（middleware を起動せずに後段関数を単体テストするための shortcut）。
func authCtxWith(tenantID, subject, token string) context.Context {
	ctx := context.Background()
	ctx = context.WithValue(ctx, auth.TenantIDKey, tenantID)
	ctx = context.WithValue(ctx, auth.SubjectKey, subject)
	ctx = context.WithValue(ctx, auth.TokenKey, token)
	return ctx
}

func TestWithTenantFromRequest_PassesThroughWhenNoAuth(t *testing.T) {
	// auth middleware を経由していない素の ctx は変更されない。
	in := context.Background()
	out := withTenantFromRequest(in)
	// ポインタ等価ではなく、tenant_id が context value に attach されていないことを確認する。
	if got := auth.TenantIDFromContext(out); got != "" {
		t.Errorf("plain ctx should not have tenant_id, got %q", got)
	}
	// SDK の TenantID が cfg fallback に任される（test では SDK 経由検証は不要、out は in と等価動作）。
}

func TestWithTenantFromRequest_PropagatesAuthContext(t *testing.T) {
	in := authCtxWith("T-PROD", "alice", "jwt-token-xxx")
	out := withTenantFromRequest(in)
	// auth middleware の helper で取れる tenant_id / subject は変わらず維持される。
	if got := auth.TenantIDFromContext(out); got != "T-PROD" {
		t.Errorf("tenant_id should be preserved through, got %q", got)
	}
	if got := auth.SubjectFromContext(out); got != "alice" {
		t.Errorf("subject should be preserved through, got %q", got)
	}
	// ctx は wrap されているため、in とは異なる context object になる
	// （WithValue は context.valueCtx を新規生成する）。
	if in == out {
		t.Errorf("auth ctx should be wrapped, but ctx pointer is identical")
	}
}

func TestWithTenantFromRequest_EmptyTenantFallsBack(t *testing.T) {
	// auth middleware が attach した tenant_id が空文字なら wrap せず素通り。
	in := authCtxWith("", "noone", "anon-token")
	out := withTenantFromRequest(in)
	// fallback path: ctx は wrap されない（同 object）。
	if in != out {
		t.Errorf("empty tenant_id should not wrap ctx")
	}
}

func TestClose_NilSafe(t *testing.T) {
	// nil receiver / nil client いずれも panic しない。
	var c *Client
	if err := c.Close(); err != nil {
		t.Errorf("nil Client.Close should be no-op, got %v", err)
	}
	c2 := &Client{client: nil}
	if err := c2.Close(); err != nil {
		t.Errorf("Client with nil client.Close should be no-op, got %v", err)
	}
}
