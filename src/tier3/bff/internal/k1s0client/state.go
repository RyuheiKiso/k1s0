// k1s0 State ラッパー。
//
// SDK の StateClient.Get / Save / Delete を per-request tenant 伝搬付きで露出する。
// BFF REST / GraphQL から呼び、SDK 型を上位層に漏らさない。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// StateGet は k1s0 State から指定キーを取得する。
// auth middleware の tenant_id を SDK へ伝搬する。
func (c *Client) StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error) {
	// SDK facade を呼ぶ。
	return c.client.State().Get(withTenantFromRequest(ctx), store, key)
}

// StateSave は k1s0 State にキーを保存する。
// auth middleware の tenant_id を SDK へ伝搬する。
func (c *Client) StateSave(ctx context.Context, store, key string, data []byte) (string, error) {
	// SDK facade を呼ぶ。
	return c.client.State().Save(withTenantFromRequest(ctx), store, key, data)
}

// StateDelete は k1s0 State から指定キーを削除する。
// expectedEtag が空でなければ optimistic concurrency control を効かせる。
// auth middleware の tenant_id を SDK へ伝搬する。
func (c *Client) StateDelete(ctx context.Context, store, key, expectedEtag string) error {
	// SDK facade を呼ぶ。
	return c.client.State().Delete(withTenantFromRequest(ctx), store, key, expectedEtag)
}
