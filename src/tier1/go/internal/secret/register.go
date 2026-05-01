// 本ファイルは t1-secret Pod が gRPC server に登録する SecretsService の handler。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-006（t1-secret: Active 1 / standby 2、HPA 禁止、OpenBao 直結）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（plan 04-06 結線済）:
//   SecretsService の 7 RPC（Get / BulkGet / Rotate / GetDynamic / Encrypt / Decrypt /
//   RotateKey）を OpenBao adapter 越しに実装する。adapter は cmd/secret/main.go で
//   必ず注入される（production: 実 OpenBao / dev: in-memory KVv2 backend）。

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

// Deps は SecretsService handler が依存する adapter 集合。cmd/secret/main.go で必ず
// 全フィールド非 nil で構築される。
type Deps struct {
	// 静的 secret 用 adapter（FR-T1-SECRETS-001）。
	SecretsAdapter openbao.SecretsAdapter
	// 動的 secret 用 adapter（FR-T1-SECRETS-002）。
	DynamicAdapter openbao.DynamicAdapter
	// Transit 暗号化 adapter（FR-T1-SECRETS-003、AES-256-GCM）。
	TransitAdapter openbao.TransitAdapter
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
// cmd/secret/main.go から非 nil な Deps と共に呼び出される。
func Register(deps Deps) func(*grpc.Server) {
	return func(srv *grpc.Server) {
		secretsv1.RegisterSecretsServiceServer(srv, &secretHandler{deps: deps})
	}
}

// translateErr は OpenBao SDK のエラーを gRPC status code に翻訳する。
func translateErr(err error, rpc string) error {
	if errors.Is(err, openbao.ErrSecretNotFound) {
		return status.Errorf(codes.NotFound, "tier1/secrets: %s: secret not found", rpc)
	}
	// Transit Decrypt 系: 鍵不在は NotFound、ciphertext 改竄/AAD 不一致は InvalidArgument。
	if errors.Is(err, openbao.ErrTransitKeyNotFound) {
		return status.Errorf(codes.NotFound, "tier1/secrets: %s: transit key not found", rpc)
	}
	if errors.Is(err, openbao.ErrTransitCiphertextMalformed) {
		return status.Errorf(codes.InvalidArgument, "tier1/secrets: %s: ciphertext malformed", rpc)
	}
	return status.Errorf(codes.Internal, "tier1/secrets: %s: %v", rpc, err)
}

// transitKeyName は <tenant_id>.<key_label> の形でテナント prefix を付与する。
// FR-T1-SECRETS-003 受け入れ基準「鍵名は <tenant_id>.<key_label> で tier1 が
// 自動プレフィックス」を満たすため、handler 段で必ず prefix を強制する。
func transitKeyName(tenantID, keyLabel string) string {
	return tenantID + "." + keyLabel
}

// Encrypt は AES-256-GCM で平文を暗号化する（FR-T1-SECRETS-003）。
func (h *secretHandler) Encrypt(ctx context.Context, req *secretsv1.EncryptRequest) (*secretsv1.EncryptResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// NFR-E-AC-003 二重防御: tenant_id 越境防止。
	tenantID, err := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.Encrypt")
	if err != nil {
		return nil, err
	}
	// key_name 必須（空は鍵空間が確定しないため不正）。
	if req.GetKeyName() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: key_name required")
	}
	// plaintext 必須（空 plaintext は意味があり得ても、API 利用では明示的な空入力を弾く）。
	if len(req.GetPlaintext()) == 0 {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: plaintext required (non-empty)")
	}
	resp, err := h.deps.TransitAdapter.Encrypt(openbao.TransitEncryptRequest{
		KeyName:   transitKeyName(tenantID, req.GetKeyName()),
		Plaintext: req.GetPlaintext(),
		AAD:       req.GetAad(),
	})
	if err != nil {
		return nil, translateErr(err, "Encrypt")
	}
	return &secretsv1.EncryptResponse{
		Ciphertext: resp.Ciphertext,
		KeyVersion: int32(resp.KeyVersion),
	}, nil
}

