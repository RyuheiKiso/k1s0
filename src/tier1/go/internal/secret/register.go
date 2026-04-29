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
	// 現在時刻を Rotate 応答の rotated_at_ms に詰めるため。
	"time"

	// OpenBao adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
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
	// 静的 secret 用 adapter（nil 時は Get / BulkGet / Rotate が Unimplemented）。
	SecretsAdapter openbao.SecretsAdapter
	// 動的 secret 用 adapter（FR-T1-SECRETS-002、nil 時は GetDynamic が Unimplemented）。
	DynamicAdapter openbao.DynamicAdapter
	// Rotate の冪等性 cache（共通規約 §「冪等性と再試行」: 24h TTL で同一 idempotency_key
	// 再試行時に初回 response を返す）。nil の場合は dedup なし（後方互換 / 早期 dev）。
	Idempotency common.IdempotencyCache
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
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tenantID := req.GetContext().GetTenantId()
	// tenantID 未設定はテナント境界違反として弾く（adapter の path 構築前で短絡）。
	if tenantID == "" {
		// InvalidArgument で返却する（BulkGet / GetDynamic / Rotate と一貫）。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: tenant_id required in TenantContext")
	}
	// secret 名も必須（空名はテナント prefix のみで lookup → 誤動作の元）。
	if req.GetName() == "" {
		// InvalidArgument で返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: name required")
	}
	if h.deps.SecretsAdapter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: Get not yet wired to OpenBao")
	}
	ar := openbao.SecretGetRequest{
		Name:     req.GetName(),
		TenantID: tenantID,
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

// BulkGet はテナント配下の全 secret を取得する。
// proto は context のみを受け、name 列は持たないため、adapter.ListAndGet が
// `tenant/<tenantID>/` prefix で List → Get を内部実行する。
// FR-T1-SECRETS-002（テナントに割当された全シークレット）対応。
func (h *secretHandler) BulkGet(ctx context.Context, req *secretsv1.BulkGetSecretRequest) (*secretsv1.BulkGetSecretResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// adapter 未注入時は未結線扱い。
	if h.deps.SecretsAdapter == nil {
		// Unimplemented 返却。
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: BulkGet not yet wired to OpenBao")
	}
	// テナント識別子を取り出す（必須）。
	tenantID := req.GetContext().GetTenantId()
	// tenantID 未設定はテナント境界違反として弾く（NFR-E-AC-003）。
	if tenantID == "" {
		// 不正引数として返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: tenant_id required in TenantContext")
	}
	// adapter で list + per-key get を実行する。
	items, err := h.deps.SecretsAdapter.ListAndGet(ctx, tenantID)
	// adapter エラーを翻訳して返す。
	if err != nil {
		// translateErr で gRPC code に翻訳する。
		return nil, translateErr(err, "BulkGet")
	}
	// proto 応答 map を構築する。
	results := make(map[string]*secretsv1.GetSecretResponse, len(items))
	// 取得結果を 1 件ずつ proto 応答に詰める。
	for name, resp := range items {
		// 1 件分の proto 応答を構築する。
		results[name] = &secretsv1.GetSecretResponse{
			// values map をコピー渡しする。
			Values: resp.Values,
			// version を詰める。
			Version: resp.Version,
		}
	}
	// 応答を返却する。
	return &secretsv1.BulkGetSecretResponse{Results: results}, nil
}

// Rotate は OpenBao KVv2 でバージョン bump を行う。
// 実値生成（DB password 等）は呼出側責務、本 RPC はバージョン管理層と監査記録の hook を担う。
//
// proto 応答の全フィールドを埋める:
//   - new_version: bump 後のバージョン
//   - previous_version: 直前のバージョン（new_version - 1、初回は 0）
//   - rotated_at_ms: 実行時刻（UTC Unix epoch ミリ秒）
//   - ttl_sec: 静的 secret は 0 固定（動的 secret は plan 04-06 後段で算出）
func (h *secretHandler) Rotate(ctx context.Context, req *secretsv1.RotateSecretRequest) (*secretsv1.RotateSecretResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数として返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tenantID := req.GetContext().GetTenantId()
	// tenantID 未設定はテナント境界違反として弾く（rotate 対象の誤同定を防止）。
	if tenantID == "" {
		// InvalidArgument で返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: tenant_id required in TenantContext")
	}
	// secret 名も必須（空名は rotate 対象が確定しないため不正）。
	if req.GetName() == "" {
		// InvalidArgument で返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: name required")
	}
	// adapter 未注入時は未結線扱い。
	if h.deps.SecretsAdapter == nil {
		// Unimplemented を返却する。
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: Rotate not yet wired to OpenBao")
	}
	// 実 rotate 実行クロージャ。idempotency cache hit 時は呼ばれない。
	doRotate := func() (interface{}, error) {
		ar := openbao.SecretRotateRequest{
			Name:     req.GetName(),
			TenantID: tenantID,
		}
		resp, err := h.deps.SecretsAdapter.Rotate(ctx, ar)
		if err != nil {
			return nil, translateErr(err, "Rotate")
		}
		// 直前バージョン（new_version - 1）を計算する（初回は 0）。
		prev := resp.Version - 1
		if prev < 0 {
			prev = 0
		}
		return &secretsv1.RotateSecretResponse{
			NewVersion:      resp.Version,
			PreviousVersion: prev,
			RotatedAtMs:     time.Now().UnixMilli(),
			// 静的 secret は TTL 0、動的 secret は GetDynamic 経路で発行する。
			TtlSec: 0,
		}, nil
	}
	// 共通規約 §「冪等性と再試行」: 同一 idempotency_key の再試行は初回 response を返す。
	idempKey := common.IdempotencyKey(tenantID, "Secrets.Rotate", req.GetIdempotencyKey())
	if idempKey == "" || h.deps.Idempotency == nil {
		out, err := doRotate()
		if err != nil {
			return nil, err
		}
		return out.(*secretsv1.RotateSecretResponse), nil
	}
	out, err := h.deps.Idempotency.GetOrCompute(ctx, idempKey, doRotate)
	if err != nil {
		return nil, err
	}
	return out.(*secretsv1.RotateSecretResponse), nil
}

// GetDynamic は動的 Secret 発行（FR-T1-SECRETS-002）。
// engine="postgres" 等の OpenBao Database Engine が TTL 付き credential を都度発行する。
func (h *secretHandler) GetDynamic(ctx context.Context, req *secretsv1.GetDynamicSecretRequest) (*secretsv1.GetDynamicSecretResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// テナント未指定はテナント境界違反として弾く（NFR-E-AC-003）。
	tenantID := req.GetContext().GetTenantId()
	if tenantID == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: tenant_id required in TenantContext")
	}
	// engine / role 必須。
	if req.GetEngine() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: engine required (e.g. \"postgres\")")
	}
	if req.GetRole() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: role required")
	}
	// adapter 未注入時は未結線扱い（Unimplemented）。
	if h.deps.DynamicAdapter == nil {
		return nil, status.Error(codes.Unimplemented, "tier1/secrets: GetDynamic not yet wired to OpenBao Database Engine")
	}
	// adapter 入力に変換する。
	ar := openbao.DynamicSecretRequest{
		Engine:     req.GetEngine(),
		Role:       req.GetRole(),
		TenantID:   tenantID,
		TTLSeconds: req.GetTtlSec(),
	}
	// adapter で発行する。
	resp, err := h.deps.DynamicAdapter.GetDynamic(ctx, ar)
	if err != nil {
		return nil, translateErr(err, "GetDynamic")
	}
	// 応答を proto に詰め替える。
	return &secretsv1.GetDynamicSecretResponse{
		Values:     resp.Values,
		LeaseId:    resp.LeaseID,
		TtlSec:     resp.TTLSeconds,
		IssuedAtMs: resp.IssuedAtMs,
	}, nil
}

