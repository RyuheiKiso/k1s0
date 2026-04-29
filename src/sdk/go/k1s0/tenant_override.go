// 本ファイルは k1s0 Go SDK の per-request テナント上書きヘルパ。
//
// 設計動機:
//   k1s0.Client は New(cfg) 時に cfg.TenantID / cfg.Subject を保持するが、tier3 BFF
//   のように 1 SDK インスタンスで複数エンドユーザのリクエストを処理する経路では、
//   呼出ごとに JWT 由来の tenant_id を伝搬したい。本パッケージは context.Value で
//   per-request override を持ち回し、tenantContext(ctx) 系メソッドが override を
//   優先して使うことで、SDK 利用者が cfg を毎回再構築する必要をなくす。
//
// 利用例（BFF）:
//   ctx := k1s0.WithTenant(r.Context(), tenantID, subject)
//   resp, err := client.State().Get(ctx, store, key)

package k1s0

import "context"

// tenantOverrideKey は context.Value の key 衝突を避ける private 型。
type tenantOverrideKey struct{}

// TenantOverride は per-request の tenant_id / subject を持つ。
type TenantOverride struct {
	// テナント識別子。空文字なら override なし扱い。
	TenantID string
	// 行為主体（JWT sub / SPIFFE ID）。
	Subject string
	// 相関 ID（OTel traceparent などから引いた W3C trace_id）。空文字は override なし。
	CorrelationID string
}

// WithTenant は ctx に TenantOverride を attach した子 ctx を返す。
// 後続の SDK 呼出は override を tenant_context に詰めて tier1 へ送る。
//
// ctx には複数 override を多段で attach 可能で、最も内側（最後に attach された）が優先される。
func WithTenant(ctx context.Context, tenantID, subject string) context.Context {
	return context.WithValue(ctx, tenantOverrideKey{}, TenantOverride{
		TenantID: tenantID,
		Subject:  subject,
	})
}

// WithTenantOverride は完全な TenantOverride 構造体を ctx に attach する。
// CorrelationID も渡したい場合に使う。
func WithTenantOverride(ctx context.Context, ov TenantOverride) context.Context {
	return context.WithValue(ctx, tenantOverrideKey{}, ov)
}

// tenantOverrideFromContext は ctx から override を取り出す。未 attach なら ok=false。
func tenantOverrideFromContext(ctx context.Context) (TenantOverride, bool) {
	if ctx == nil {
		return TenantOverride{}, false
	}
	v, ok := ctx.Value(tenantOverrideKey{}).(TenantOverride)
	if !ok || v.TenantID == "" {
		return TenantOverride{}, false
	}
	return v, true
}
