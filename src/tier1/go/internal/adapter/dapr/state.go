// 本ファイルは Dapr State Management building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - State API → Valkey Cluster（Dapr State Management）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//
// 役割（リリース時点 placeholder）:
//   handler.go が呼び出す I/O を封じ込め、Dapr SDK との接続を将来差し込めるよう
//   interface ベースで境界を提供する。

package dapr

// 標準 Go ライブラリ。
import (
	// 全 RPC で context を伝搬する。
	"context"
)

// StateGetRequest は State Get 操作の adapter 入力。
// proto の k1s0.tier1.state.v1.GetRequest と等価だが、handler 側で TenantContext
// を分離して渡す形にする（Dapr SDK が tenant prefix を付与）。
type StateGetRequest struct {
	// Dapr Component 名（例: valkey-default）。
	Store string
	// テナント prefix 付与済キー。
	Key string
	// テナント識別子（Dapr metadata に渡す）。
	TenantID string
}

// StateGetResponse は State Get の応答。
type StateGetResponse struct {
	// 値本文（bytes 透過）。
	Data []byte
	// 楽観的排他用の ETag。
	Etag string
	// キー未存在時 true。
	NotFound bool
}

// StateSetRequest は Set / Delete 共通の入力。
type StateSetRequest struct {
	// Dapr Component 名。
	Store string
	// キー。
	Key string
	// 値本文（Set 時のみ）。
	Data []byte
	// 期待 ETag（楽観的排他、空は無条件）。
	ExpectedEtag string
	// TTL 秒数（0 で永続）。
	TTLSeconds int32
	// テナント識別子。
	TenantID string
}

// StateAdapter は State Management building block の操作集合。
// handler 側は本 interface に依存し、テスト時は mock 実装を注入できる。
type StateAdapter interface {
	// 単一キー取得。
	Get(ctx context.Context, req StateGetRequest) (StateGetResponse, error)
	// 単一キー保存。
	Set(ctx context.Context, req StateSetRequest) error
	// 単一キー削除。
	Delete(ctx context.Context, req StateSetRequest) error
}

// daprStateAdapter は Client を介して実 Dapr SDK を呼ぶ実装（リリース時点 placeholder）。
type daprStateAdapter struct {
	// Dapr Client への参照（本リリース時点 SidecarAddress のみ保持）。
	client *Client
}

// NewStateAdapter は Client から StateAdapter を生成する。
func NewStateAdapter(client *Client) StateAdapter {
	// daprStateAdapter は struct リテラルで初期化する。
	return &daprStateAdapter{client: client}
}

// Get は plan 04-04 で実装。本リリース時点は ErrNotWired を返す。
func (a *daprStateAdapter) Get(_ context.Context, _ StateGetRequest) (StateGetResponse, error) {
	// placeholder: 上位 handler が Unimplemented に変換する。
	return StateGetResponse{}, ErrNotWired
}

// Set は plan 04-04 で実装。
func (a *daprStateAdapter) Set(_ context.Context, _ StateSetRequest) error {
	// placeholder
	return ErrNotWired
}

// Delete は plan 04-04 で実装。
func (a *daprStateAdapter) Delete(_ context.Context, _ StateSetRequest) error {
	// placeholder
	return ErrNotWired
}
