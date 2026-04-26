// 本ファイルは Dapr Output Binding building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Binding API → 外部 HTTP / SMTP / S3（Dapr Output Binding）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
//
// リリース時点 placeholder。実 Dapr SDK 接続は plan 04-12 で実装。

package dapr

// 標準 Go ライブラリ。
import (
	// 全 RPC で context を伝搬する。
	"context"
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

// daprBindingAdapter は実装（リリース時点 placeholder）。
type daprBindingAdapter struct {
	// Dapr Client への参照。
	client *Client
}

// NewBindingAdapter は BindingAdapter を生成する。
func NewBindingAdapter(client *Client) BindingAdapter {
	// 実装インスタンスを構築する。
	return &daprBindingAdapter{client: client}
}

// Invoke は plan 04-12 で実装。
func (a *daprBindingAdapter) Invoke(_ context.Context, _ BindingRequest) (BindingResponse, error) {
	// placeholder
	return BindingResponse{}, ErrNotWired
}
