// k1s0 SDK Client の Infrastructure 層ラッパー。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   k1s0 SDK Client を tier2 サービス側の Infrastructure 境界として再露出する。
//   Domain / Application 層からは本ラッパー経由でのみ k1s0 を呼ぶ（直接 SDK 型に依存させない）。

// Package external は k1s0 / 他外部システムへのアクセスを集約する Infrastructure 層。
package external

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// k1s0 SDK 高水準 facade。
	"github.com/k1s0/sdk-go/k1s0"

	// stock-reconciler 設定。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
	// shared dapr ラッパー（k1s0.Client を組み立てる共通ロジック）。
	shareddapr "github.com/k1s0/k1s0/src/tier2/go/shared/dapr"
)

// K1s0Client は SDK Client の薄いラッパー。Closer / Save / Get / Publish の最小 API のみ露出する。
type K1s0Client struct {
	// SDK Client（接続を保持する）。
	client *k1s0.Client
}

// NewK1s0Client は config から K1s0Client を組み立てる。
func NewK1s0Client(ctx context.Context, cfg config.K1s0Config) (*K1s0Client, error) {
	// shared dapr ラッパーを使って k1s0 SDK Client を初期化する。
	c, err := shareddapr.NewClient(ctx, shareddapr.Config{
		// gRPC target。
		Target: cfg.Target,
		// テナント ID。
		TenantID: cfg.TenantID,
		// 主体識別子。
		Subject: cfg.Subject,
		// TLS 利用フラグ。
		UseTLS: cfg.UseTLS,
	})
	// 接続失敗はそのまま伝搬する。
	if err != nil {
		// 呼出元で fail-fast。
		return nil, err
	}
	// ラッパーを返す。
	return &K1s0Client{client: c}, nil
}

// Close は SDK Client を解放する。
func (c *K1s0Client) Close() error {
	// nil ガード。
	if c == nil || c.client == nil {
		// 何もしない。
		return nil
	}
	// SDK Client の Close を委譲する。
	return c.client.Close()
}

// StateGet は k1s0 State から指定キーを取得する。
//
// 未存在時は (nil, "", false, nil)。
func (c *K1s0Client) StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error) {
	// SDK facade を呼ぶ。
	return c.client.State().Get(ctx, store, key)
}

// StateSave は k1s0 State にキーを保存する。新 ETag を返す。
func (c *K1s0Client) StateSave(ctx context.Context, store, key string, data []byte) (string, error) {
	// SDK facade を呼ぶ（オプション無し）。
	return c.client.State().Save(ctx, store, key, data)
}

// StateSaveWithEtag は楽観的排他付きで保存する。
func (c *K1s0Client) StateSaveWithEtag(ctx context.Context, store, key string, data []byte, expectedEtag string) (string, error) {
	// WithExpectedEtag option を渡して呼ぶ。
	return c.client.State().Save(ctx, store, key, data, k1s0.WithExpectedEtag(expectedEtag))
}

// PubSubPublish は指定 topic に publish する。返り値は offset。
func (c *K1s0Client) PubSubPublish(ctx context.Context, topic string, data []byte, contentType, idempotencyKey string) (int64, error) {
	// idempotencyKey が空なら option を付けない。
	if idempotencyKey == "" {
		// 通常 publish。
		return c.client.PubSub().Publish(ctx, topic, data, contentType)
	}
	// 冪等性キー付き publish。
	return c.client.PubSub().Publish(ctx, topic, data, contentType, k1s0.WithIdempotencyKey(idempotencyKey))
}