// Decrypt は AES-256-GCM で暗号文を復号する（FR-T1-SECRETS-003）。
func (h *secretHandler) Decrypt(ctx context.Context, req *secretsv1.DecryptRequest) (*secretsv1.DecryptResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// NFR-E-AC-003 二重防御。
	tenantID, err := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.Decrypt")
	if err != nil {
		return nil, err
	}
	if req.GetKeyName() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: key_name required")
	}
	if len(req.GetCiphertext()) == 0 {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: ciphertext required (non-empty)")
	}
	resp, err := h.deps.TransitAdapter.Decrypt(openbao.TransitDecryptRequest{
		KeyName:    transitKeyName(tenantID, req.GetKeyName()),
		Ciphertext: req.GetCiphertext(),
		AAD:        req.GetAad(),
	})
	if err != nil {
		return nil, translateErr(err, "Decrypt")
	}
	return &secretsv1.DecryptResponse{
		Plaintext:  resp.Plaintext,
		KeyVersion: int32(resp.KeyVersion),
	}, nil
}

// RotateKey は Transit 鍵を新版に上げる（FR-T1-SECRETS-003 受け入れ基準
// 「鍵バージョン管理が自動」、「復号時は暗号文中のバージョン番号から適切な鍵を選択」）。
func (h *secretHandler) RotateKey(ctx context.Context, req *secretsv1.RotateKeyRequest) (*secretsv1.RotateKeyResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// NFR-E-AC-003 二重防御。
	tenantID, err := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.RotateKey")
	if err != nil {
		return nil, err
	}
	if req.GetKeyName() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: key_name required")
	}
	resp, err := h.deps.TransitAdapter.RotateKey(openbao.TransitRotateKeyRequest{
		KeyName: transitKeyName(tenantID, req.GetKeyName()),
	})
	if err != nil {
		return nil, translateErr(err, "RotateKey")
	}
	return &secretsv1.RotateKeyResponse{
		NewVersion:      int32(resp.NewVersion),
		PreviousVersion: int32(resp.PreviousVersion),
		RotatedAtMs:     resp.RotatedAtMs,
	}, nil
}

// Get は単一 secret を OpenBao から取得する。
func (h *secretHandler) Get(ctx context.Context, req *secretsv1.GetSecretRequest) (*secretsv1.GetSecretResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: nil request")
	}
	// NFR-E-AC-003 二重防御: JWT 由来 tenant_id と body の一致を handler 段でも検証。
	tenantID, err := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.Get")
	if err != nil {
		return nil, err
	}
	// secret 名も必須（空名はテナント prefix のみで lookup → 誤動作の元）。
	if req.GetName() == "" {
		// InvalidArgument で返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: name required")
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
	// NFR-E-AC-003 二重防御: JWT 由来 tenant_id と body の一致を handler 段でも検証。
	tenantID, terr := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.BulkGet")
	if terr != nil {
		return nil, terr
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
	// NFR-E-AC-003 二重防御: JWT 由来 tenant_id と body の一致を handler 段でも検証。
	tenantID, terr := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.Rotate")
	if terr != nil {
		return nil, terr
	}
	// secret 名も必須（空名は rotate 対象が確定しないため不正）。
	if req.GetName() == "" {
		// InvalidArgument で返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: name required")
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
	// NFR-E-AC-003 二重防御: JWT 由来 tenant_id と body の一致を handler 段でも検証。
	tenantID, terr := common.EnforceTenantBoundary(ctx, req.GetContext().GetTenantId(), "Secrets.GetDynamic")
	if terr != nil {
		return nil, terr
	}
	// engine / role 必須。
	if req.GetEngine() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: engine required (e.g. \"postgres\")")
	}
	if req.GetRole() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/secrets: role required")
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

