// k1s0 Invoke ラッパー。
//
// SDK の InvokeClient.Call を per-request tenant 伝搬付きで露出する。
// Stream は BFF からは使わない想定（Server-Streaming は SSE / WebSocket で受け直すのが望ましい）。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// InvokeCall は他 Dapr アプリへの unary 呼出。
// timeoutMs=0 なら SDK 既定のデッドラインに従う。
func (c *Client) InvokeCall(ctx context.Context, appID, method string, data []byte, contentType string, timeoutMs int32) (responseData []byte, responseContentType string, status int32, err error) {
	// SDK facade を呼ぶ。
	return c.client.Invoke().Call(withTenantFromRequest(ctx), appID, method, data, contentType, timeoutMs)
}
