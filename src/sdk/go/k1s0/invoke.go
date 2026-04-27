// 本ファイルは k1s0 Go SDK の ServiceInvoke 動詞統一 facade。
// InvokeStream（サーバ stream）は本リリース時点 では raw 経由。
package k1s0

import (
	"context"

	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
)

// InvokeClient は InvokeService の動詞統一 facade。
type InvokeClient struct{ client *Client }

// Invoke は親 Client から InvokeClient を返す。
func (c *Client) Invoke() *InvokeClient { return c.invoke }

// Call は任意サービスの任意メソッドを呼び出す（unary）。
func (i *InvokeClient) Call(ctx context.Context, appID, method string, data []byte, contentType string, timeoutMs int32) (responseData []byte, responseContentType string, status int32, err error) {
	resp, e := i.client.raw.ServiceInvoke.Invoke(ctx, &serviceinvokev1.InvokeRequest{
		AppId:       appID,
		Method:      method,
		Data:        data,
		ContentType: contentType,
		Context:     i.client.tenantContext(),
		TimeoutMs:   timeoutMs,
	})
	if e != nil {
		return nil, "", 0, e
	}
	return resp.GetData(), resp.GetContentType(), resp.GetStatus(), nil
}
