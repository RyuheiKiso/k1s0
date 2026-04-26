// 本ファイルは TenantContext の参照ヘルパ。
// 5 ハンドラ全てで使い回す共通 utility。
//
// 注: docs 正典 では Policy Enforcer（internal/policy/enforcer.go）が gRPC interceptor で
//     JWT 検証 → ctx.Value(key.TenantID) に格納する設計（DS-SW-COMP-... ）。
//     本リリース時点 では interceptor 未実装のため、proto 内 context フィールドから
//     直接 tenant_id を取得する暫定実装としている。plan 04-02 で interceptor 化する。

package state

// SDK 生成 stub の共通型。
import (
	// 共通型（TenantContext / ErrorDetail）。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
)

// tenantIDOf は proto の TenantContext から tenant_id を取り出す。
// 引数 nil（呼出側がコンテキストを省略）は空文字を返し、Policy Enforcer で却下される設計。
func tenantIDOf(ctx *commonv1.TenantContext) string {
	// nil ガード。
	if ctx == nil {
		// 空文字を返却する。
		return ""
	}
	// tenant_id フィールドを返却する。
	return ctx.GetTenantId()
}
