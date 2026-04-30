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
	// AuthInfo を context から取り出すための context.Context 引数を取る。
	"context"

	// gRPC code 定義（InvalidArgument 等）。
	"google.golang.org/grpc/codes"
	// gRPC status を返却する helper。
	"google.golang.org/grpc/status"

	// 共通型（TenantContext / ErrorDetail）。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"

	// AuthInterceptor が context に詰めた AuthInfo（JWT 由来 tenant_id）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
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
//
// 互換: 既存の handler シグネチャを壊さないため、tenantContext のみで動作する path
// は残す。AuthInfo 由来の tenant_id 比較は requireTenantIDFromCtx（ctx 引数あり版）
// で実施する。HTTP gateway 経由 + AuthMode=jwks/hmac の場合、interceptor は
// req=nil で呼ばれ body 由来の tenant_id 検査ができないため、handler 段で
// AuthInfo.TenantID と body tenant_id の不一致を PermissionDenied で弾く必要がある。
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

// requireTenantIDFromCtx は context にある AuthInfo（JWT 由来 tenant_id）と
// proto body の tenant_id を比較して越境を弾く。AuthInfo が無い場合（auth=off の
// 開発環境）は body 由来の tenant_id をそのまま返す。ある場合は両者一致を要求する。
//
// NFR-E-AC-003 二重防御: gRPC interceptor は req=nil（HTTP gateway 経路）で
// extractTenantID() = "" になり mismatch 検査を skip するため、handler 段で
// 改めて検査する責務を負う。
func requireTenantIDFromCtx(goCtx context.Context, ctx *commonv1.TenantContext, rpc string) (string, error) {
	// 既存ロジック: body の tenant_id 必須検査。
	tid, err := requireTenantID(ctx, rpc)
	if err != nil {
		return "", err
	}
	// AuthInfo が context にあれば JWT 由来 tenant_id と一致を要求。
	if info, ok := common.AuthFromContext(goCtx); ok && info != nil && info.TenantID != "" {
		if info.TenantID != tid {
			return "", status.Errorf(codes.PermissionDenied,
				"tier1/state: cross-tenant request rejected (%s): jwt=%q body=%q",
				rpc, info.TenantID, tid)
		}
	}
	return tid, nil
}
