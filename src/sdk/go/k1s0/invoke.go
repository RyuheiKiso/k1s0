// 本ファイルは k1s0 Go SDK の ServiceInvoke 動詞統一 facade。
// 当初 unary のみだったが、Stream（サーバストリーミング）を追加し全 RPC を網羅。
package k1s0

import (
	"context"
	"errors"
	"io"

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
		Context:     i.client.tenantContext(ctx),
		TimeoutMs:   timeoutMs,
	})
	if e != nil {
		return nil, "", 0, e
	}
	return resp.GetData(), resp.GetContentType(), resp.GetStatus(), nil
}

// InvokeChunkHandler は Stream で受信した各 chunk を処理するコールバック。
// 戻り値の error が non-nil なら Stream は中断され、handler のエラーが Stream() の戻り値になる。
type InvokeChunkHandler func(chunk *serviceinvokev1.InvokeChunk) error

// Stream はサーバストリーミング呼出。受信した各 chunk を handler に渡す。
// stream 終端（io.EOF）で正常終了、context 完了 / handler error / RPC error で中断する。
// EOF は nil を返す。
func (i *InvokeClient) Stream(ctx context.Context, appID, method string, data []byte, contentType string, timeoutMs int32, handler InvokeChunkHandler) error {
	// proto Request を構築する。
	req := &serviceinvokev1.InvokeRequest{
		AppId: appID, Method: method, Data: data, ContentType: contentType,
		Context: i.client.tenantContext(ctx), TimeoutMs: timeoutMs,
	}
	// 生成 stub の InvokeStream を起動する。
	stream, err := i.client.raw.ServiceInvoke.InvokeStream(ctx, req)
	if err != nil {
		return err
	}
	// 各 chunk を受信し handler に渡す。EOF は正常終了。
	for {
		chunk, err := stream.Recv()
		// 終端時。
		if errors.Is(err, io.EOF) {
			return nil
		}
		// それ以外の RPC error は伝搬する。
		if err != nil {
			return err
		}
		// handler のエラーは Stream を中断させる。
		if err := handler(chunk); err != nil {
			return err
		}
	}
}
