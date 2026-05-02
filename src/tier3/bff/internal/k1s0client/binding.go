// k1s0 Binding ラッパー。
//
// SDK の BindingClient.Invoke を per-request tenant 伝搬付きで露出する。
// HTTP / SMTP / S3 等の Output Binding を呼ぶ用途。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// BindingInvoke は Output Binding を呼出して外部システムへ出力する。
// operation は binding component が定義する動詞（例: "create" / "send"）。
func (c *Client) BindingInvoke(ctx context.Context, name, operation string, data []byte, metadata map[string]string) (responseData []byte, responseMetadata map[string]string, err error) {
	// SDK facade を呼ぶ。
	return c.client.Binding().Invoke(withTenantFromRequest(ctx), name, operation, data, metadata)
}
