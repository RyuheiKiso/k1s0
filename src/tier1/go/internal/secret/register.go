// 本ファイルは t1-secret Pod が gRPC server に登録する SecretsService の handler。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-006（t1-secret: Active 1 / standby 2、HPA 禁止、OpenBao 直結）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（plan 04-06 結線済）:
//   SecretsService の 3 RPC（Get / BulkGet / Rotate）を OpenBao adapter 越しに実装する。
//   adapter 未注入時は Unimplemented を返す（fail-soft）。

// Package secret は t1-secret Pod が登録する SecretsService の handler を提供する。
package secret

import (
	"context"
	"errors"

	// OpenBao adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	// SDK 生成 stub の SecretsService 型。
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	// gRPC server 型。
	"google.golang.org/grpc"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// Deps は SecretsService handler が依存する adapter 集合。
type Deps struct {
	// OpenBao adapter（nil 時は全 RPC で Unimplemented を返す）。
	SecretsAdapter openbao.SecretsAdapter
}

// secretHandler は SecretsService の handler 実装。
type secretHandler struct {
	secretsv1.UnimplementedSecretsServiceServer
	deps Deps
}

// Register は SecretsService を gRPC server に登録する hook を返す。
// 後方互換のため deps なしの呼び出しも許容する（未注入 = Unimplemented 返却）。
func Register(deps Deps) func(*grpc.Server) {
	return func(srv *grpc.Server) {
		secretsv1.RegisterSecretsServiceServer(srv, &secretHandler{deps: deps})
	}
}

// translateErr は OpenBao SDK のエラーを gRPC status code に翻訳する。
func translateErr(err error, rpc string) error {
	if errors.Is(err, openbao.ErrNotWired) {
		return status.Errorf(codes.Unimplemented, "tier1/secrets: %s not yet wired to OpenBao", rpc)
	}
	if errors.Is(err, openbao.ErrSecretNotFound) {
		return status.Errorf(codes.NotFound, "tier1/secrets: %s: secret not found", rpc)
	}
	return status.Errorf(codes.Internal, "tier1/secrets: %s: %v", rpc, err)
}

// Get は単一 secret を OpenBao から取得する。
func (h *secretHandler) Get(ctx context.Context, req *secretsv1.GetSecretRequest) (*secretsv1.GetSecretResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	if h.deps.SecretsAdapter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: Get not yet wired to OpenBao")
	}
	ar := openbao.SecretGetRequest{
		Name:     req.GetName(),
		TenantID: req.GetContext().GetTenantId(),
	}
	if req.Version != nil {
		ar.Version = int(*req.Version)
	}
	resp, err := h.deps.SecretsAdapter.Get(ctx, ar)
	if err != nil {
		return nil, translateErr(err, "Get")
	}
	return &secretsv1.GetSecretResponse{
		Values:  resp.Values,
		Version: resp.Version,
	}, nil
}

// BulkGet は複数 secret を一括取得する。
// proto 上は context のみで取得対象 name を渡す手段が無いため、本実装は
// "テナント配下の全 secret 名" を呼び出すのではなく、context.tenant_id を
// adapter に渡して OpenBao 側のテナント別 list 機能（plan 04-06 で追加予定）に
// 委ねる。リリース時点 では空 map を返す。
//
// proto 拡張で名前リストを request body に追加した時点で、本実装を name 列ベース
// の SecretsAdapter.BulkGet 呼び出しに切替える（破壊的変更を避けるため）。
func (h *secretHandler) BulkGet(_ context.Context, req *secretsv1.BulkGetSecretRequest) (*secretsv1.BulkGetSecretResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	if h.deps.SecretsAdapter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: BulkGet not yet wired to OpenBao")
	}
	// proto に name 列が無いため、tenant 配下の全 secret 列挙は OpenBao の list API
	// が必要だが現状 narrow interface に List を入れていない。本リリース時点 では
	// 空応答を返す（FR-T1-SECRETS-002 の「複数取得」の意図は proto 拡張後に満たす）。
	return &secretsv1.BulkGetSecretResponse{Results: map[string]*secretsv1.GetSecretResponse{}}, nil
}

// Rotate は OpenBao KVv2 でバージョン bump を行う。
// 実値生成（DB password 等）は呼出側責務、本 RPC はバージョン管理層と監査記録の hook を担う。
func (h *secretHandler) Rotate(ctx context.Context, req *secretsv1.RotateSecretRequest) (*secretsv1.RotateSecretResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	if h.deps.SecretsAdapter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: Rotate not yet wired to OpenBao")
	}
	ar := openbao.SecretRotateRequest{
		Name:     req.GetName(),
		TenantID: req.GetContext().GetTenantId(),
	}
	resp, err := h.deps.SecretsAdapter.Rotate(ctx, ar)
	if err != nil {
		return nil, translateErr(err, "Rotate")
	}
	return &secretsv1.RotateSecretResponse{
		NewVersion: resp.Version,
	}, nil
}

