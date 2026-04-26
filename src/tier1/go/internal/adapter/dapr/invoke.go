// 本ファイルは Dapr Service Invocation building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Service Invoke API → Dapr Service Invocation
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
//
// リリース時点 placeholder。実 Dapr SDK 接続は plan 04-11 で実装。

package dapr

// 標準 Go ライブラリ。
import (
	// 全 RPC で context を伝搬する。
	"context"
)

// InvokeRequest は ServiceInvocation 呼出の入力。
type InvokeRequest struct {
	// 呼出先アプリ識別子（Dapr app_id）。
	AppID string
	// メソッド名（HTTP では path 相当）。
	Method string
	// 呼出データ。
	Data []byte
	// Content-Type。
	ContentType string
	// テナント識別子。
	TenantID string
	// タイムアウト（ミリ秒、0 で 5000ms 既定）。
	TimeoutMs int32
}

// InvokeResponse は ServiceInvocation 応答。
type InvokeResponse struct {
	// 応答本文。
	Data []byte
	// Content-Type。
	ContentType string
	// HTTP ステータス相当。
	Status int32
}

// InvokeAdapter は ServiceInvocation 操作の interface。
type InvokeAdapter interface {
	// 任意サービスの任意メソッド呼出。
	Invoke(ctx context.Context, req InvokeRequest) (InvokeResponse, error)
}

// daprInvokeAdapter は実装（リリース時点 placeholder）。
type daprInvokeAdapter struct {
	// Dapr Client への参照。
	client *Client
}

// NewInvokeAdapter は InvokeAdapter を生成する。
func NewInvokeAdapter(client *Client) InvokeAdapter {
	// 実装インスタンスを構築する。
	return &daprInvokeAdapter{client: client}
}

// Invoke は plan 04-11 で実装。
func (a *daprInvokeAdapter) Invoke(_ context.Context, _ InvokeRequest) (InvokeResponse, error) {
	// placeholder
	return InvokeResponse{}, ErrNotWired
}
