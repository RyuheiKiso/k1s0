// 本ファイルは TenantContext の参照ヘルパ。
// 5 ハンドラ全てで使い回す共通 utility。
//
// 注: docs 正典 では Policy Enforcer（internal/policy/enforcer.go）が gRPC interceptor で
//     JWT 検証 → ctx.Value(key.TenantID) に格納する設計（DS-SW-COMP-... ）。
//     本リリース時点 では interceptor 未実装のため、proto 内 context フィールドから
//     直接 tenant_id を取得する暫定実装としている。plan 04-02 で interceptor 化する。
//
// テナント境界（NFR-E-AC-003 / FR-T1-LOG-003）:
//   tenant_id が空のリクエストは「越境防止」を担保できないため InvalidArgument で
//   弾く。Policy Enforcer 導入後も二重防御として handler 側でも検証を残す。

package state

import (
	// gRPC code 定義（InvalidArgument 等）。
	"google.golang.org/grpc/codes"
	// gRPC status を返却する helper。
	"google.golang.org/grpc/status"

	// 共通型（TenantContext / ErrorDetail）。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
)

// tenantIDOf は proto の TenantContext から tenant_id を取り出す。
// 引数 nil（呼出側がコンテキストを省略）は空文字を返し、requireTenantID で弾く。
func tenantIDOf(ctx *commonv1.TenantContext) string {
	// nil ガード。
	if ctx == nil {
		// 空文字を返却する。
		return ""
	}
	// tenant_id フィールドを返却する。
	return ctx.GetTenantId()
}

// requireTenantID は tenant_id が空でないことを検証し、空なら InvalidArgument を返す。
// rpc は handler 名（"State.Get" など）でログ識別子に使う。NFR-E-AC-003 越境防止のため、
// すべての公開 handler の冒頭で呼ぶこと。
func requireTenantID(ctx *commonv1.TenantContext, rpc string) (string, error) {
	// proto から tenant_id を取り出す。
	tid := tenantIDOf(ctx)
	// 空なら越境防止違反として弾く。
	if tid == "" {
		// gRPC InvalidArgument で返却する。
		return "", status.Errorf(codes.InvalidArgument, "tier1/state: tenant_id required in TenantContext (%s)", rpc)
	}
	// 非空なら呼出側に渡す。
	return tid, nil
}
