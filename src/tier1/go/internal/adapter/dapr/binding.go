// 本ファイルは Dapr Output Binding building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Binding API → 外部 HTTP / SMTP / S3（Dapr Output Binding）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
//
// 役割（plan 04-12 結線済）:
//   handler.go が呼び出す Output Binding 操作（create / get / list / delete / send 等）を
//   Dapr SDK の InvokeBinding で実行する。応答 BindingEvent.Data と Metadata を返却する。
//   テナント識別子は metadata に同梱して component 側に伝搬する。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"

	// Dapr SDK の InvokeBindingRequest / BindingEvent 型を参照する。
	daprclient "github.com/dapr/go-sdk/client"
)

// BindingRequest は Output Binding 呼出の入力。
type BindingRequest struct {
	// Dapr Component 名（s3-archive / smtp-notify 等、運用設定）。
	Name string
	// 操作種別（create / get / list / delete / send 等）。
	Operation string
	// 操作データ本文。
	Data []byte
	// メタデータ。
	Metadata map[string]string
	// テナント識別子。
	TenantID string
}

// BindingResponse は Output Binding 応答。
type BindingResponse struct {
	// 応答本文（バインディング型依存）。
	Data []byte
	// メタデータ。
	Metadata map[string]string
}

// BindingAdapter は Output Binding 操作の interface。
type BindingAdapter interface {
	// 出力バインディング呼出。
	Invoke(ctx context.Context, req BindingRequest) (BindingResponse, error)
}

// daprBindingAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type daprBindingAdapter struct {
	client *Client
}

// NewBindingAdapter は BindingAdapter を生成する。
func NewBindingAdapter(client *Client) BindingAdapter {
	return &daprBindingAdapter{client: client}
}

// Invoke は Output Binding を呼び出す。
// metadata にテナント識別子を含めて sidecar に伝搬する（呼出元 metadata は破壊しない）。
func (a *daprBindingAdapter) Invoke(ctx context.Context, req BindingRequest) (BindingResponse, error) {
	// metadata 合成（呼出元 map を破壊しないため新規 map を作る）。
	meta := make(map[string]string, len(req.Metadata)+1)
	for k, v := range req.Metadata {
		meta[k] = v
	}
	if req.TenantID != "" {
		meta[metadataKeyTenant] = req.TenantID
	}
	// SDK 呼出。Name / Operation / Data / Metadata を構造体に詰める。
	in := &daprclient.InvokeBindingRequest{
		Name:      req.Name,
		Operation: req.Operation,
		Data:      req.Data,
		Metadata:  meta,
	}
	ev, err := a.client.bindingClient().InvokeBinding(ctx, in)
	if err != nil {
		return BindingResponse{}, err
	}
	// SDK は BindingEvent.Data と Metadata を返す。
	if ev == nil {
		return BindingResponse{}, nil
	}
	return BindingResponse{
		Data:     ev.Data,
		Metadata: ev.Metadata,
	}, nil
}
