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
		Context: s.tenantContext(),
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
		Context: s.tenantContext(),
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

// tenantContext は親 Client の Config から TenantContext proto を生成する。
func (s *SecretsClient) tenantContext() *commonv1.TenantContext {
	// 構造体リテラルで TenantContext を構築する。
	return &commonv1.TenantContext{
		// テナント ID。
		TenantId: s.client.cfg.TenantID,
		// subject。
		Subject: s.client.cfg.Subject,
		// correlation_id は OTel interceptor 後段付与。
		CorrelationId: "",
	}
}
