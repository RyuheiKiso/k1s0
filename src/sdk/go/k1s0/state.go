// 本ファイルは k1s0 Go SDK の State 動詞統一 facade。
// `k1s0.State().Save(...)` 形式で StateService への呼出を提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//   README.md コードサンプル

package k1s0

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の StateService 型。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
)

// StateClient は StateService の動詞統一 facade。
type StateClient struct {
	// 親 Client への参照（接続 / TenantContext を共有）。
	client *Client
}

// SetOption は Save の任意パラメータを設定する。
type SetOption func(*statev1.SetRequest)

// WithExpectedEtag は楽観的排他のための ETag を Save に渡す。
func WithExpectedEtag(etag string) SetOption {
	// クロージャで SetRequest を変更する。
	return func(req *statev1.SetRequest) {
		// 期待 ETag を設定する。
		req.ExpectedEtag = etag
	}
}

// WithTTL は TTL（秒）を Save に渡す。
func WithTTL(ttlSec int32) SetOption {
	// クロージャで SetRequest を変更する。
	return func(req *statev1.SetRequest) {
		// TTL 秒を設定する。
		req.TtlSec = ttlSec
	}
}

// WithSetIdempotencyKey は Save の冪等性キーを設定する（共通規約 §「冪等性と再試行」）。
// 同 key の再投入は tier1 側で 24h 重複抑止される（最初のレスポンスを再生する）。
// PubSub.Publish の WithIdempotencyKey との関数名衝突を避けるため別名にしている。
func WithSetIdempotencyKey(key string) SetOption {
	return func(req *statev1.SetRequest) {
		req.IdempotencyKey = key
	}
}

// Get はキー単位の取得。未存在時は (nil, "", false, nil) を返す。
func (s *StateClient) Get(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error) {
	// proto Request を構築する。
	req := &statev1.GetRequest{
		// Store 名。
		Store: store,
		// キー（テナント prefix は tier1 が自動付与）。
		Key: key,
		// 親 Client から TenantContext を継承する。
		Context: s.tenantContext(ctx),
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.State.Get(ctx, req)
	// gRPC エラー時はそのまま伝搬する。
	if e != nil {
		// caller に error を返却する。
		return nil, "", false, e
	}
	// 未存在時は found=false で短絡する。
	if resp.GetNotFound() {
		// 値は空、エラーなし。
		return nil, "", false, nil
	}
	// 取得成功。
	return resp.GetData(), resp.GetEtag(), true, nil
}

// Save はキー単位の保存。新 ETag を返す。
// オプション WithExpectedEtag / WithTTL を可変長引数で受ける。
func (s *StateClient) Save(ctx context.Context, store, key string, data []byte, opts ...SetOption) (newEtag string, err error) {
	// proto Request を構築する。
	req := &statev1.SetRequest{
		// Store 名。
		Store: store,
		// キー。
		Key: key,
		// 値本文。
		Data: data,
		// TenantContext を継承する。
		Context: s.tenantContext(ctx),
	}
	// 各 SetOption を req に適用する。
	for _, opt := range opts {
		// クロージャを呼び出して req を変更する。
		opt(req)
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := s.client.raw.State.Set(ctx, req)
	// gRPC エラー時はそのまま伝搬する。
	if e != nil {
		// caller に error を返却する。
		return "", e
	}
	// 新 ETag を返却する。
	return resp.GetNewEtag(), nil
}

// Delete はキー単位の削除。expected_etag が空なら無条件削除。
func (s *StateClient) Delete(ctx context.Context, store, key, expectedEtag string) error {
	// proto Request を構築する。
	req := &statev1.DeleteRequest{
		// Store 名。
		Store: store,
		// キー。
		Key: key,
		// 期待 ETag（空は無条件）。
		ExpectedEtag: expectedEtag,
		// TenantContext を継承する。
		Context: s.tenantContext(ctx),
	}
	// 生成 stub 経由で RPC 呼び出し。
	_, e := s.client.raw.State.Delete(ctx, req)
	// エラーをそのまま伝搬する。
	return e
}

// tenantContext は ctx の per-request override を優先しつつ TenantContext proto を生成する。
// override 不在時は親 Client の Config から構築する（log.go の tenantContext と同方針）。
func (s *StateClient) tenantContext(ctx context.Context) *commonv1.TenantContext {
	return s.client.tenantContext(ctx)
}
