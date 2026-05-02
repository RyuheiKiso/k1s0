// 本ファイルは k1s0 Go SDK の Secrets 動詞統一 facade。
// `k1s0.Secrets().Get(...)` 形式で SecretsService への呼出を提供する。

package k1s0

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の SecretsService 型。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
)

// SecretsClient は SecretsService の動詞統一 facade。
type SecretsClient struct {
	// 親 Client への参照。
	client *Client
}

// Get はシークレット名で値（key=value マップ）を取得する。
func (s *SecretsClient) Get(ctx context.Context, name string) (values map[string]string, version int32, err error) {
	// proto Request を構築する。
	req := &secretsv1.GetSecretRequest{
		// シークレット名。
		Name: name,
		// TenantContext を継承する。
		Context: s.tenantContext(ctx),
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.Get(ctx, req)
	// gRPC エラー時はそのまま伝搬する。
	if e != nil {
		// caller に error を返却する。
		return nil, 0, e
	}
	// values と version を返却する。
	return resp.GetValues(), resp.GetVersion(), nil
}

// RotateOption は Rotate の任意パラメータを設定する。
type RotateOption func(*secretsv1.RotateSecretRequest)

// WithGracePeriod は旧バージョンの猶予時間（秒）を Rotate に渡す。
func WithGracePeriod(gracePeriodSec int32) RotateOption {
	// クロージャで RotateSecretRequest を変更する。
	return func(req *secretsv1.RotateSecretRequest) {
		// 猶予時間を設定する。
		req.GracePeriodSec = gracePeriodSec
	}
}

// WithIdempotencyKeyRotate は冪等性キーを Rotate に渡す（同一キーで再試行可能）。
func WithIdempotencyKeyRotate(key string) RotateOption {
	// クロージャで RotateSecretRequest を変更する。
	return func(req *secretsv1.RotateSecretRequest) {
		// 冪等性キーを設定する。
		req.IdempotencyKey = key
	}
}

// Rotate はシークレットのローテーションを実行する。新バージョンを返す。
func (s *SecretsClient) Rotate(ctx context.Context, name string, opts ...RotateOption) (newVersion, previousVersion int32, err error) {
	// proto Request を構築する。
	req := &secretsv1.RotateSecretRequest{
		// シークレット名。
		Name: name,
		// TenantContext を継承する。
		Context: s.tenantContext(ctx),
	}
	// 各 RotateOption を req に適用する。
	for _, opt := range opts {
		// クロージャを呼び出して req を変更する。
		opt(req)
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.Rotate(ctx, req)
	// gRPC エラー時はそのまま伝搬する。
	if e != nil {
		// caller に error を返却する。
		return 0, 0, e
	}
	// new_version / previous_version を返却する。
	return resp.GetNewVersion(), resp.GetPreviousVersion(), nil
}

// DynamicSecret は動的 Secret 発行（FR-T1-SECRETS-002）の応答を SDK 利用者向けに整理した型。
type DynamicSecret struct {
	// credential 一式（"username" / "password" など、engine 別の field）。
	Values map[string]string
	// OpenBao の lease ID（renewal / revoke 用）。
	LeaseID string
	// 実際に付与された TTL 秒（要求値から ceiling までクランプされる）。
	TTLSec int32
	// 発効時刻（Unix epoch ミリ秒）。
	IssuedAtMs int64
}

// GetDynamic は動的 Secret 発行（FR-T1-SECRETS-002）。
// engine="postgres" / "mysql" / "kafka" 等、OpenBao Database Engine の種別を指定する。
// ttlSec=0 で既定 1 時間、上限 24 時間。
func (s *SecretsClient) GetDynamic(ctx context.Context, engine, role string, ttlSec int32) (DynamicSecret, error) {
	// proto Request を構築する。
	req := &secretsv1.GetDynamicSecretRequest{
		Engine:  engine,
		Role:    role,
		TtlSec:  ttlSec,
		Context: s.tenantContext(ctx),
	}
	// gRPC 呼出。
	resp, err := s.client.raw.Secrets.GetDynamic(ctx, req)
	if err != nil {
		return DynamicSecret{}, err
	}
	// 応答を SDK 型に詰め替える。
	return DynamicSecret{
		Values:     resp.GetValues(),
		LeaseID:    resp.GetLeaseId(),
		TTLSec:     resp.GetTtlSec(),
		IssuedAtMs: resp.GetIssuedAtMs(),
	}, nil
}

// BulkSecret は BulkGet の 1 件分の結果（シークレット名 → values + version）。
type BulkSecret struct {
	// シークレット名。
	Name string
	// 値（key=value マップ）。
	Values map[string]string
	// バージョン番号。
	Version int32
}

// BulkGet はテナント配下の全シークレットを一括取得する（FR-T1-SECRETS-001）。
// tier1 側はテナント境界を自動付与し、当該テナントのシークレットのみ返す。
func (s *SecretsClient) BulkGet(ctx context.Context) ([]BulkSecret, error) {
	// proto Request を構築する。
	req := &secretsv1.BulkGetSecretRequest{
		Context: s.tenantContext(ctx),
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.BulkGet(ctx, req)
	if e != nil {
		return nil, e
	}
	// proto map を SDK 型のスライスに詰め替える。
	out := make([]BulkSecret, 0, len(resp.GetResults()))
	for name, sec := range resp.GetResults() {
		out = append(out, BulkSecret{
			Name:    name,
			Values:  sec.GetValues(),
			Version: sec.GetVersion(),
		})
	}
	return out, nil
}

// Encrypt は Transit Engine 経由の暗号化（FR-T1-SECRETS-003）。
// keyName は tier1 が <tenant_id>.<key_name> で自動 prefix する。
// aad は GCM の追加認証データ（同じ aad を Decrypt 時にも渡す必要あり）。
// 戻り値の ciphertext は version-prefixed nonce-embedded GCM 形式。
func (s *SecretsClient) Encrypt(ctx context.Context, keyName string, plaintext, aad []byte) (ciphertext []byte, keyVersion int32, err error) {
	// proto Request を構築する。
	req := &secretsv1.EncryptRequest{
		Context:   s.tenantContext(ctx),
		KeyName:   keyName,
		Plaintext: plaintext,
		Aad:       aad,
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.Encrypt(ctx, req)
	if e != nil {
		return nil, 0, e
	}
	return resp.GetCiphertext(), resp.GetKeyVersion(), nil
}

// Decrypt は Transit Engine 経由の復号（FR-T1-SECRETS-003）。
// keyName / aad は Encrypt 時と同じ値を渡すこと（GCM の整合性検証で必須）。
// 戻り値の keyVersion は復号に使われた鍵バージョン（旧版鍵で暗号化された場合の追跡用）。
func (s *SecretsClient) Decrypt(ctx context.Context, keyName string, ciphertext, aad []byte) (plaintext []byte, keyVersion int32, err error) {
	// proto Request を構築する。
	req := &secretsv1.DecryptRequest{
		Context:    s.tenantContext(ctx),
		KeyName:    keyName,
		Ciphertext: ciphertext,
		Aad:        aad,
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.Decrypt(ctx, req)
	if e != nil {
		return nil, 0, e
	}
	return resp.GetPlaintext(), resp.GetKeyVersion(), nil
}

// RotateKey は Transit Engine の鍵をローテーションする（FR-T1-SECRETS-003）。
// 既存版は保持され、その鍵で暗号化された ciphertext は引き続き Decrypt 可能。
// 新規 Encrypt は新版鍵を使う。戻り値は (新版, 旧版, ローテーション時刻 ms)。
func (s *SecretsClient) RotateKey(ctx context.Context, keyName string) (newVersion, previousVersion int32, rotatedAtMs int64, err error) {
	// proto Request を構築する。
	req := &secretsv1.RotateKeyRequest{
		Context: s.tenantContext(ctx),
		KeyName: keyName,
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.Secrets.RotateKey(ctx, req)
	if e != nil {
		return 0, 0, 0, e
	}
	return resp.GetNewVersion(), resp.GetPreviousVersion(), resp.GetRotatedAtMs(), nil
}

// tenantContext は ctx の per-request override を優先しつつ TenantContext proto を生成する。
// override 不在時は親 Client の Config から構築する。
func (s *SecretsClient) tenantContext(ctx context.Context) *commonv1.TenantContext {
	return s.client.tenantContext(ctx)
}
