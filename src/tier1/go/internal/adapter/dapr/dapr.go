// 本ファイルは tier1 Go の Dapr Go SDK アダプタ層の起点。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - 5 モジュールパイプライン: API Router → Policy Enforcer → Dapr Adapter → Log Adapter / Metrics Emitter（DS-SW-COMP-020）
//   docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md
//     - internal/adapter/dapr/ に Dapr building block ラッパを配置
//   docs/02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md
//
// 役割（plan 04-04 Wire-up 完了）:
//   t1-state Pod が叩く 5 つの Dapr building block（State / PubSub / ServiceInvocation /
//   Output Binding / flagd-Binding）への薄いラッパを提供する。
//   State adapter が plan 04-04 で実 Dapr SDK と結線済（残りは plan 04-05 〜 04-13）。
//
// テスタビリティ設計:
//   外部 SDK との結合点を `daprStateClient` のような narrow interface に封じ込め、
//   実装側（GRPCClient）と試験用 fake を差し替え可能にする。これにより handler →
//   adapter → narrow interface の三段で各層を単独で unit-test できる。
//
// 未実装（plan 04-05 〜 04-13）:
//   - PubSub / ServiceInvocation / Output Binding / Feature の SDK 結線（plan 04-12 / 04-13）
//   - tier1 ErrorDetail へのエラーマッピング（gRPC google.rpc.Status → ErrorDetail）

// Package dapr は tier1 Go ファサードが Dapr Go SDK を呼ぶためのアダプタ層。
package dapr

import (
	// context.Context をシグネチャに含めるため import する（Dapr SDK 慣用）。
	"context"
	// 接続失敗 / 未実装エラーの整形に使う。
	"errors"

	// Dapr 公式 Go SDK。client.NewClientWithAddress で gRPC 接続を確立する。
	daprclient "github.com/dapr/go-sdk/client"
)

// ErrNotWired は Dapr backend と未結線である旨を示すセンチネルエラー。
// PubSub / Binding / Invoke / Feature など、まだ SDK 結線していない adapter は
// 本エラーを返し、handler 側で `errors.Is(err, dapr.ErrNotWired)` を判定して
// gRPC `codes.Unimplemented` に翻訳する。
var ErrNotWired = errors.New("tier1: Dapr backend not yet wired")

// daprStateClient は本パッケージが Dapr SDK から **実際に使う state 関連メソッド**
// だけを集めた narrow interface。`*daprclient.GRPCClient` がこれを満たすため、
// production では SDK インスタンスをそのまま注入し、test では fake を注入する。
//
// 抽象を Dapr SDK の Client 全体ではなく必要メソッドに絞る理由:
//   - 試験 fake が小さく済む（モック工数低減）
//   - Dapr SDK のメジャーアップグレードで影響する surface を最小化
type daprStateClient interface {
	// State 取得（Etag 込みで返却）。
	GetState(ctx context.Context, storeName, key string, meta map[string]string) (*daprclient.StateItem, error)
	// State 保存（meta で TTL 等を指定、so で consistency / concurrency を指定）。
	SaveState(ctx context.Context, storeName, key string, data []byte, meta map[string]string, so ...daprclient.StateOption) error
	// State 保存（楽観的排他: 期待 etag が一致した場合のみ書き込む）。
	SaveStateWithETag(ctx context.Context, storeName, key string, data []byte, etag string, meta map[string]string, so ...daprclient.StateOption) error
	// State 削除。
	DeleteState(ctx context.Context, storeName, key string, meta map[string]string) error
	// State 削除（楽観的排他）。
	DeleteStateWithETag(ctx context.Context, storeName, key string, etag *daprclient.ETag, meta map[string]string, opts *daprclient.StateOptions) error
}

// Client は tier1 Go ファサードから見た Dapr SDK のアダプタ。
// 本リリース時点では State 系のみ narrow interface として保持し、PubSub / Binding /
// Invoke / Feature は ErrNotWired を返す placeholder のまま（plan 04-05 〜 04-13 で同様に
// 結線する）。
type Client struct {
	// Dapr sidecar の gRPC 接続先（観測性 / デバッグ用途で SidecarAddress() から参照される）。
	sidecarAddress string
	// 実 Dapr SDK の State 用 client（試験では fake を差し替え可能）。
	state daprStateClient
	// SDK Client インスタンス（Close 時に SDK の Close を呼ぶ必要があるため保持）。
	// fake 注入時は nil。
	closer interface{ Close() }
}

// Config は Client 初期化時に渡される設定。
type Config struct {
	// Dapr sidecar の gRPC 接続先（例: "localhost:50001"）。
	// 既定は dapr.io/app-port=50001 と整合。
	SidecarAddress string
}

// New は Config から Client を生成し、Dapr SDK の gRPC 接続を確立する。
// 接続検証は SDK 内部で行われ、失敗時は err を返す。
func New(_ context.Context, cfg Config) (*Client, error) {
	// アドレス未指定時は Dapr 慣用既定値を使う。
	addr := cfg.SidecarAddress
	if addr == "" {
		addr = defaultDaprAddress
	}
	// Dapr SDK Client（gRPC）を構築する。SDK 内部で gRPC.Dial を呼ぶ。
	sdkClient, err := daprclient.NewClientWithAddress(addr)
	if err != nil {
		return nil, err
	}
	// Client インスタンスを返却（state は SDK の同一 Client を narrow interface 越しに保持）。
	return &Client{
		sidecarAddress: addr,
		state:          sdkClient,
		closer:         sdkClient,
	}, nil
}

// NewWithStateClient は test 用コンストラクタ。任意の daprStateClient 実装を
// 受け取って Client を構築する。production の New と異なり SDK 接続は行わない。
func NewWithStateClient(addr string, sc daprStateClient) *Client {
	// addr は SidecarAddress() のために保持する。
	return &Client{sidecarAddress: addr, state: sc, closer: nil}
}

// Close は Client が保持する Dapr SDK Client の gRPC 接続を解放する。
func (c *Client) Close() error {
	// fake 注入時は closer が nil なので no-op。
	if c.closer == nil {
		return nil
	}
	// Dapr SDK Client の Close は err を返さないため適用後に nil を返す。
	c.closer.Close()
	return nil
}

// SidecarAddress は Client が想定する Dapr sidecar のアドレスを返す。
// 観測性 / デバッグ用途で main から参照される。
func (c *Client) SidecarAddress() string {
	return c.sidecarAddress
}

// stateClient は内部の state-用 narrow client を返す。
// adapter 実装（state.go）からのみ使う。
func (c *Client) stateClient() daprStateClient {
	return c.state
}

// defaultDaprAddress は Dapr sidecar の既定 gRPC ポート。
// docs 正典: docs/04_概要設計/.../02_Daprファサード層コンポーネント.md（Dapr sidecar 50001）
const defaultDaprAddress = "localhost:50001"
