// 本ファイルは k1s0 Go SDK の Binding 動詞統一 facade。
package k1s0

import (
	"context"

	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
)

// BindingClient は BindingService の動詞統一 facade。
type BindingClient struct{ client *Client }

// Binding は親 Client から BindingClient を返す。
func (c *Client) Binding() *BindingClient { return c.binding }

// Invoke は出力バインディング呼出（外部 HTTP / SMTP / S3 等）。
func (b *BindingClient) Invoke(ctx context.Context, name, operation string, data []byte, metadata map[string]string) (responseData []byte, responseMetadata map[string]string, err error) {
	resp, e := b.client.raw.Binding.Invoke(ctx, &bindingv1.InvokeBindingRequest{
		Name:      name,
		Operation: operation,
		Data:      data,
		Metadata:  metadata,
		Context:   b.client.tenantContext(ctx),
	})
	if e != nil {
		return nil, nil, e
	}
	return resp.GetData(), resp.GetMetadata(), nil
}
